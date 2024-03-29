use std::f32::consts::FRAC_PI_2;

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;

use crate::{app_state::AppState, config::Config, debug_mode::is_in_debug_mode};

const MAX_PITCH: f32 = FRAC_PI_2 - 0.01;

#[derive(Component, Default)]
pub struct Player {
    velocity: Vec3,
    grounded: bool,
}

#[derive(Event)]
pub struct PlayerMovement;

fn player_spawn_transform(config: &Config) -> Transform {
    let mut position = config.spawn_position;
    position.y += config.player_capsule_radius;
    Transform::from_translation(position)
}

fn player_camera_spawn_transform(config: &Config) -> Transform {
    Transform::from_xyz(0.0, config.player_camera_height, 0.0)
        .looking_at(Vec3::new(1.0, config.player_camera_height, 0.0), Vec3::Y)
}

fn setup_player(mut commands: Commands, config: Res<Config>) {
    let camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                transform: player_camera_spawn_transform(&config),
                tonemapping: Tonemapping::TonyMcMapface,
                ..default()
            },
            BloomSettings::default(),
        ))
        .id();
    let player_capsule = commands
        .spawn((
            RigidBody::KinematicPositionBased,
            Collider::capsule(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, config.player_capsule_cylinder_height, 0.0),
                config.player_capsule_radius,
            ),
            TransformBundle::from(player_spawn_transform(&config)),
            KinematicCharacterController {
                up: Vec3::Y,
                ..default()
            },
            Player::default(),
            Name::new("Player"),
        ))
        .id();
    commands.entity(player_capsule).push_children(&[camera]);
}

fn player_force_push(
    mut commands: Commands,
    buttons: Res<Input<MouseButton>>,
    rapier_context: Res<RapierContext>,
    player_query: Query<Entity, With<Player>>,
    camera_query: Query<(&Parent, &Transform, &GlobalTransform), With<Camera>>,
    entity_names: Query<&Name>,
    rigid_bodies: Query<&RigidBody>,
    config: Res<Config>,
) {
    if buttons.just_pressed(MouseButton::Right) {
        for (parent, transform, global_transform) in &camera_query {
            let Ok(player) = player_query.get(parent.get()) else {
                warn!("Parent of camera has no kinematic character controller!");
                continue;
            };
            let ray_pos = global_transform.translation();
            let ray_dir: Vec3 = -transform.local_z();
            let filter = QueryFilter::new();
            if let Some((entity, toi)) = rapier_context.cast_ray(
                ray_pos,
                ray_dir,
                config.player_force_push_max_distance,
                true,
                filter.exclude_rigid_body(player),
            ) {
                let Ok(name) = entity_names.get(entity) else {
                    continue;
                };

                info!("HIT '{}' toi={}", name, toi);

                if let Ok(rigid_body) = rigid_bodies.get(entity) {
                    if *rigid_body == RigidBody::Dynamic {
                        let impulse = ExternalImpulse {
                            impulse: -ray_dir * config.player_force_push_velocity,
                            torque_impulse: Vec3::ZERO,
                        };
                        commands.entity(entity).insert(impulse);
                    }
                }
            }
        }
    }
}

fn player_movement(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut KinematicCharacterController, &mut Player)>,
    camera_query: Query<(&Parent, &Transform), With<Camera>>,
    mut player_movement: EventWriter<PlayerMovement>,
    config: Res<Config>,
) {
    for (parent, transform) in &camera_query {
        let Ok((mut controller, mut player)) = player_query.get_mut(parent.get()) else {
            warn!("Parent of camera has no kinematic character controller!");
            continue;
        };
        // This is mostly taken from bevy_flycam's movement code.
        let mut velocity = Vec3::ZERO;
        let local_z = transform.local_z();
        let forward = -Vec3::new(local_z.x, 0., local_z.z);
        let right = Vec3::new(local_z.z, 0., -local_z.x);

        if keys.just_pressed(KeyCode::Space) && player.grounded {
            player.velocity = Vec3::new(0.0, config.jump_velocity, 0.0);
            player.grounded = false;
        } else if player.grounded && player.velocity.y < 0.0 {
            // Don't set this to 0 or the physics engine will constantly flip us between
            // being grounded and not-grounded each frame.
            player.velocity.y = -0.1;
        } else if !player.grounded {
            player.velocity.y -= config.gravity * time.delta_seconds();
        }

        for key in keys.get_pressed() {
            match key {
                KeyCode::W => {
                    velocity += forward;
                }
                KeyCode::A => {
                    velocity -= right;
                }
                KeyCode::S => {
                    velocity -= forward;
                }
                KeyCode::D => {
                    velocity += right;
                }
                _ => {}
            }
        }

        velocity = velocity.normalize_or_zero();

        if velocity != Vec3::ZERO {
            player_movement.send(PlayerMovement);
        }

        let desired_translation = velocity * time.delta_seconds() * config.player_speed
            + player.velocity * time.delta_seconds();

        controller.translation = Some(desired_translation);
    }
}

fn update_player_after_physics(
    mut query: Query<(&mut Player, &KinematicCharacterControllerOutput)>,
) {
    for (mut player, output) in query.iter_mut() {
        player.grounded = output.grounded;
    }
}

fn player_look(
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Camera3d>>,
    config: Res<Config>,
) {
    let Ok(window) = primary_window.get_single() else {
        warn!("No primary window when trying to mouselook!");
        return;
    };
    for mut transform in &mut query {
        for event in motion_events.iter() {
            // This is mostly taken from bevy_flycam's mouselook code.
            let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            // Using smallest of height or width ensures equal vertical and horizontal sensitivity.
            let window_scale = window.height().min(window.width());
            pitch -= (config.mouse_sensitivity * event.delta.y * window_scale).to_radians();
            yaw -= (config.mouse_sensitivity * event.delta.x * window_scale).to_radians();

            pitch = pitch.clamp(-MAX_PITCH, MAX_PITCH);

            // Order is important to prevent unintended roll.
            transform.rotation =
                Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        }
    }
}

fn maybe_respawn_player(
    mut player_query: Query<(&mut Player, &mut Transform), Without<Camera>>,
    mut camera_query: Query<(&Parent, &mut Transform), With<Camera>>,
    config: Res<Config>,
) {
    for (parent, mut camera_transform) in &mut camera_query {
        let Ok((mut player, mut player_transform)) = player_query.get_mut(parent.get()) else {
            warn!("Parent of camera has no kinematic character controller!");
            continue;
        };

        if player_transform.translation.y < config.fall_off_level_y {
            // Really we are teleporting the player back to their spawn position,
            // rather than respawning them. Also, this could run into weird edge
            // cases, e.g. if the player pushed a crate over their spawn position,
            // but this is better than dooming the player to an infinite fall.
            *player = default();
            *player_transform = player_spawn_transform(&config);
            *camera_transform = player_camera_spawn_transform(&config);
        }
    }
}

fn grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        window.cursor.grab_mode = CursorGrabMode::Confined;
        window.cursor.visible = false;
    } else {
        warn!("No primary window when trying to grab cursor!");
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::SettingUpScene), setup_player)
            .add_systems(Startup, grab_cursor)
            .add_systems(
                Update,
                (
                    maybe_respawn_player,
                    player_movement.run_if(not(is_in_debug_mode)),
                    player_look.run_if(not(is_in_debug_mode)),
                    player_force_push.run_if(not(is_in_debug_mode)),
                    update_player_after_physics,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_event::<PlayerMovement>();
    }
}

use std::f32::consts::FRAC_PI_2;

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;

use crate::{app_state::AppState, config::Config};

/// Gravity speed in meters per second.
const GRAVITY_SPEED: f32 = 5.0;

const MAX_PITCH: f32 = FRAC_PI_2 - 0.01;

#[derive(Component)]
pub struct Player;

pub struct PlayerMovement;

fn setup_player(mut commands: Commands, config: Res<Config>) {
    let camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                transform: Transform::from_xyz(0.0, config.player_camera_height, 0.0)
                    .looking_at(Vec3::new(1.0, config.player_camera_height, 0.0), Vec3::Y),
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
            TransformBundle::from(Transform::from_xyz(0.0, config.player_capsule_radius, 0.0)),
            KinematicCharacterController {
                up: Vec3::Y,
                ..default()
            },
            Player,
        ))
        .id();
    commands.entity(player_capsule).push_children(&[camera]);
}

fn player_movement(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut controller_query: Query<&mut KinematicCharacterController>,
    camera_query: Query<(&Parent, &Transform), With<Camera>>,
    mut player_movement: EventWriter<PlayerMovement>,
    config: Res<Config>,
) {
    for (parent, transform) in &camera_query {
        let Ok(mut controller) = controller_query.get_mut(parent.get()) else {
            warn!("Parent of camera has no kinematic character controller!");
            continue;
        };
        // This is mostly taken from bevy_flycam's movement code.
        let mut velocity = Vec3::ZERO;
        let local_z = transform.local_z();
        let forward = -Vec3::new(local_z.x, 0., local_z.z);
        let right = Vec3::new(local_z.z, 0., -local_z.x);

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

        let gravity = Vec3::NEG_Y * GRAVITY_SPEED * time.delta_seconds();
        let desired_translation = velocity * time.delta_seconds() * config.player_speed + gravity;

        controller.translation = Some(desired_translation);
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
        app.add_system(setup_player.in_schedule(OnEnter(AppState::SettingUpScene)))
            .add_startup_system(grab_cursor)
            .add_systems((player_movement, player_look).in_set(OnUpdate(AppState::InGame)))
            .add_event::<PlayerMovement>();
    }
}

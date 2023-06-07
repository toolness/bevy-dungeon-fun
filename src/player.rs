use std::f32::consts::FRAC_PI_2;

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    ecs::event::ManualEventReader,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;

/// Player speed in meters per second.
const PLAYER_SPEED: f32 = 5.0;

/// The distance of the camera from the bottom of the player's capsule.
const CAMERA_HEIGHT: f32 = 1.0;

/// The radius of the player's capsule.
const CAPSULE_RADIUS: f32 = 0.5;

/// The height of the cylindrical part of the player's capsule.
const CAPSULE_CYLINDER_HEIGHT: f32 = 1.0;

const MOUSE_SENSITIVITY: f32 = 0.00012;

const MAX_PITCH: f32 = FRAC_PI_2 - 0.01;

#[derive(Resource, Default)]
struct MouseMotionState(ManualEventReader<MouseMotion>);

fn setup_player(mut commands: Commands) {
    let camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                transform: Transform::from_xyz(0.0, CAMERA_HEIGHT, 0.0)
                    .looking_at(Vec3::new(1.0, CAMERA_HEIGHT, 0.0), Vec3::Y),
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
                Vec3::new(0.0, CAPSULE_CYLINDER_HEIGHT, 0.0),
                CAPSULE_RADIUS,
            ),
            TransformBundle::from(Transform::from_xyz(0.0, CAPSULE_RADIUS, 0.0)),
            KinematicCharacterController {
                up: Vec3::Y,
                ..default()
            },
        ))
        .id();
    commands.entity(player_capsule).push_children(&[camera]);
}

fn player_movement(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut controller_query: Query<&mut KinematicCharacterController>,
    camera_query: Query<(&Parent, &Transform), With<Camera>>,
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

        let desired_translation = velocity * time.delta_seconds() * PLAYER_SPEED;

        controller.translation = Some(desired_translation);
    }
}

fn player_look(
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut mouse_motion: ResMut<MouseMotionState>,
    motion_events: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(window) = primary_window.get_single() else {
        warn!("No primary window when trying to mouselook!");
        return;
    };
    for mut transform in &mut query {
        for event in mouse_motion.0.iter(&motion_events) {
            // This is mostly taken from bevy_flycam's mouselook code.
            let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            // Using smallest of height or width ensures equal vertical and horizontal sensitivity.
            let window_scale = window.height().min(window.width());
            pitch -= (MOUSE_SENSITIVITY * event.delta.y * window_scale).to_radians();
            yaw -= (MOUSE_SENSITIVITY * event.delta.x * window_scale).to_radians();

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
        app.init_resource::<MouseMotionState>()
            .add_startup_system(setup_player)
            .add_startup_system(grab_cursor)
            .add_system(player_movement)
            .add_system(player_look);
    }
}

use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    prelude::*,
};
use bevy_rapier3d::prelude::*;

/// Player speed in meters per second.
const PLAYER_SPEED: f32 = 5.0;

/// The distance of the camera from the bottom of the player's capsule.
const CAMERA_HEIGHT: f32 = 1.0;

/// The radius of the player's capsule.
const CAPSULE_RADIUS: f32 = 0.25;

/// The height of the cylindrical part of the player's capsule.
const CAPSULE_CYLINDER_HEIGHT: f32 = 1.0;

fn setup_player(mut commands: Commands) {
    let camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..default()
                },
                transform: Transform::from_xyz(0.0, CAMERA_HEIGHT, 0.5)
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

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_player)
            .add_system(player_movement);
    }
}

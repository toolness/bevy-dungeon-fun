use bevy::{prelude::*, pbr::PointLightShadowMap};
use bevy_flycam::prelude::*;

fn main() {
    let blueish = Color::Rgba { red: 0.052, green: 0.049, blue: 0.097, alpha: 1.0 };
    App::new().insert_resource(AmbientLight {
        color: blueish,
        brightness: 5.0 / 5.0f32,
    })
    .insert_resource(ClearColor(blueish))
    .insert_resource(PointLightShadowMap { size: 4096 })
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup)
    .add_plugin(NoCameraPlayerPlugin)
    .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_height = 1.5;
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, player_height, 0.5).looking_at(Vec3::new(1.0, player_height, 0.0), Vec3::Y),
            ..default()
        },
        FlyCam
    ));
    let scene = asset_server.load("dungeon.gltf#Scene0");
    commands.spawn(SceneBundle {
        scene,
        ..default()
    });
}

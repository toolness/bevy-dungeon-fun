use bevy::{prelude::*, pbr::{PointLightShadowMap, NotShadowCaster}};
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
    .add_system(enable_shadows_on_lights)
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

fn enable_shadows_on_lights(mut commands: Commands, mut query: Query<&mut PointLight>, other_query: Query<(Entity, &Name), Without<NotShadowCaster>>) {
    let mut found = false;
    for mut entity in &mut query {
        if !entity.shadows_enabled {
            println!("Enabling shadows for light!");
            entity.shadows_enabled = true;
            found = true;
        }
    }

    if found {
        for (entity, name) in &other_query {
            println!("Disabling shadows on {}", name);
            // This is the mesh for the torch objects.
            if name.starts_with("Cylinder.001") {
                println!("Entity: {:?}", name);
                commands.entity(entity).insert(NotShadowCaster);
            }
        }
    }
}

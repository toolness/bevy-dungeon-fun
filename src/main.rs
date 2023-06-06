use bevy::{
    asset::LoadState,
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    pbr::{NotShadowCaster, PointLightShadowMap},
    prelude::*,
    window::WindowMode,
};
use bevy_flycam::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    let windowed = std::env::args().any(|a| a == "--windowed");
    let mode = if windowed {
        WindowMode::Windowed
    } else {
        WindowMode::BorderlessFullscreen
    };
    let blueish = Color::Rgba {
        red: 0.052,
        green: 0.049,
        blue: 0.097,
        alpha: 1.0,
    };
    App::new()
        .insert_resource(AmbientLight {
            color: blueish,
            brightness: 5.0 / 5.0f32,
        })
        .insert_resource(ClearColor(blueish))
        .insert_resource(PointLightShadowMap { size: 4096 })
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            always_on_top: true,
            ..default()
        })
        .add_system(bevy::window::close_on_esc.run_if(is_not_wasm))
        .add_startup_system(setup)
        .add_startup_system(setup_physics)
        .add_system(fix_lighting.run_if(did_scene_load.and_then(run_once())))
        .add_plugin(NoCameraPlayerPlugin)
        .run();
}

const GLTF_SCENE: &str = "dungeon.gltf#Scene0";

fn is_not_wasm() -> bool {
    !cfg!(target_arch = "wasm32")
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_height = 1.5;
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(0.0, player_height, 0.5)
                .looking_at(Vec3::new(1.0, player_height, 0.0), Vec3::Y),
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings::default(),
        FlyCam,
    ));
    let scene = asset_server.load(GLTF_SCENE);
    commands.spawn(SceneBundle { scene, ..default() });
}

fn setup_physics(mut commands: Commands) {
    commands
        .spawn(Collider::cuboid(100.0, 0.1, 100.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)));

    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(1.5))
        .insert(TransformBundle::from(Transform::from_xyz(4.0, 4.0, 0.0)));
}

fn did_scene_load(asset_server: Res<AssetServer>) -> bool {
    let handle = asset_server.get_handle_untyped(GLTF_SCENE);
    asset_server.get_load_state(handle) == LoadState::Loaded
}

fn fix_lighting(
    mut commands: Commands,
    mut query: Query<(&Name, &mut PointLight)>,
    other_query: Query<(Entity, &Name), Without<NotShadowCaster>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (name, mut light) in &mut query {
        if !light.shadows_enabled {
            // Even though we've told the light to cast shadows in Blender, either Blender's glTF exporter doesn't
            // export this, or bevy doesn't import it. Either way, we need to enable it manually.
            info!("Enabling shadows for light {}.", name);
            light.shadows_enabled = true;
        }
    }

    for (entity, name) in &other_query {
        if name.starts_with("TorchCylinder") {
            // We've set the torches to not cast shadows in Blender, but either Blender's glTF exporter doesn't
            // export this, or bevy doesn't import it. Either way, we need to set it manually.
            info!("Disabling shadows on {}.", name);
            commands.entity(entity).insert(NotShadowCaster);
        }
    }

    for (_handle, mat) in materials.iter_mut() {
        let [r, g, b, _a] = mat.emissive.as_linear_rgba_f32();
        let scale: f32 = 10.0;
        if r > 0.0 || g > 0.0 || b > 0.0 {
            // Bring the color into HDR space so bevy applies bloom to it.
            //
            // TODO: We should probably grab the material by name, rather than scaling
            // every single emissive material.
            info!(
                "Scaling emissive {:?} by a factor of {}.",
                mat.emissive, scale
            );
            mat.emissive = Color::rgb_linear(r * scale, g * scale, b * scale);
        }
    }
}

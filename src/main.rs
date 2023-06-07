mod player;

use bevy::{
    asset::LoadState,
    input::keyboard,
    pbr::{NotShadowCaster, PointLightShadowMap},
    prelude::*,
    window::WindowMode,
};
use bevy_rapier3d::prelude::*;
use player::PlayerPlugin;

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
            enabled: false,
            ..default()
        })
        .add_system(bevy::window::close_on_esc.run_if(is_not_wasm))
        .add_startup_system(setup_scene)
        .add_plugin(PlayerPlugin)
        .add_system(toggle_debug_mode)
        .add_systems(
            (
                fix_scene_emissive_materials,
                fix_scene_point_lights,
                fix_scene_torches,
                fix_scene_physics,
            )
                .distributive_run_if(did_scene_load)
                .distributive_run_if(run_once()),
        )
        .run();
}

const GLTF_SCENE: &str = "dungeon.gltf#Scene0";

fn is_not_wasm() -> bool {
    !cfg!(target_arch = "wasm32")
}

fn setup_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    let scene = asset_server.load(GLTF_SCENE);
    commands.spawn(SceneBundle { scene, ..default() });
}

fn toggle_debug_mode(
    keyboard_input: Res<Input<keyboard::KeyCode>>,
    mut context: ResMut<DebugRenderContext>,
) {
    if keyboard_input.just_pressed(keyboard::KeyCode::G) {
        context.enabled = !context.enabled;
    }
}

fn did_scene_load(asset_server: Res<AssetServer>) -> bool {
    let handle = asset_server.get_handle_untyped(GLTF_SCENE);
    asset_server.get_load_state(handle) == LoadState::Loaded
}

fn fix_scene_physics(
    mut commands: Commands,
    mut query: Query<(Entity, &Name, &mut Visibility, &Transform, &Children)>,
    child_meshes_query: Query<(&Name, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
) {
    let mut count = 0;
    info!("Iterating over {} meshes.", query.iter().count());
    for (entity, name, mut visibility, transform, children) in &mut query {
        if name.ends_with("-colonly") {
            count += 1;
            *visibility = Visibility::Hidden;
            let Some(child) = children.first() else {
                warn!(
                    "colonly object {} has no children, expected 1.",
                    name,
                );
                continue;
            };
            let Ok((child_name, mesh_handle)) = child_meshes_query.get(*child) else {
                warn!(
                    "colonly object {} first child has no mesh.",
                    name,
                );
                continue;
            };
            if children.len() > 1 {
                warn!(
                    "colonly object {} has {} children, expected 1.",
                    name,
                    children.len()
                );
                continue;
            }
            let Some(mesh) = meshes.get(mesh_handle) else {
                warn!(
                    "colonly object {} mesh {} not loaded.",
                    name,
                    child_name
                );
                continue;
            };
            let Some(collider) = Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh) else {
                warn!(
                    "unable to generate rapier collider from colonly object {} mesh {}.",
                    name,
                    child_name
                );
                continue;
            };
            commands.entity(entity).insert(collider);
        }
    }

    info!("Converted {} collision-only meshes.", count);
}

fn fix_scene_point_lights(mut query: Query<&mut PointLight>) {
    let mut count = 0;
    for mut light in &mut query {
        if !light.shadows_enabled {
            // Even though we've told the light to cast shadows in Blender, either Blender's glTF exporter doesn't
            // export this, or bevy doesn't import it. Either way, we need to enable it manually.
            count += 1;
            light.shadows_enabled = true;
        }
    }

    info!("Enabled shadows for {} point lights.", count);
}

fn fix_scene_torches(
    mut commands: Commands,
    query: Query<(Entity, &Name), Without<NotShadowCaster>>,
) {
    let mut count = 0;
    for (entity, name) in &query {
        if name.starts_with("TorchCylinder") {
            // We've set the torches to not cast shadows in Blender, but either Blender's glTF exporter doesn't
            // export this, or bevy doesn't import it. Either way, we need to set it manually.
            count += 1;
            commands.entity(entity).insert(NotShadowCaster);
        }
    }

    info!("Disabled shadows for {} torches.", count);
}

fn fix_scene_emissive_materials(mut materials: ResMut<Assets<StandardMaterial>>) {
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

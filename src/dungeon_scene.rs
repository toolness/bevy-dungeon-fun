use bevy::{pbr::NotShadowCaster, prelude::*, scene::SceneInstance};
use bevy_rapier3d::prelude::*;

use crate::app_state::AppState;

const GLTF_SCENE: &str = "dungeon.gltf#Scene0";

fn load_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading scene...");
    let scene = asset_server.load(GLTF_SCENE);
    commands
        .spawn(SceneBundle { scene, ..default() })
        .insert(DungeonScene);
}

#[derive(Component)]
struct DungeonScene;

pub struct DungeonScenePlugin;

impl Plugin for DungeonScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(load_scene)
            .add_system(wait_for_scene_to_load.in_set(OnUpdate(AppState::LoadingAssets)))
            .add_systems(
                (
                    fix_scene_emissive_materials,
                    fix_scene_point_lights,
                    fix_scene_torches,
                    fix_scene_physics,
                    start_game,
                )
                    .chain()
                    .in_schedule(OnEnter(AppState::SettingUpScene)),
            );
    }
}

fn start_game(mut next_state: ResMut<NextState<AppState>>) {
    info!("Finished setting up scene! Starting game...");
    next_state.set(AppState::InGame);
}

// Our SceneBundle has loaded once SceneInstance has been added to it.
fn wait_for_scene_to_load(
    query: Query<&DungeonScene, With<SceneInstance>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !query.is_empty() {
        info!("Scene is loaded! Setting it up...");
        next_state.set(AppState::SettingUpScene);
    }
}

fn fix_scene_physics(
    mut commands: Commands,
    mut query: Query<(Entity, &Name, &mut Visibility, &Children)>,
    child_meshes_query: Query<(&Name, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
) {
    let mut count = 0;
    info!("Iterating over {} meshes.", query.iter().count());
    for (entity, name, mut visibility, children) in &mut query {
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

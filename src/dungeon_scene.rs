use bevy::{pbr::NotShadowCaster, prelude::*, render::primitives::Aabb, scene::SceneInstance};
use bevy_rapier3d::prelude::*;

use crate::app_state::AppState;

const GLTF_SCENE: &str = "dungeon.gltf#Scene0";

#[derive(Resource, Default)]
struct AssetsLoading(Vec<HandleUntyped>);

fn load_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    info!("Loading scene...");
    let scene = asset_server.load(GLTF_SCENE);
    loading.0.push(scene.clone_untyped());
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
            .init_resource::<AssetsLoading>()
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
// But also, we need to wait for our resources to load too, which will
// take longer to load on Web.
//
// Aside: This is really really confusing and I have spent literally
// half this project just trying to figure out how to *reliably*
// wait for the level to load so I can start modifying it.
fn wait_for_scene_to_load(
    scene_query: Query<&DungeonScene, With<SceneInstance>>,
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    use bevy::asset::LoadState;

    if !scene_query.is_empty() {
        match server.get_group_load_state(loading.0.iter().map(|handle| handle.id())) {
            LoadState::Loaded => {
                info!("Scene is loaded! Setting it up...");
                next_state.set(AppState::SettingUpScene);
            }
            LoadState::Failed => {
                error!("Scene failed to load!");
            }
            LoadState::Loading => {
                info!("Scene is still loading...");
            }
            _ => {}
        }
    }
}

fn fix_scene_physics(
    mut commands: Commands,
    mut query: Query<(Entity, &Name, &mut Visibility, &Children)>,
    child_meshes_query: Query<(&Name, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
) {
    let mut colonly_count = 0;
    let mut rigid_count = 0;
    info!("Iterating over {} meshes.", query.iter().count());
    for (entity, name, mut visibility, children) in &mut query {
        if name.ends_with("-colonly") {
            colonly_count += 1;
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
            commands
                .entity(entity)
                .insert(collider)
                .insert(RigidBody::Fixed);
        } else if name.ends_with("-rigid") {
            rigid_count += 1;
            let mut aabb = Aabb::default();
            for child in children.iter() {
                let Ok((child_name, mesh_handle)) = child_meshes_query.get(*child) else {
                    warn!(
                        "rigid object {} child has no mesh.",
                        name,
                    );
                    continue;
                };
                let Some(mesh) = meshes.get(mesh_handle) else {
                    warn!(
                        "rigid object {} mesh {} not loaded.",
                        name,
                        child_name
                    );
                    continue;
                };
                // This isn't very efficient, as we're recomputing it for every single instance
                // of every single mesh. We don't have many mehes in our scene so it's not that
                // big a deal, though.
                let Some(mesh_aabb) = mesh.compute_aabb() else {
                    warn!(
                        "rigid object {} mesh {} has no AABB.",
                        name,
                        child_name
                    );
                    continue;
                };
                aabb = union_aabb(&aabb, &mesh_aabb);
            }
            // This makes assumptions about the AABB, namely that the object is symmetrical around
            // its origin. Fortunately, this is actually the case for our barrels and crates.
            let collider = if name.contains("Barrel") {
                // It's a barrel.
                Collider::cylinder(aabb.half_extents.y, aabb.half_extents.x)
            } else {
                // It's a crate.
                Collider::cuboid(
                    aabb.half_extents.x,
                    aabb.half_extents.y,
                    aabb.half_extents.z,
                )
            };
            commands
                .entity(entity)
                .insert(RigidBody::Dynamic)
                .insert(collider);
        }
    }

    info!(
        "Converted {} collision-only meshes and added {} rigid body colliders.",
        colonly_count, rigid_count
    );
}

fn union_aabb(a: &Aabb, b: &Aabb) -> Aabb {
    let min = a.min().min(b.min());
    let max = a.max().max(b.max());
    Aabb::from_min_max(min.into(), max.into())
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

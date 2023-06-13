use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

use crate::app_state::{AppState, AssetsLoading};

#[derive(serde::Deserialize, bevy::reflect::TypeUuid, Resource, Default, Clone)]
#[uuid = "83187ffe-c216-4626-803f-e2a96e016323"]
pub struct Config {
    /// Player speed in meters per second.
    pub player_speed: f32,
    /// The distance of the camera from the bottom of the player's capsule.
    pub player_camera_height: f32,
    /// The radius of the player's capsule.
    pub player_capsule_radius: f32,
    /// The height of the cylindrical part of the player's capsule.
    pub player_capsule_cylinder_height: f32,
    pub mouse_sensitivity: f32,
    /// Multiply the colors of all emissive materials by this amount.
    /// This will put the colors into the HDR space so bevy can apply
    /// bloom to it, etc.
    pub emissive_scale: f32,
    /// Gravity in meters per second squared.
    pub gravity: f32,
    /// Jump velocity in meters per second.
    pub jump_velocity: f32,
    /// If the player's y-coordinate is below this value, they've fallen
    /// off the level and should be respawned.
    pub fall_off_level_y: f32,
    /// If the player falls off the level, this will be their y-coordinate
    /// when they're respawned.
    pub spawn_position: Vec3,
    /// Instructions shown at beginning of game.
    pub instructions: String,
    pub player_force_push_max_distance: f32,
    pub player_force_push_velocity: f32,
}

fn load_config(asset_server: ResMut<AssetServer>, mut loading: ResMut<AssetsLoading>) {
    let config: Handle<Config> = asset_server.load("config.json");
    loading.0.push(config.clone_untyped());
}

fn apply_config(mut config: ResMut<Config>, loaded_config: Res<Assets<Config>>) {
    for (_, loaded) in loaded_config.iter() {
        info!("Loaded configuration.");
        // Technically this means we are storing two copies of the configuration,
        // but it's pretty small and much more convenient to access as a global
        // resource than via an asset handle.
        *config = loaded.clone();
        return;
    }
    error!("No configuration found!");
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(JsonAssetPlugin::<Config>::new(&["json"]))
            .init_resource::<Config>()
            .add_startup_system(load_config)
            .add_system(apply_config.in_schedule(OnExit(AppState::LoadingAssets)));
    }
}

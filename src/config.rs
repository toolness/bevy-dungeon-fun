use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

use crate::app_state::{AppState, AssetsLoading};

#[derive(serde::Deserialize, bevy::reflect::TypeUuid, Resource, Default, Copy, Clone)]
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
}

fn load_config(asset_server: ResMut<AssetServer>, mut loading: ResMut<AssetsLoading>) {
    let config: Handle<Config> = asset_server.load("config.json");
    loading.0.push(config.clone_untyped());
}

fn apply_config(mut config: ResMut<Config>, loaded_config: Res<Assets<Config>>) {
    for (_, loaded) in loaded_config.iter() {
        info!("Loaded configuration.");
        *config = *loaded;
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

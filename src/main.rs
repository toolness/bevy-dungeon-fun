mod app_state;
mod config;
mod debug_mode;
mod dungeon_scene;
mod instructions;
mod player;

use app_state::AppState;
use bevy::{input::keyboard, pbr::PointLightShadowMap, prelude::*, window::WindowMode};
use bevy_rapier3d::prelude::*;
use config::ConfigPlugin;
use debug_mode::DebugModePlugin;
use dungeon_scene::DungeonScenePlugin;
use instructions::InstructionsPlugin;
use player::PlayerPlugin;

fn main() {
    let windowed = std::env::args().any(|a| a == "--windowed" || a == "-w");
    let mode = if windowed {
        WindowMode::Windowed
    } else {
        WindowMode::BorderlessFullscreen
    };
    App::new()
        .add_state::<AppState>()
        .insert_resource(PointLightShadowMap { size: 4096 })
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin {
            enabled: false,
            ..default()
        })
        .insert_resource(GizmoConfig {
            enabled: true,
            depth_bias: -1.0,
            ..Default::default()
        })
        .add_systems(Update, bevy::window::close_on_esc.run_if(is_not_wasm))
        .add_plugins(ConfigPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(DungeonScenePlugin)
        .add_plugins(InstructionsPlugin)
        .add_plugins(DebugModePlugin)
        .add_systems(Update, toggle_rapier_debug_render_mode)
        .run();
}

fn is_not_wasm() -> bool {
    !cfg!(target_arch = "wasm32")
}

fn toggle_rapier_debug_render_mode(
    keyboard_input: Res<Input<keyboard::KeyCode>>,
    mut context: ResMut<DebugRenderContext>,
) {
    if keyboard_input.just_pressed(keyboard::KeyCode::G) {
        context.enabled = !context.enabled;
    }
}

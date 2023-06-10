mod app_state;
mod dungeon_scene;
mod instructions;
mod player;

use app_state::AppState;
use bevy::{input::keyboard, pbr::PointLightShadowMap, prelude::*, window::WindowMode};
use bevy_rapier3d::prelude::*;
use dungeon_scene::DungeonScenePlugin;
use instructions::InstructionsPlugin;
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
        .add_state::<AppState>()
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
        .add_plugin(PlayerPlugin)
        .add_plugin(DungeonScenePlugin)
        .add_plugin(InstructionsPlugin)
        .add_system(toggle_debug_mode)
        .run();
}

fn is_not_wasm() -> bool {
    !cfg!(target_arch = "wasm32")
}

fn toggle_debug_mode(
    keyboard_input: Res<Input<keyboard::KeyCode>>,
    mut context: ResMut<DebugRenderContext>,
) {
    if keyboard_input.just_pressed(keyboard::KeyCode::G) {
        context.enabled = !context.enabled;
    }
}

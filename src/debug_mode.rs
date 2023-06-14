use bevy::{prelude::*, window::PrimaryWindow};

#[cfg(feature = "debug_mode")]
fn toggle_grab_cursor(
    keys: Res<Input<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    use bevy::window::CursorGrabMode;

    if keys.just_pressed(KeyCode::Grave) {
        if let Ok(mut window) = primary_window.get_single_mut() {
            if window.cursor.visible {
                window.cursor.grab_mode = CursorGrabMode::Confined;
                window.cursor.visible = false;
            } else {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }
        } else {
            warn!("No primary window when trying to grab cursor!");
        }
    }
}

pub fn is_in_debug_mode(primary_window: Query<&Window, With<PrimaryWindow>>) -> bool {
    if let Ok(window) = primary_window.get_single() {
        window.cursor.visible
    } else {
        false
    }
}

pub struct DebugModePlugin;

impl Plugin for DebugModePlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut bevy::prelude::App) {
        {
            #[cfg(feature = "debug_mode")]
            {
                app.add_system(toggle_grab_cursor);
                let inspector = bevy_inspector_egui::quick::WorldInspectorPlugin::new();
                app.add_plugin(inspector.run_if(is_in_debug_mode));
            }
        }
    }
}

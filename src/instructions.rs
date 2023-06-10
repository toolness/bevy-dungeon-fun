use bevy::prelude::*;

use crate::player::PlayerMovement;

#[derive(Component)]
struct InstructionText;

fn show_instructions(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 30.0,
        color: Color::WHITE,
    };
    commands.spawn((
        TextBundle::from_section("Use WASD to move and mouse to look.", text_style)
            .with_text_alignment(TextAlignment::Left)
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(10.0),
                    top: Val::Px(10.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
        InstructionText,
    ));
}

fn hide_instructions(
    mut commands: Commands,
    query: Query<Entity, With<InstructionText>>,
    player_movement: EventReader<PlayerMovement>,
) {
    if let Ok(instructions) = query.get_single() {
        if !player_movement.is_empty() {
            commands.entity(instructions).despawn();
        }
    }
}

pub struct InstructionsPlugin;

impl Plugin for InstructionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(show_instructions)
            .add_system(hide_instructions);
    }
}

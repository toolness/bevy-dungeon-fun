use bevy::prelude::*;

#[derive(Component)]
struct InstructionText;

fn instructions(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 30.0,
        color: Color::WHITE,
    };
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: -1,
            ..Default::default()
        },
        ..Default::default()
    });
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

pub struct InstructionsPlugin;

impl Plugin for InstructionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(instructions);
    }
}

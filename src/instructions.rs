use bevy::prelude::*;

use crate::{
    app_state::{AppState, AssetsLoading},
    config::Config,
    player::PlayerMovement,
};

#[derive(Component)]
struct InstructionText;

#[derive(Resource)]
struct Fonts {
    fira_sans_bold: Handle<Font>,
}

fn load_fonts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    let fonts = Fonts {
        fira_sans_bold: asset_server.load("fonts/FiraSans-Bold.ttf"),
    };
    let mut untyped_fonts = vec![fonts.fira_sans_bold.clone_untyped()];
    loading.0.append(&mut untyped_fonts);
    commands.insert_resource(fonts);
}

fn show_instructions(mut commands: Commands, fonts: Res<Fonts>, config: Res<Config>) {
    let text_style = TextStyle {
        font: fonts.fira_sans_bold.clone(),
        font_size: 30.0,
        color: Color::WHITE,
    };
    commands.spawn((
        TextBundle::from_section(config.instructions.clone(), text_style)
            .with_text_alignment(TextAlignment::Left)
            .with_style(Style {
                position_type: PositionType::Absolute,
                left: Val::Px(10.0),
                top: Val::Px(10.0),
                ..Default::default()
            }),
        InstructionText,
        Name::new("Instructions"),
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
        app.add_systems(Startup, load_fonts)
            .add_systems(OnEnter(AppState::InGame), show_instructions)
            .add_systems(Update, hide_instructions.run_if(in_state(AppState::InGame)));
    }
}

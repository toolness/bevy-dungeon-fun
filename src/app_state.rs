use bevy::prelude::*;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum AppState {
    #[default]
    LoadingAssets,
    SettingUpScene,
    InGame,
}

#[derive(Resource, Default)]
pub struct AssetsLoading(pub Vec<HandleUntyped>);

pub fn start_game(mut next_state: ResMut<NextState<AppState>>) {
    info!("Starting game...");
    next_state.set(AppState::InGame);
}

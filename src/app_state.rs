use bevy::prelude::*;

#[derive(States, PartialEq, Eq, Debug, Clone, Hash, Default)]
pub enum AppState {
    #[default]
    LoadingAssets,
    SettingUpScene,
    InGame,
}

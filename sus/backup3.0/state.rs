use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Intro,
    Playing,
    Paused,
    GameOver,
    Win,
    Score, 
}

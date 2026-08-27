use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Controls,
    Story,
    Playing,
    Paused,
    GameOver,
    WinStory,
    Win,
    Score, 
}

#[derive(Resource)]
pub struct VolumeSettings {
    pub music: f32,
    pub sfx: f32,
}

impl Default for VolumeSettings {
    fn default() -> Self {
        Self { music: 0.5, sfx: 0.5 } // Default 50% volume
    }
}

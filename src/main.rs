use bevy::prelude::*;
use crate::state::VolumeSettings;

mod state;
mod screens;
mod ship;
mod laser;
mod star;
mod camera;
mod enemy;
mod crosshair;
mod ui;

#[derive(Component)]
pub struct BackgroundMusic;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.06)))
        .init_resource::<VolumeSettings>()
        .add_plugins(DefaultPlugins)
        .init_state::<state::GameState>() 
        .add_systems(Startup, setup_background_music)
        .add_systems(Update, update_music_volume)
        .add_plugins((
            camera::CameraPlugin,
            ship::ShipPlugin,
            laser::LaserPlugin,
            star::StarPlugin,
            enemy::EnemyPlugin,
            crosshair::CrosshairPlugin,
            ui::GameUiPlugin,
            screens::ScreensPlugin,
        ))
        .run();
}

fn setup_background_music(mut commands: Commands, asset_server: Res<AssetServer>, volume: Res<VolumeSettings>) {
    commands.spawn((
        AudioPlayer(asset_server.load::<AudioSource>("music/MesmerizingGalaxyLoop.mp3")),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(volume.music)),
        BackgroundMusic,
    ));
}

fn update_music_volume(
    volume: Res<VolumeSettings>,
    mut sink_query: Query<&mut bevy::audio::AudioSink, With<BackgroundMusic>> // Changed to &mut AudioSink
) {
    if volume.is_changed() {
        for mut sink in &mut sink_query { // Iterated mutably
            sink.set_volume(bevy::audio::Volume::Linear(volume.music));
        }
    }
}

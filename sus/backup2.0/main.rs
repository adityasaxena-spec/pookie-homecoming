use bevy::prelude::*;

mod state;
mod screens;
mod ship;
mod laser;
mod star;
mod camera;
mod enemy;
mod crosshair;
mod ui;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.06)))
        .add_plugins(DefaultPlugins)
        .init_state::<state::GameState>() // MOVED HERE: Must come after DefaultPlugins
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

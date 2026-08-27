use bevy::prelude::*;

mod ship;
mod laser;
mod star;
mod camera;
mod enemy;
mod crosshair;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.06)))
        .add_plugins(DefaultPlugins)
        .add_plugins((
            camera::CameraPlugin,
            ship::ShipPlugin,
            laser::LaserPlugin,
            star::StarPlugin,
            enemy::EnemyPlugin,
            crosshair::CrosshairPlugin,
        ))
        .run();
}

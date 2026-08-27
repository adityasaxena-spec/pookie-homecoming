use bevy::prelude::*;
use crate::ship::Ship;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
           .add_systems(Update, camera_follow);
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn camera_follow(
    ship_query: Query<&Ship>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Ship>)>,
    time: Res<Time>,
) {
    let ship = ship_query.single().unwrap();
    let mut camera_transform = camera_query.single_mut().unwrap();
    let target = ship.position.extend(camera_transform.translation.z);
    camera_transform.translation = camera_transform.translation.lerp(target, 4.0 * time.delta_secs());
}

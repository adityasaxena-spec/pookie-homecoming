use bevy::prelude::*;
use bevy::window::CursorOptions;

use crate::camera::setup_camera;

pub struct CrosshairPlugin;

impl Plugin for CrosshairPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                setup_crosshair.after(setup_camera),
                hide_cursor,
            ),
        )
        .add_systems(Update, move_crosshair);
    }
}

#[derive(Component)]
struct Crosshair;

fn setup_crosshair(mut commands: Commands) {
    // Horizontal line
    commands.spawn((
        Sprite::from_color(
            Color::srgb(1.0, 0.2, 0.2),
            Vec2::new(16.0, 3.0),
        ),
        Transform::from_xyz(0.0, 0.0, 10.0),
        Crosshair,
    ));

    // Vertical line
    commands.spawn((
        Sprite::from_color(
            Color::srgb(1.0, 0.2, 0.2),
            Vec2::new(3.0, 16.0),
        ),
        Transform::from_xyz(0.0, 0.0, 10.0),
        Crosshair,
    ));
}

fn hide_cursor(
    mut cursor_options: Query<&mut CursorOptions>,
) {
    let mut cursor = cursor_options.single_mut().unwrap();
    cursor.visible = false;
}

fn move_crosshair(
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut crosshair_query: Query<&mut Transform, With<Crosshair>>,
) {
    let window = window_query.single().unwrap();
    let (camera, camera_transform) = camera_query.single().unwrap();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) =
            camera.viewport_to_world_2d(camera_transform, cursor_pos)
        {
            for mut transform in &mut crosshair_query {
                transform.translation.x = world_pos.x;
                transform.translation.y = world_pos.y;
            }
        }
    }
}

use bevy::prelude::*;

pub struct StarPlugin;

impl Plugin for StarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_stars)
           .add_systems(Update, move_stars);
    }
}

#[derive(Component)]
struct Star {
    seed: Vec2,
}

#[derive(Resource)]
struct StarFieldConfig {
    size: f32,
}

fn spawn_stars(mut commands: Commands, window_query: Query<&Window>) {
    let window = window_query.single().unwrap();
    let field_size = window.width().max(window.height()) * 2.5;
    let density = 200.0 / (1600.0 * 1600.0);
    let star_count = (density * field_size * field_size) as u32;

    commands.insert_resource(StarFieldConfig { size: field_size });

    let mut rng_state: u32 = 12345;
    for _ in 0..star_count {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let x = (rng_state % 10000) as f32 / 10000.0 * field_size;
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let y = (rng_state % 10000) as f32 / 10000.0 * field_size;

        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 1.0, 1.0), Vec2::new(2.0, 2.0)),
            Transform::from_xyz(0.0, 0.0, -1.0),
            Star { seed: Vec2::new(x, y) },
        ));
    }
}

fn move_stars(
    field_config: Res<StarFieldConfig>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<Star>)>,
    mut star_query: Query<(&Star, &mut Transform), Without<Camera2d>>,
) {
    let camera_pos = camera_query.single().unwrap().translation;
    let field_size = field_config.size;

    for (star, mut transform) in &mut star_query {
        let rel_x = (star.seed.x - camera_pos.x).rem_euclid(field_size) - field_size / 2.0;
        let rel_y = (star.seed.y - camera_pos.y).rem_euclid(field_size) - field_size / 2.0;
        transform.translation = Vec3::new(camera_pos.x + rel_x, camera_pos.y + rel_y, -1.0);
    }
}

use bevy::prelude::*;
use crate::ship::Ship;

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_laser_sound)
           .add_systems(Update, (shoot, move_lasers));
    }
}

const LASER_SPEED: f32 = 500.0;

#[derive(Component)]
pub struct Laser {
    pub direction: Vec2,
}

#[derive(Resource)]
struct LaserSound(Handle<AudioSource>);

fn load_laser_sound(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LaserSound(asset_server.load("laser_pew.wav")));
}

fn shoot(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    laser_sound: Res<LaserSound>,
    query: Query<&Ship>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let ship = query.single().unwrap();
        let local_offset = Vec2::new(0.0, 20.0);
        let rotation = Mat2::from_angle(ship.facing);
        let spawn_pos = ship.position + rotation * local_offset;
        let fire_direction = rotation * Vec2::new(0.0, 1.0);

        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 1.0, 0.0), Vec2::new(4.0, 16.0)),
            Transform::from_translation(spawn_pos.extend(0.0))
                .with_rotation(Quat::from_rotation_z(ship.facing)),
            Laser { direction: fire_direction },
        ));

        commands.spawn((
           AudioPlayer::new(laser_sound.0.clone()),
           PlaybackSettings::DESPAWN,
        ));
    }
}

fn move_lasers(
    mut commands: Commands,
    time: Res<Time>,
    window_query: Query<&Window>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<Laser>)>,
    mut laser_query: Query<(Entity, &Laser, &mut Transform), Without<Camera2d>>,
) {
    let window = window_query.single().unwrap();
    let camera_pos = camera_query.single().unwrap().translation;
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    let margin = 50.0;

    for (entity, laser, mut transform) in &mut laser_query {
        transform.translation += (laser.direction * LASER_SPEED * time.delta_secs()).extend(0.0);
        let rel = transform.translation - camera_pos;
        if rel.x.abs() > half_width + margin || rel.y.abs() > half_height + margin {
            commands.entity(entity).despawn();
        }
    }
}

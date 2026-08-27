use bevy::prelude::*;
use rand::RngExt; 
use crate::ship::{Ship, ShipStability, PlayerStats}; 
use crate::state::{GameState, VolumeSettings};

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LaserCooldown(Timer::from_seconds(0.2, TimerMode::Once)))
           .add_systems(Startup, load_laser_sound)
           .add_systems(Update, (shoot, move_lasers).run_if(in_state(GameState::Playing)))
           .add_systems(OnExit(GameState::GameOver), reset_lasers)
           .add_systems(OnExit(GameState::Score), reset_lasers);
    }
}

const LASER_SPEED: f32 = 500.0;

#[derive(Component)]
pub struct Laser {
    pub direction: Vec2,
}

#[derive(Resource)]
struct LaserSound(Handle<AudioSource>);

#[derive(Resource)]
struct LaserCooldown(Timer);

fn reset_lasers(mut commands: Commands, laser_query: Query<Entity, With<Laser>>) {
    for entity in &laser_query {
        commands.entity(entity).despawn();
    }
}

fn load_laser_sound(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LaserSound(asset_server.load("laser_pew.wav")));
}

fn shoot(
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    laser_sound: Res<LaserSound>,
    query: Query<&Ship>,
    stability: Res<ShipStability>,
    mut stats: ResMut<PlayerStats>,
    mut cooldown: ResMut<LaserCooldown>,
    time: Res<Time>,
    volume: Res<VolumeSettings>,
) {
    cooldown.0.tick(time.delta());

    if mouse.pressed(MouseButton::Left) && cooldown.0.is_finished() {
        let ship = query.single().unwrap();
        let mut rng = rand::rng();

        stats.lasers_fired += 1;

        if stability.is_yellow() {
            let next_cooldown = rng.random_range(0.05..1.2); 
            cooldown.0.set_duration(std::time::Duration::from_secs_f32(next_cooldown));
        } else {
            cooldown.0.set_duration(std::time::Duration::from_secs_f32(0.2));
        }
        cooldown.0.reset();

        let local_offset = Vec2::new(0.0, 20.0);
        let rotation = Mat2::from_angle(ship.facing);
        let spawn_pos = ship.position + rotation * local_offset;
        
        let mut fire_direction = rotation * Vec2::new(0.0, 1.0);

        if stability.is_orange() {
            let offset_angle = rng.random_range(-0.3..0.3);
            fire_direction = Mat2::from_angle(offset_angle) * fire_direction;
        }

        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 1.0, 0.0), Vec2::new(4.0, 16.0)),
            Transform::from_translation(spawn_pos.extend(0.0))
                .with_rotation(Quat::from_rotation_z(fire_direction.y.atan2(fire_direction.x) - std::f32::consts::FRAC_PI_2)),
            Laser { direction: fire_direction },
        ));

        commands.spawn((
           AudioPlayer::new(laser_sound.0.clone()),
           PlaybackSettings::DESPAWN.with_volume(bevy::audio::Volume::Linear(volume.sfx)),
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

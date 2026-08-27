use bevy::prelude::*;
use rand::RngExt;
use crate::ship::Ship;
use crate::laser::Laser;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_systems(
                Update,
                (
                    spawn_enemies,
                    move_enemies,
                    enemy_shoot,
                    move_enemy_lasers,
                    laser_hits_enemy,
                    enemy_laser_hits_player,
                    enemy_collides_with_player,
                    animate_explosion,
                    fade_damage_numbers,
                ),
            );
    }
}

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

#[derive(Component)]
pub struct Enemy {
    hp: f32,
    shoot_timer: Timer,
}

#[derive(Component)]
struct EnemyLaser {
    direction: Vec2,
}

#[derive(Component)]
struct DamageNumber {
    velocity: Vec2,
    timer: Timer,
}

const ENEMY_MAX_HP: f32 = 100.0;
const ENEMY_SPEED: f32 = 80.0;
const ENEMY_LASER_SPEED: f32 = 220.0;
const ENEMY_LASER_DAMAGE: f32 = 10.0;
const LASER_BASE_DAMAGE: f32 = 20.0;
const LASER_CRIT_DAMAGE: f32 = 25.0;
const CRIT_CHANCE: f32 = 0.10;
const HIT_RADIUS: f32 = 24.0;
const PLAYER_HIT_RADIUS: f32 = 22.0;

fn spawn_enemies(
    time: Res<Time>,
    mut timer: ResMut<EnemySpawnTimer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let window = window_query.single().unwrap();
    let camera_pos = camera_query.single().unwrap().translation.truncate();

    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;
    let spawn_radius = (half_w * half_w + half_h * half_h).sqrt() + 80.0; // just outside the visible corner

    let mut rng = rand::rng();
    let angle = rng.random_range(0.0..std::f32::consts::TAU);
    let spawn_pos = camera_pos + Vec2::new(angle.cos(), angle.sin()) * spawn_radius;

    commands.spawn((
        Mesh2d(meshes.add(Triangle2d::new(
            Vec2::new(0.0, 18.0),
            Vec2::new(-14.0, -14.0),
            Vec2::new(14.0, -14.0),
        ))),
        MeshMaterial2d(materials.add(Color::srgb(0.9, 0.15, 0.15))),
        Transform::from_translation(spawn_pos.extend(1.0)),
        Enemy {
            hp: ENEMY_MAX_HP,
            shoot_timer: Timer::from_seconds(rng.random_range(1.2..2.2), TimerMode::Repeating),
        },
    ));
}

fn move_enemies(
    time: Res<Time>,
    ship_query: Query<&Ship>,
    mut enemy_query: Query<&mut Transform, With<Enemy>>,
) {
    let ship = ship_query.single().unwrap();
    let dt = time.delta_secs();

    for mut transform in &mut enemy_query {
        let pos = transform.translation.truncate();
        let to_player = (ship.position - pos).normalize_or_zero();
        transform.translation += (to_player * ENEMY_SPEED * dt).extend(0.0);

        let angle = to_player.y.atan2(to_player.x) - std::f32::consts::FRAC_PI_2;
        transform.rotation = Quat::from_rotation_z(angle);
    }
}

fn enemy_shoot(
    time: Res<Time>,
    mut commands: Commands,
    ship_query: Query<&Ship>,
    mut enemy_query: Query<(&Transform, &mut Enemy)>,
) {
    let ship = ship_query.single().unwrap();

    for (transform, mut enemy) in &mut enemy_query {
        if !enemy.shoot_timer.tick(time.delta()).just_finished() {
            continue;
        }
        let pos = transform.translation.truncate();
        let direction = (ship.position - pos).normalize_or_zero();

        commands.spawn((
            Sprite::from_color(Color::srgb(1.0, 0.3, 0.3), Vec2::new(4.0, 14.0)),
            Transform::from_translation(pos.extend(0.5))
                .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2)),
            EnemyLaser { direction },
        ));
    }
}

fn move_enemy_lasers(
    mut commands: Commands,
    time: Res<Time>,
    window_query: Query<&Window>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<EnemyLaser>)>,
    mut laser_query: Query<(Entity, &EnemyLaser, &mut Transform), Without<Camera2d>>,
) {
    let window = window_query.single().unwrap();
    let camera_pos = camera_query.single().unwrap().translation;
    let half_width = window.width() / 2.0;
    let half_height = window.height() / 2.0;
    let margin = 50.0;

    for (entity, laser, mut transform) in &mut laser_query {
        transform.translation += (laser.direction * ENEMY_LASER_SPEED * time.delta_secs()).extend(0.0);
        let rel = transform.translation - camera_pos;
        if rel.x.abs() > half_width + margin || rel.y.abs() > half_height + margin {
            commands.entity(entity).despawn();
        }
    }
}

fn laser_hits_enemy(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    laser_query: Query<(Entity, &Transform), With<Laser>>,
    mut enemy_query: Query<(Entity, &Transform, &mut Enemy)>,
) {
    let mut rng = rand::rng();

    for (laser_entity, laser_transform) in &laser_query {
        let laser_pos = laser_transform.translation.truncate();

        for (enemy_entity, enemy_transform, mut enemy) in &mut enemy_query {
            let enemy_pos = enemy_transform.translation.truncate();
            if laser_pos.distance(enemy_pos) > HIT_RADIUS {
                continue;
            }

            let is_crit = rng.random_bool(CRIT_CHANCE as f64);
            let damage = if is_crit { LASER_CRIT_DAMAGE } else { LASER_BASE_DAMAGE };

            enemy.hp -= damage;
            commands.entity(laser_entity).despawn();

            commands.spawn((
                Text2d::new(format!("{}", damage as i32)),
                TextFont { font_size: px(20.0).into(), ..default() },
                TextColor(Color::srgb(1.0, 0.9, 0.1)),
                Transform::from_translation(enemy_pos.extend(2.0)),
                DamageNumber {
                    velocity: Vec2::new(rng.random_range(-20.0..20.0), 60.0),
                    timer: Timer::from_seconds(0.8, TimerMode::Once),
                },
            ));

            if enemy.hp <= 0.0 {
                commands.entity(enemy_entity).despawn();
                spawn_explosion(&mut commands, &mut meshes, &mut materials, &asset_server, enemy_pos);
            }

            break;
        }
    }
}

fn enemy_laser_hits_player(
    mut commands: Commands,
    ship_query: Query<&Ship>,
    laser_query: Query<(Entity, &Transform), With<EnemyLaser>>,
) {
    let ship = ship_query.single().unwrap();

    for (entity, transform) in &laser_query {
        if transform.translation.truncate().distance(ship.position) < PLAYER_HIT_RADIUS {
            commands.entity(entity).despawn();
            // TODO: hook into player Stability/HP resource once that system exists (planned TODO day 3)
            eprintln!("Player hit for {} damage (placeholder — no HP system yet)", ENEMY_LASER_DAMAGE);
        }
    }
}

fn fade_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut TextColor, &mut DamageNumber)>,
) {
    for (entity, mut transform, mut color, mut dmg) in &mut query {
        dmg.timer.tick(time.delta());
        transform.translation += (dmg.velocity * time.delta_secs()).extend(0.0);

        let alpha = 1.0 - dmg.timer.fraction();
        color.0.set_alpha(alpha);

        if dmg.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

const ENEMY_COLLISION_DAMAGE: f32 = 20.0;

#[derive(Component)]
struct Explosion {
    timer: Timer,
}

fn enemy_collides_with_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    ship_query: Query<&Ship>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    let ship = ship_query.single().unwrap();

    for (entity, transform) in &enemy_query {
        let enemy_pos = transform.translation.truncate();
        if enemy_pos.distance(ship.position) < PLAYER_HIT_RADIUS + 10.0 {
            commands.entity(entity).despawn();

            // TODO: hook into player Stability/HP resource once that system exists (Day 3)
            eprintln!("Player hit by collision for {} damage (placeholder — no HP system yet)", ENEMY_COLLISION_DAMAGE);

            spawn_explosion(&mut commands, &mut meshes, &mut materials, &asset_server, enemy_pos);
        }
    }
}

fn spawn_explosion(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>,
    pos: Vec2,
) {

    commands.spawn((
        AudioPlayer::new(asset_server.load("explosion_dush.wav")),
        PlaybackSettings::DESPAWN,
    ));

    // a handful of small triangles flung outward, shrinking and fading — cheap "explosion" with no image assets
    let mut rng = rand::rng();
    for _ in 0..8 {
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(80.0..220.0);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;

        commands.spawn((
            Mesh2d(meshes.add(Triangle2d::new(
                Vec2::new(0.0, 5.0),
                Vec2::new(-4.0, -4.0),
                Vec2::new(4.0, -4.0),
            ))),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0.6, 0.1))),
            Transform::from_translation(pos.extend(1.5)),
            Explosion {
                timer: Timer::from_seconds(0.5, TimerMode::Once),
            },
            ExplosionPiece { velocity },
        ));
    }
}

#[derive(Component)]
struct ExplosionPiece {
    velocity: Vec2,
}

fn animate_explosion(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut MeshMaterial2d<ColorMaterial>, &ExplosionPiece, &mut Explosion)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (entity, mut transform, material_handle, piece, mut explosion) in &mut query {
        explosion.timer.tick(time.delta());
        transform.translation += (piece.velocity * time.delta_secs()).extend(0.0);

        let progress = explosion.timer.fraction();
        transform.scale = Vec3::splat(1.0 - progress);

        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            material.color.set_alpha(1.0 - progress);
        }

        if explosion.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

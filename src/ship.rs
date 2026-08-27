use bevy::prelude::*;
use crate::state::GameState;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShipStability {
            current: 100.0,
            max: 100.0,
            regen_timer: Timer::from_seconds(1.5, TimerMode::Once),
        })
        .insert_resource(GameModeSettings { is_endless: false })
        .insert_resource(PlayerStats::default())
        .add_systems(Startup, setup_ship)
        .add_systems(Update, (move_ship, regen_stability).run_if(in_state(GameState::Playing)))
        // Reset systems attached to exiting the end screens
        .add_systems(OnExit(GameState::GameOver), reset_ship)
        .add_systems(OnExit(GameState::Score), reset_ship);
    }
}

#[derive(Resource)]
pub struct GameModeSettings {
    pub is_endless: bool,
}

#[derive(Resource, Default)]
pub struct PlayerStats {
    pub total_damage: f32,
    pub lasers_fired: u32,
    pub lasers_hit: u32,
}

#[derive(Resource)]
pub struct ShipStability {
    pub current: f32,
    pub max: f32,
    pub regen_timer: Timer,
}

impl ShipStability {
    pub fn is_yellow(&self) -> bool { self.current <= 75.0 }
    pub fn is_orange(&self) -> bool { self.current <= 50.0 }
    pub fn is_red(&self) -> bool { self.current <= 25.0 }
}

fn regen_stability(time: Res<Time>, mut stability: ResMut<ShipStability>) {
    stability.regen_timer.tick(time.delta());
    if stability.regen_timer.is_finished() && stability.current < stability.max {
        stability.current = (stability.current + 10.0 * time.delta_secs()).min(stability.max);
    }
}

#[derive(Component)]
pub struct Ship {
    pub position: Vec2,
    pub velocity: Vec2,
    pub facing: f32,
}

fn setup_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Mesh2d(meshes.add(Triangle2d::new(
            Vec2::new(0.0, 20.0),
            Vec2::new(-16.0, -16.0),
            Vec2::new(16.0, -16.0),
        ))),
        MeshMaterial2d(materials.add(Color::srgb(0.3, 0.7, 1.0))),
        Transform::from_xyz(0.0, -200.0, 1.0),
        Ship {
            position: Vec2::new(0.0, -200.0),
            velocity: Vec2::ZERO,
            facing: 0.0,
        },
    ));
}

fn reset_ship(
    mut stability: ResMut<ShipStability>,
    mut stats: ResMut<PlayerStats>,
    mut game_mode: ResMut<GameModeSettings>,
    mut ship_query: Query<&mut Ship>,
) {
    stability.current = stability.max;
    *stats = PlayerStats::default();
    game_mode.is_endless = false;
    
    if let Ok(mut ship) = ship_query.single_mut() {
        ship.position = Vec2::new(0.0, -200.0);
        ship.velocity = Vec2::ZERO;
        ship.facing = 0.0;
    }
}

const SHIP_ACCEL: f32 = 250.0;
const SHIP_DRAG: f32 = 1.2;
const SHIP_MAX_SPEED: f32 = 300.0;

fn move_ship(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    stability: Res<ShipStability>,
    game_mode: Res<GameModeSettings>,
    mut query: Query<(&mut Ship, &mut Transform)>,
) {
    let (mut ship, mut transform) = query.single_mut().unwrap();
    let dt = time.delta_secs();

    let mut input = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyA) { input.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { input.x += 1.0; }
    if keys.pressed(KeyCode::KeyW) { input.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) { input.y -= 1.0; }
    
    if stability.is_red() {
        input *= -1.0; 
    }
    
    let input = input.normalize_or_zero();

    if input != Vec2::ZERO {
        ship.velocity += input * SHIP_ACCEL * dt;
        ship.velocity = ship.velocity.clamp_length_max(SHIP_MAX_SPEED);
    } else {
        ship.velocity *= (1.0 - SHIP_DRAG * dt).max(0.0);
    }

    let velocity = ship.velocity;
    ship.position += velocity * dt;

    let window = window_query.single().unwrap();
    let (camera, camera_transform) = camera_query.single().unwrap();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            let to_cursor = world_pos - ship.position;
            if to_cursor.length_squared() > 1.0 {
                let target_angle = to_cursor.y.atan2(to_cursor.x) - std::f32::consts::FRAC_PI_2;
                ship.facing = lerp_angle(ship.facing, target_angle, 20.0 * dt);
            }
        }
    }

    let (idle_bob, thrust_wiggle) = if game_mode.is_endless {
        (Vec2::ZERO, Vec2::ZERO)
    } else {
        let speed_ratio = (ship.velocity.length() / SHIP_MAX_SPEED).clamp(0.0, 1.0);
        let t = time.elapsed_secs();
        let bob = Vec2::new((t * 1.3).sin(), (t * 1.7).sin()) * 3.0;
        let wiggle = Vec2::new((t * 25.0).sin(), (t * 31.0).sin()) * speed_ratio * 4.0;
        (bob, wiggle)
    };

    transform.translation = (ship.position + idle_bob + thrust_wiggle).extend(1.0);
    transform.rotation = Quat::from_rotation_z(ship.facing);
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let diff = (b - a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    a + diff * t.clamp(-1.0, 1.0)
}

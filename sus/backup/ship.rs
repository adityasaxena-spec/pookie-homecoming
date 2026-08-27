use bevy::prelude::*;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ship)
           .add_systems(Update, move_ship);
    }
}

//const SHIP_SPEED: f32 = 300.0;

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

const SHIP_ACCEL: f32 = 250.0;   // how slowly it builds up speed — lower = more sluggish
const SHIP_DRAG: f32 = 1.2;      // how slowly it coasts to a stop when you let go — lower = longer drift
const SHIP_MAX_SPEED: f32 = 300.0;

fn move_ship(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut query: Query<(&mut Ship, &mut Transform)>,
) {
    let (mut ship, mut transform) = query.single_mut().unwrap();
    let dt = time.delta_secs();

    let mut input = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyA) { input.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { input.x += 1.0; }
    if keys.pressed(KeyCode::KeyW) { input.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) { input.y -= 1.0; }
    let input = input.normalize_or_zero();

    if input != Vec2::ZERO {
        ship.velocity += input * SHIP_ACCEL * dt;
        ship.velocity = ship.velocity.clamp_length_max(SHIP_MAX_SPEED);
    } else {
        ship.velocity *= (1.0 - SHIP_DRAG * dt).max(0.0);
    }

    let velocity = ship.velocity;
    ship.position += velocity * dt;

    // facing now comes from the mouse cursor, independent of movement
    let window = window_query.single().unwrap();
    let (camera, camera_transform) = camera_query.single().unwrap();

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            let to_cursor = world_pos - ship.position;
            if to_cursor.length_squared() > 1.0 {
                let target_angle = to_cursor.y.atan2(to_cursor.x) - std::f32::consts::FRAC_PI_2;
                ship.facing = lerp_angle(ship.facing, target_angle, 20.0 * dt); // higher = snappier aim response
            }
        }
    }

    let speed_ratio = (ship.velocity.length() / SHIP_MAX_SPEED).clamp(0.0, 1.0);
    let t = time.elapsed_secs();
    let idle_bob = Vec2::new((t * 1.3).sin(), (t * 1.7).sin()) * 3.0;
    let thrust_wiggle = Vec2::new((t * 25.0).sin(), (t * 31.0).sin()) * speed_ratio * 4.0;

    transform.translation = (ship.position + idle_bob + thrust_wiggle).extend(1.0);
    transform.rotation = Quat::from_rotation_z(ship.facing);
}

fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let diff = (b - a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
    a + diff * t.clamp(-1.0, 1.0)
}



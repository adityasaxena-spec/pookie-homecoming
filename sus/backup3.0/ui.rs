use bevy::prelude::*;
use crate::ship::{Ship, ShipStability, GameModeSettings};
use crate::enemy::WaveManager;
use crate::state::GameState;
use rand::RngExt;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Checkpoints {
            locations: vec![
                Vec2::new(1500.0, 1500.0),
                Vec2::new(3000.0, -1000.0),
                Vec2::new(-2000.0, 2000.0),
            ],
        })
        .add_systems(Startup, setup_ui)
        .add_systems(Update, (
            update_stability_bar,
            update_wave_nodes,
            update_compass,
            check_checkpoints,
            update_checkpoint_popup,
        ).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource)]
pub struct Checkpoints {
    pub locations: Vec<Vec2>,
}

#[derive(Component)]
struct StabilityBarUI;

#[derive(Component)]
struct WaveNodeUI(u32);

#[derive(Component)]
struct CompassMarker;

#[derive(Component)]
struct CompassDistanceText;

#[derive(Component)]
struct CompassLabel(f32);

#[derive(Component)]
struct CheckpointPopup(Timer);

fn setup_ui(mut commands: Commands) {
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    }).with_children(|parent| {
        // --- COMPASS BAR (TOP CENTER) ---
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(25.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-150.0)),
                width: Val::Px(300.0),
                height: Val::Px(60.0), // Increased to prevent clipping
                overflow: Overflow::clip(),
                ..default()
            },
        )).with_children(|compass| {
            // Horizontal baseline line
            compass.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(22.0), // Shifted down
                    width: Val::Percent(100.0),
                    height: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // Directional tick marks and N, E, S, W text labels
            let directions = [("N", 0.0), ("E", 90.0), ("S", 180.0), ("W", 270.0)];
            for (name, deg) in directions {
                // Vertical tick mark on the line
                compass.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(18.0), // Shifted down
                        width: Val::Px(2.0),
                        height: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.8, 0.8, 0.8)),
                    CompassLabel(deg),
                ));

                // Letter label underneath the tick mark
                compass.spawn((
                    Text::new(name),
                    TextFont { font_size: bevy::text::FontSize::Px(12.0), ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(28.0), // Shifted down
                        ..default()
                    },
                    CompassLabel(deg),
                ));
            }

            // Target / Event Trigger Marker (dot + distance text above it)
            compass.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(19.0), // Shifted down
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(1.0, 0.9, 0.4)),
                CompassMarker,
            ));

            compass.spawn((
                Text::new("15m"),
                TextFont { font_size: bevy::text::FontSize::Px(12.0), ..default() },
                TextColor(Color::srgb(1.0, 0.9, 0.4)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(4.0), // Shifted down to fit within the box
                    ..default()
                },
                CompassDistanceText,
            ));
        });

        // --- STABILITY BAR (BOTTOM LEFT) ---
        parent.spawn((
            Text::new("heat bar / hp bar"),
            TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() },
            Node { position_type: PositionType::Absolute, bottom: Val::Px(45.0), left: Val::Px(20.0), ..default() }
        ));

        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(20.0),
                width: Val::Px(200.0),
                height: Val::Px(20.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::WHITE),
        )).with_children(|bg| {
            bg.spawn((
                Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
                BackgroundColor(Color::srgb(0.2, 0.9, 0.2)),
                StabilityBarUI,
            ));
        });

        // --- WAVE PROGRESS NODES (RIGHT SIDE) ---
        parent.spawn((
            Text::new("event line"),
            TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() },
            Node { position_type: PositionType::Absolute, right: Val::Px(40.0), top: Val::Percent(25.0), ..default() }
        ));

        parent.spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(60.0),
            top: Val::Percent(30.0),
            height: Val::Percent(40.0),
            width: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        }).with_children(|line_container| {
            line_container.spawn((
                Node { position_type: PositionType::Absolute, width: Val::Px(2.0), height: Val::Percent(100.0), ..default() },
                BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
            ));

            for i in 1..=3 {
                line_container.spawn((
                    Node { width: Val::Px(14.0), height: Val::Px(14.0), ..default() },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    WaveNodeUI(i),
                ));
            }
        });
    });
}

fn update_stability_bar(
    stability: Res<ShipStability>,
    mut query: Query<(&mut Node, &mut BackgroundColor), With<StabilityBarUI>>,
) {
    for (mut node, mut color) in &mut query {
        let pct = (stability.current / stability.max).clamp(0.0, 1.0) * 100.0;
        node.width = Val::Percent(pct);
        color.0 = if stability.is_red() {
            Color::srgb(0.9, 0.2, 0.2)
        } else if stability.is_orange() {
            Color::srgb(1.0, 0.5, 0.1)
        } else if stability.is_yellow() {
            Color::srgb(1.0, 0.9, 0.1)
        } else {
            Color::srgb(0.2, 0.9, 0.2)
        };
    }
}

fn update_wave_nodes(
    wave: Res<WaveManager>,
    mut query: Query<(&WaveNodeUI, &mut BackgroundColor)>,
) {
    for (node, mut color) in &mut query {
        let target_node = if node.0 > 3 { 3 } else { node.0 };
        if wave.current_wave > target_node {
            color.0 = Color::srgb(0.2, 0.9, 0.2);
        } else if wave.current_wave == target_node && wave.is_active {
            color.0 = Color::srgb(1.0, 0.2, 0.2);
        } else if wave.current_wave == target_node {
            color.0 = Color::srgb(1.0, 0.9, 0.1);
        } else {
            color.0 = Color::srgb(0.3, 0.3, 0.3);
        }
    }
}

fn angle_diff(target: f32, current: f32) -> f32 {
    // Fixed: rem_euclid correctly wraps negative angle drifts in endless mode
    (target - current + 180.0).rem_euclid(360.0) - 180.0
}

fn update_compass(
    ship_query: Query<&Ship>,
    checkpoints: Res<Checkpoints>,
    wave: Res<WaveManager>,
    mut marker_query: Query<(&mut Node, &mut Visibility), (With<CompassMarker>, Without<CompassDistanceText>, Without<CompassLabel>)>,
    mut text_query: Query<(&mut Node, &mut Text, &mut Visibility), (With<CompassDistanceText>, Without<CompassMarker>, Without<CompassLabel>)>,
    mut label_query: Query<(&mut Node, &mut Visibility, &CompassLabel), (Without<CompassMarker>, Without<CompassDistanceText>)>,
) {
    let Some(ship) = ship_query.iter().next() else { return };
    let heading = ship.facing.to_degrees();
    
    for (mut node, mut vis, label) in &mut label_query {
        let diff = angle_diff(label.0, heading);
        let pct = 50.0 - (diff / 90.0) * 50.0;
        if pct >= 0.0 && pct <= 100.0 {
            node.left = Val::Percent(pct);
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    let hide_marker = wave.is_active;
    let target_idx = (wave.current_wave.saturating_sub(1)) as usize;
    let target_pos = checkpoints.locations.get(target_idx).copied().unwrap_or(Vec2::new(1000.0, 1000.0));

    for (mut node, mut vis) in &mut marker_query {
        *vis = if hide_marker { Visibility::Hidden } else { Visibility::Visible };
        if !hide_marker {
            let target_vec = target_pos - ship.position;
            let target_angle = target_vec.y.atan2(target_vec.x) - std::f32::consts::FRAC_PI_2;
            let diff = angle_diff(target_angle.to_degrees(), heading);
            node.left = Val::Percent((50.0 - (diff / 90.0) * 50.0).clamp(0.0, 100.0));
        }
    }

    for (mut node, mut text, mut vis) in &mut text_query {
        *vis = if hide_marker { Visibility::Hidden } else { Visibility::Visible };
        if !hide_marker {
            let target_vec = target_pos - ship.position;
            let distance = target_vec.length();
            let target_angle = target_vec.y.atan2(target_vec.x) - std::f32::consts::FRAC_PI_2;
            let diff = angle_diff(target_angle.to_degrees(), heading);
            
            node.left = Val::Percent((50.0 - (diff / 90.0) * 50.0).clamp(0.0, 100.0));
            text.0 = format!("{:.0}m", distance);
        }
    }
}

fn check_checkpoints(
    mut commands: Commands,
    ship_query: Query<&Ship>,
    mut wave: ResMut<WaveManager>,
    mut checkpoints: ResMut<Checkpoints>,
    game_mode: Res<GameModeSettings>,
) {
    let Some(ship) = ship_query.iter().next() else { return };
    if wave.is_active { return; }

    let target_idx = (wave.current_wave - 1) as usize;
    
    if target_idx >= checkpoints.locations.len() {
        let mut rng = rand::rng();
        let random_offset = Vec2::new(rng.random_range(-4000.0..4000.0), rng.random_range(-4000.0..4000.0));
        checkpoints.locations.push(ship.position + random_offset);
    }

    if let Some(&target) = checkpoints.locations.get(target_idx) {
        if ship.position.distance(target) < 100.0 {
            if !game_mode.is_endless && wave.current_wave > wave.max_waves {
                return;
            }

            wave.is_active = true;

            commands.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                CheckpointPopup(Timer::from_seconds(5.0, TimerMode::Once)),
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(format!("WAVE {} TRIGGERED\nHostiles Incoming", wave.current_wave)),
                    TextFont { font_size: bevy::text::FontSize::Px(42.0), ..default() },
                    TextColor(Color::srgb(1.0, 0.8, 0.2)),
                    TextLayout::justify(Justify::Center),
                ));
            });
        }
    }
}

fn update_checkpoint_popup(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut CheckpointPopup, &Children)>,
    mut text_query: Query<&mut TextColor>,
) {
    for (entity, mut popup, children) in &mut query {
        popup.0.tick(time.delta());
        let alpha = 1.0 - popup.0.fraction();

        for child in children.iter() {
            if let Ok(mut color) = text_query.get_mut(child) {
                color.0.set_alpha(alpha);
            }
        }

        if popup.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

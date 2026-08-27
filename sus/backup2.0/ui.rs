use bevy::prelude::*;
use crate::ship::{Ship, ShipStability};
use crate::enemy::WaveManager;
use crate::state::GameState;

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

// Replaces EventTriggerTarget with a list of checkpoints for each wave
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

        // --- STABILITY BAR (Bottom Left) ---
        parent.spawn((
            Text::new("heat bar / hp bar"),
            TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() },
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(45.0),
                left: Val::Px(20.0),
                ..default()
            }
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
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.9, 0.2)),
                StabilityBarUI,
            ));
        });

        // --- EVENT LINE (Right Center) ---
        parent.spawn((
            Text::new("event line"),
            TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() },
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(40.0),
                top: Val::Percent(25.0),
                ..default()
            }
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
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(2.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
            ));

            for i in 1..=3 {
                line_container.spawn((
                    Node {
                        width: Val::Px(14.0),
                        height: Val::Px(14.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    WaveNodeUI(i),
                ));
            }
        });

        // --- COMPASS (Top Center) ---
        parent.spawn((
            Text::new("compass"),
            TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(50.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-25.0)),
                ..default()
            }
        ));

        parent.spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-150.0)),
            width: Val::Px(300.0),
            height: Val::Px(30.0),
            ..default()
        }).with_children(|compass| {
            compass.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(14.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ));

            let labels = [("N", 0.0), ("E", -90.0), ("S", 180.0), ("W", 90.0)];
            for (letter, angle) in labels {
                compass.spawn((
                    Text::new(letter),
                    TextFont { font_size: bevy::text::FontSize::Px(12.0), ..default() },
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(16.0),
                        margin: UiRect::left(Val::Px(-4.0)), 
                        ..default()
                    },
                    CompassLabel(angle),
                ));
            }

            compass.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    width: Val::Px(10.0),
                    height: Val::Px(10.0),
                    margin: UiRect::left(Val::Px(-5.0)), 
                    ..default()
                },
                BackgroundColor(Color::srgb(1.0, 0.8, 0.2)),
                CompassMarker,
            ));

            compass.spawn((
                Text::new(""),
                TextFont { font_size: bevy::text::FontSize::Px(14.0), ..default() },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(-10.0),
                    margin: UiRect::left(Val::Px(-15.0)), 
                    ..default()
                },
                CompassDistanceText,
            ));
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
        if wave.current_wave > node.0 {
            color.0 = Color::srgb(0.2, 0.9, 0.2); // Cleared
        } else if wave.current_wave == node.0 && wave.is_active {
            color.0 = Color::srgb(1.0, 0.2, 0.2); // Combat active (Red)
        } else if wave.current_wave == node.0 {
            color.0 = Color::srgb(1.0, 0.9, 0.1); // Navigating to (Yellow)
        } else {
            color.0 = Color::srgb(0.3, 0.3, 0.3); // Future (Gray)
        }
    }
}

// Finds the shortest path mapping between two angles (-180 to 180)
fn angle_diff(target: f32, current: f32) -> f32 {
    (target - current + 540.0) % 360.0 - 180.0
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
    
    // Mapping heading: 0 is North, 90 is West, -90 is East, 180/-180 is South
    let heading = ship.facing.to_degrees();
    
    // 1. Move the N, E, S, W labels based on ship heading
    for (mut node, mut vis, label) in &mut label_query {
        let diff = angle_diff(label.0, heading);
        // Field of view of 180 degrees (-90 to 90 degrees visible at once)
        let pct = 50.0 - (diff / 90.0) * 50.0;
        
        if pct >= 0.0 && pct <= 100.0 {
            node.left = Val::Percent(pct);
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }

    // Hide target marker if a wave is currently being fought, or if you win
    let hide_marker = wave.is_active || wave.current_wave > wave.max_waves;
    
    for (mut node, mut vis) in &mut marker_query {
        *vis = if hide_marker { Visibility::Hidden } else { Visibility::Visible };
        if !hide_marker {
            if let Some(&target_pos) = checkpoints.locations.get((wave.current_wave.saturating_sub(1)) as usize) {
                let target_vec = target_pos - ship.position;
                let target_angle = target_vec.y.atan2(target_vec.x) - std::f32::consts::FRAC_PI_2;
                let diff = angle_diff(target_angle.to_degrees(), heading);
                
                // Clamp to edge of compass if out of immediate view
                node.left = Val::Percent((50.0 - (diff / 90.0) * 50.0).clamp(0.0, 100.0));
            }
        }
    }

    for (mut node, mut text, mut vis) in &mut text_query {
        *vis = if hide_marker { Visibility::Hidden } else { Visibility::Visible };
        if !hide_marker {
            if let Some(&target_pos) = checkpoints.locations.get((wave.current_wave.saturating_sub(1)) as usize) {
                let target_vec = target_pos - ship.position;
                let distance = target_vec.length();
                let target_angle = target_vec.y.atan2(target_vec.x) - std::f32::consts::FRAC_PI_2;
                let diff = angle_diff(target_angle.to_degrees(), heading);
                
                node.left = Val::Percent((50.0 - (diff / 90.0) * 50.0).clamp(0.0, 100.0));
                text.0 = format!("{:.0}m", distance);
            }
        }
    }
}

fn check_checkpoints(
    mut commands: Commands,
    ship_query: Query<&Ship>,
    mut wave: ResMut<WaveManager>,
    checkpoints: Res<Checkpoints>,
) {
    let Some(ship) = ship_query.iter().next() else { return };

    if wave.is_active || wave.current_wave > wave.max_waves {
        return;
    }

    if let Some(&target) = checkpoints.locations.get((wave.current_wave - 1) as usize) {
        if ship.position.distance(target) < 100.0 {
            // Trigger the wave and hide compass
            wave.is_active = true;

            // Spawn fadeout notification popup
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

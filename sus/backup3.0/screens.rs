use bevy::prelude::*;
use bevy::app::AppExit;
use crate::state::GameState;
use crate::ship::{ShipStability, PlayerStats, GameModeSettings};
use crate::enemy::WaveManager;

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (check_game_over, check_win).run_if(in_state(GameState::Playing)))
           .add_systems(Update, toggle_pause) 
           
           .add_systems(OnEnter(GameState::Intro), setup_intro)
           .add_systems(Update, start_game_on_space.run_if(in_state(GameState::Intro)))
           .add_systems(OnExit(GameState::Intro), cleanup_screen)
           
           .add_systems(OnEnter(GameState::Paused), setup_pause)
           .add_systems(OnExit(GameState::Paused), cleanup_screen)
           
           .add_systems(OnEnter(GameState::GameOver), setup_game_over)
           .add_systems(Update, exit_on_space.run_if(in_state(GameState::GameOver)))
           .add_systems(OnExit(GameState::GameOver), cleanup_screen)
           
           .add_systems(OnEnter(GameState::Win), setup_win)
           .add_systems(Update, handle_win_inputs.run_if(in_state(GameState::Win)))
           .add_systems(OnExit(GameState::Win), cleanup_screen)

           .add_systems(OnEnter(GameState::Score), setup_score_screen)
           .add_systems(Update, exit_on_space.run_if(in_state(GameState::Score)))
           .add_systems(OnExit(GameState::Score), cleanup_screen);
    }
}

#[derive(Component)]
struct ScreenUi;

fn check_game_over(
    stability: Res<ShipStability>, 
    game_mode: Res<GameModeSettings>,
    mut next_state: ResMut<NextState<GameState>>
) {
    if stability.current <= 0.0 {
        // If playing endless and stability hits zero, transition straight to the score screen!
        if game_mode.is_endless {
            next_state.set(GameState::Score);
        } else {
            next_state.set(GameState::GameOver);
        }
    }
}

fn check_win(wave: Res<WaveManager>, game_mode: Res<GameModeSettings>, mut next_state: ResMut<NextState<GameState>>) {
    // Trigger win screen once the 3 base waves are beaten (and not in endless loop yet)
    if !game_mode.is_endless && wave.current_wave > wave.max_waves && !wave.is_active {
        next_state.set(GameState::Win);
    }
}

fn start_game_on_space(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}

fn exit_on_space(keys: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Space) {
        app_exit.write(AppExit::Success);
    }
}

fn handle_win_inputs(
    keys: Res<ButtonInput<KeyCode>>,
    mut game_mode: ResMut<GameModeSettings>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        // Endless Exploration selected
        game_mode.is_endless = true;
        next_state.set(GameState::Playing);
    } else if keys.just_pressed(KeyCode::KeyC) {
        // Chill score view selected
        game_mode.is_endless = false;
        next_state.set(GameState::Score);
    }
}

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match state.get() {
            GameState::Playing => next_state.set(GameState::Paused),
            GameState::Paused => next_state.set(GameState::Playing),
            _ => {}
        }
    }
}

fn cleanup_screen(mut commands: Commands, query: Query<Entity, With<ScreenUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn spawn_text_screen(commands: &mut Commands, title: &str, body: &str, footer: &str, bg_color: Color) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(bg_color),
        ZIndex(100),
        ScreenUi,
    )).with_children(|parent| {
        parent.spawn((
            Text::new(title),
            TextFont { font_size: bevy::text::FontSize::Px(52.0), ..default() },
            TextColor(Color::srgb(1.0, 0.8, 0.2)),
            Node { margin: UiRect::bottom(Val::Px(25.0)), ..default() }
        ));
        parent.spawn((
            Text::new(body),
            TextFont { font_size: bevy::text::FontSize::Px(18.0), ..default() },
            TextLayout::justify(Justify::Center), 
            Node { margin: UiRect::bottom(Val::Px(40.0)), ..default() }
        ));
        parent.spawn((
            Text::new(footer),
            TextFont { font_size: bevy::text::FontSize::Px(18.0), ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    });
}

fn setup_intro(mut commands: Commands) {
    spawn_text_screen(
        &mut commands,
        "HOMECOMING",
        "Commander, your ship was hit by an asteroid and is limping home.\nYour thrusters and comms are heavily damaged.\n\nThankfully, your built-in Pookie AI has mapped a route back to Pookietopia.\nManage your Ship Stability and survive the hostile territories ahead.",
        "Press SPACE to Start",
        Color::srgb(0.05, 0.05, 0.1),
    );
}

fn setup_pause(mut commands: Commands) {
    spawn_text_screen(
        &mut commands,
        "PAUSED",
        "Take a breather, Commander.",
        "Press ESC to Resume",
        Color::srgba(0.0, 0.0, 0.0, 0.85),
    );
}

fn setup_game_over(mut commands: Commands) {
    spawn_text_screen(
        &mut commands,
        "CRITICAL FAILURE",
        "Ship stability has reached zero.\nThe ship has broken apart in deep space.",
        "Press SPACE to Exit",
        Color::srgb(0.15, 0.0, 0.0),
    );
}

fn setup_win(mut commands: Commands) {
    spawn_text_screen(
        &mut commands,
        "POOKIETOPIA REACHED",
        "1. You have defeated all the enemies in your way like the certified brave pookie you are :3\n\
         2. And you have reached your home planet, Pookietopia.\n\
         3. Everyone was happy to see you return, threw a party for you, and repaired your spaceship too.\n\
         4. Do you want to explore space more (Endless) or Chill (View your Score now)?",
        "Press [E] for Endless  |  Press [C] for Chill Score Page",
        Color::srgb(0.0, 0.15, 0.05),
    );
}

fn setup_score_screen(mut commands: Commands, stats: Res<PlayerStats>) {
    let accuracy = if stats.lasers_fired > 0 {
        (stats.lasers_hit as f32 / stats.lasers_fired as f32) * 100.0
    } else {
        0.0
    };

    let grade = match accuracy as u32 {
        90..=100 => "S+ (Certified Pookie Master)",
        75..=89  => "A (Awesome Pookie)",
        50..=74  => "B (Brave Pookie)",
        _        => "C (Needs More Practice :3)",
    };

    let body_text = format!(
        "SCORE PAGE:\n\n\
         Shows your score for the current run:\n\
         - Total Damage Dealt: {:.0}\n\
         - Lasers Fired: {} | Lasers Hit: {}\n\
         - Accuracy: {:.1}% (Grade: {})\n\n\
         Are you a proud certified pookie enjoyer???\n\
         If yes, share your score with your friends and flex on them :3\n\n\
         And if you liked this game a lot and wanna support this game\n\
         go to my itch.io page [[https://your-itch-io-placeholder-link.itch.io]] to dono and comment.\n\n\
         Thank you very much for playing my first game ever :3",
        stats.total_damage, stats.lasers_fired, stats.lasers_hit, accuracy, grade
    );

    spawn_text_screen(
        &mut commands,
        "MISSION DEBRIEF",
        &body_text,
        "Press SPACE to Exit",
        Color::srgb(0.05, 0.05, 0.1),
    );
}

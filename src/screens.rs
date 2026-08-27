use bevy::prelude::*;
use bevy::app::AppExit;
use crate::state::{GameState, VolumeSettings};
use crate::ship::{ShipStability, PlayerStats, GameModeSettings};
use crate::enemy::WaveManager;

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (check_game_over, check_win).run_if(in_state(GameState::Playing)))
           .add_systems(Update, toggle_pause) 
           
           // Controls Screen
           .add_systems(OnEnter(GameState::Controls), setup_controls)
           .add_systems(Update, advance_from_controls.run_if(in_state(GameState::Controls)))
           .add_systems(OnExit(GameState::Controls), cleanup_screen)

           // Pre-Game Story Screen
           .add_systems(OnEnter(GameState::Story), setup_story)
           .add_systems(Update, handle_story_sequence.run_if(in_state(GameState::Story)))
           .add_systems(OnExit(GameState::Story), cleanup_screen)
           
           // Paused & Settings
           .add_systems(OnEnter(GameState::Paused), setup_pause)
           .add_systems(Update, handle_pause_menu.run_if(in_state(GameState::Paused)))
           .add_systems(OnExit(GameState::Paused), cleanup_screen)
           
           // Game Over
           .add_systems(OnEnter(GameState::GameOver), setup_game_over)
           .add_systems(Update, handle_end_screen_inputs.run_if(in_state(GameState::GameOver)))
           .add_systems(OnExit(GameState::GameOver), cleanup_screen)
           
           // Post-Game Story Screen
           .add_systems(OnEnter(GameState::WinStory), setup_win_story)
           .add_systems(Update, handle_story_sequence.run_if(in_state(GameState::WinStory)))
           .add_systems(OnExit(GameState::WinStory), cleanup_screen)

           // Win Screen (Endless / Score choice)
           .add_systems(OnEnter(GameState::Win), setup_win)
           .add_systems(Update, handle_win_inputs.run_if(in_state(GameState::Win)))
           .add_systems(OnExit(GameState::Win), cleanup_screen)

           // Score Screen
           .add_systems(OnEnter(GameState::Score), setup_score_screen)
           .add_systems(Update, handle_end_screen_inputs.run_if(in_state(GameState::Score)))
           .add_systems(OnExit(GameState::Score), cleanup_screen);
    }
}

#[derive(Component)]
struct ScreenUi;

#[derive(Component)]
struct StorySequence {
    lines: Vec<String>,
    current_line: usize,
    char_index: usize,
    timer: Timer,
    next_state: GameState,
}

#[derive(Component)]
struct VolumeActionBtn(VolumeAction);

#[derive(Clone, Copy)]
enum VolumeAction { MusicUp, MusicDown, SfxUp, SfxDown }

#[derive(Component)]
struct VolumeDisplay(VolumeType);

#[derive(Clone, Copy)]
enum VolumeType { Music, Sfx }

fn check_game_over(
    stability: Res<ShipStability>, 
    game_mode: Res<GameModeSettings>,
    mut next_state: ResMut<NextState<GameState>>
) {
    if stability.current <= 0.0 {
        if game_mode.is_endless {
            next_state.set(GameState::Score);
        } else {
            next_state.set(GameState::GameOver);
        }
    }
}

fn check_win(wave: Res<WaveManager>, game_mode: Res<GameModeSettings>, mut next_state: ResMut<NextState<GameState>>) {
    if !game_mode.is_endless && wave.current_wave > wave.max_waves && !wave.is_active {
        next_state.set(GameState::WinStory);
    }
}

fn advance_from_controls(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Story);
    }
}

fn handle_end_screen_inputs(
    keys: Res<ButtonInput<KeyCode>>,
    mut app_exit: MessageWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        app_exit.write(AppExit::Success);
    } else if keys.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }
}

fn handle_win_inputs(
    keys: Res<ButtonInput<KeyCode>>,
    mut game_mode: ResMut<GameModeSettings>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        game_mode.is_endless = true;
        next_state.set(GameState::Playing);
    } else if keys.just_pressed(KeyCode::KeyC) {
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

fn handle_story_sequence(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut query: Query<(&mut StorySequence, &mut Text)>,
) {
    for (mut seq, mut text) in &mut query {
        if keys.just_pressed(KeyCode::Escape) {
            next_state.set(seq.next_state);
            return;
        }

        if keys.just_pressed(KeyCode::Space) {
            if seq.current_line < seq.lines.len() {
                let current_len = seq.lines[seq.current_line].chars().count();
                if seq.char_index < current_len {
                    seq.char_index = current_len;
                } else {
                    seq.current_line += 1;
                    seq.char_index = 0;
                }
            } else {
                next_state.set(seq.next_state);
                return;
            }
        }

        if seq.current_line < seq.lines.len() {
            seq.timer.tick(time.delta());
            if seq.timer.just_finished() {
                let current_len = seq.lines[seq.current_line].chars().count();
                if seq.char_index < current_len {
                    seq.char_index += 1;
                }
            }
        }

        let max_visible_lines = 5; 
        
        let start_index = if seq.current_line >= max_visible_lines {
            seq.current_line - max_visible_lines + 1
        } else {
            0
        };

        let mut display = String::new();
        for i in start_index..seq.current_line {
            if i < seq.lines.len() {
                display.push_str(&seq.lines[i]);
                display.push_str("\n\n");
            }
        }
        
        if seq.current_line < seq.lines.len() {
            let current_line_text: String = seq.lines[seq.current_line]
                .chars()
                .take(seq.char_index)
                .collect();
            display.push_str(&current_line_text);
        }

        text.0 = display;
    }
}

fn spawn_story_screen(commands: &mut Commands, title: &str, lines: Vec<&str>, next_state: GameState) {
    let string_lines: Vec<String> = lines.into_iter().map(|s| s.to_string()).collect();

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.05, 0.05, 0.1)),
        ZIndex(100),
        ScreenUi,
    )).with_children(|parent| {
        parent.spawn((
            Text::new(title),
            TextFont { font_size: bevy::text::FontSize::Px(42.0), ..default() },
            TextColor(Color::srgb(1.0, 0.8, 0.2)),
            Node { margin: UiRect::bottom(Val::Px(40.0)), ..default() }
        ));

        parent.spawn((
            Text::new(""),
            TextFont { font_size: bevy::text::FontSize::Px(20.0), ..default() },
            TextColor(Color::WHITE),
            TextLayout::justify(Justify::Left),
            Node {
                width: Val::Percent(80.0),
                height: Val::Percent(60.0),
                ..default()
            },
            StorySequence {
                lines: string_lines,
                current_line: 0,
                char_index: 0,
                timer: Timer::from_seconds(0.02, TimerMode::Repeating),
                next_state,
            },
        ));

        parent.spawn((
            Text::new("Press [SPACE] to continue  |  Press [ESC] to skip"),
            TextFont { font_size: bevy::text::FontSize::Px(16.0), ..default() },
            TextColor(Color::srgb(0.5, 0.5, 0.5)),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(30.0),
                ..default()
            }
        ));
    });
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

fn setup_controls(mut commands: Commands) {
    spawn_text_screen(
        &mut commands,
        "HOW TO PLAY",
        "KEYBOARD:\n\
         [W] [A] [S] [D]  -  For Moving\n\
         [ESC]  -  Pause in-game & Audio Settings\n\n\
         MOUSE:\n\
         Mouse Movement  -  For Aiming\n\
         Left Click  -  For Shooting\n\n\
         COMPASS:\n\
         Move around and find the yellow mark on your compass.\n\
         Move to these checkpoints one by one in order to keep progressing!",
        "Press [SPACE] to read the Story",
        Color::srgb(0.02, 0.02, 0.06),
    );
}

fn setup_story(mut commands: Commands) {
    spawn_story_screen(
        &mut commands,
        "PROLOGUE",
        vec![
            "1. You are a proud citizen of Pookietopia!",
            "2. It's a beautiful world where everyone is a certified Pookie, living happily in peace and harmony.",
            "3. One day, while out doing important Pookie research in your spaceship...",
            "4. BAM! A massive asteroid crashes directly into the back of your ship!",
            "5. The impact is so intense that you instantly faint on the deck.",
            "6. Three days later, you finally wake up... only to realize your ship has drifted far, far away from Pookietopia.",
            "7. Emergency power is barely keeping the lights on.",
            "8. Your rear thrusters, comms, and navigation maps are all completely busted.",
            "9. But don't lose hope! Your trusty Pookie AI managed to log the coordinates of the space-checkpoints you passed by chance.",
            "10. If you follow them back, you can reach back home. Good luck, Commander :3",
        ],
        GameState::Playing,
    );
}

fn setup_win_story(mut commands: Commands) {
    spawn_story_screen(
        &mut commands,
        "HOMECOMING",
        vec![
            "1. You defeated all the enemies in your way like the certified brave Pookie you are! :3",
            "2. At long last, you have safely reached your home planet, Pookietopia.",
            "3. Everyone was overjoyed to see you return! They threw a massive welcome-home party and even completely repaired your spaceship.",
        ],
        GameState::Win,
    );
}

fn setup_pause(mut commands: Commands, volume: Res<VolumeSettings>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ZIndex(100),
        ScreenUi,
    )).with_children(|parent| {
        parent.spawn((
            Text::new("PAUSED"),
            TextFont { font_size: bevy::text::FontSize::Px(52.0), ..default() },
            TextColor(Color::srgb(1.0, 0.8, 0.2)),
            Node { margin: UiRect::bottom(Val::Px(40.0)), ..default() }
        ));

        // --- Music Row ---
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Px(320.0),
            margin: UiRect::bottom(Val::Px(25.0)),
            ..default()
        }).with_children(|row| {
            row.spawn((Text::new("Music"), TextFont { font_size: bevy::text::FontSize::Px(24.0), ..default() }, Node { width: Val::Px(90.0), ..default() }));
            row.spawn((
                Button,
                Node { width: Val::Px(45.0), height: Val::Px(45.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                VolumeActionBtn(VolumeAction::MusicDown)
            )).with_children(|btn| { btn.spawn((Text::new("-"), TextFont { font_size: bevy::text::FontSize::Px(28.0), ..default() })); });
            row.spawn((
                Text::new(format!("{:.0}%", volume.music * 100.0)),
                TextFont { font_size: bevy::text::FontSize::Px(24.0), ..default() },
                VolumeDisplay(VolumeType::Music)
            ));
            row.spawn((
                Button,
                Node { width: Val::Px(45.0), height: Val::Px(45.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                VolumeActionBtn(VolumeAction::MusicUp)
            )).with_children(|btn| { btn.spawn((Text::new("+"), TextFont { font_size: bevy::text::FontSize::Px(28.0), ..default() })); });
        });

        // --- SFX Row ---
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Px(320.0),
            margin: UiRect::bottom(Val::Px(25.0)),
            ..default()
        }).with_children(|row| {
            row.spawn((Text::new("SFX"), TextFont { font_size: bevy::text::FontSize::Px(24.0), ..default() }, Node { width: Val::Px(90.0), ..default() }));
            row.spawn((
                Button,
                Node { width: Val::Px(45.0), height: Val::Px(45.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                VolumeActionBtn(VolumeAction::SfxDown)
            )).with_children(|btn| { btn.spawn((Text::new("-"), TextFont { font_size: bevy::text::FontSize::Px(28.0), ..default() })); });
            row.spawn((
                Text::new(format!("{:.0}%", volume.sfx * 100.0)),
                TextFont { font_size: bevy::text::FontSize::Px(24.0), ..default() },
                VolumeDisplay(VolumeType::Sfx)
            ));
            row.spawn((
                Button,
                Node { width: Val::Px(45.0), height: Val::Px(45.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                VolumeActionBtn(VolumeAction::SfxUp)
            )).with_children(|btn| { btn.spawn((Text::new("+"), TextFont { font_size: bevy::text::FontSize::Px(28.0), ..default() })); });
        });

        parent.spawn((
            Text::new("Press [ESC] to Resume"),
            TextFont { font_size: bevy::text::FontSize::Px(18.0), ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
            Node { margin: UiRect::top(Val::Px(40.0)), ..default() }
        ));
    });
}

fn handle_pause_menu(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, &VolumeActionBtn), (Changed<Interaction>, With<Button>)>,
    mut volume: ResMut<VolumeSettings>,
    mut text_query: Query<(&mut Text, &VolumeDisplay)>,
) {
    for (interaction, mut color, action_btn) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
                match action_btn.0 {
                    VolumeAction::MusicUp => volume.music = (volume.music + 0.1).clamp(0.0, 1.0),
                    VolumeAction::MusicDown => volume.music = (volume.music - 0.1).clamp(0.0, 1.0),
                    VolumeAction::SfxUp => volume.sfx = (volume.sfx + 0.1).clamp(0.0, 1.0),
                    VolumeAction::SfxDown => volume.sfx = (volume.sfx - 0.1).clamp(0.0, 1.0),
                }
            }
            Interaction::Hovered => *color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            Interaction::None => *color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        }
    }

    if volume.is_changed() {
        for (mut text, display) in &mut text_query {
            match display.0 {
                VolumeType::Music => text.0 = format!("{:.0}%", volume.music * 100.0),
                VolumeType::Sfx => text.0 = format!("{:.0}%", volume.sfx * 100.0),
            }
        }
    }
}

fn setup_game_over(mut commands: Commands) {
    spawn_text_screen(&mut commands, "CRITICAL FAILURE", "Ship stability has reached zero.\nThe ship has broken apart in deep space.", "Press [R] for New Run  |  Press [SPACE] to Exit", Color::srgb(0.15, 0.0, 0.0));
}

fn setup_win(mut commands: Commands) {
    spawn_text_screen(&mut commands, "MISSION ACCOMPLISHED", "The Pookies celebrate your triumphant return!", "Press [E] to Explore More Space (Endless)  |  Press [C] to Chill (View Score)", Color::srgb(0.0, 0.15, 0.05));
}

fn setup_score_screen(mut commands: Commands, stats: Res<PlayerStats>) {
    let accuracy = if stats.lasers_fired > 0 { (stats.lasers_hit as f32 / stats.lasers_fired as f32) * 100.0 } else { 0.0 };
    let grade = match accuracy as u32 { 90..=100 => "S+ (Certified Pookie Master)", 75..=89 => "A (Awesome Pookie)", 50..=74 => "B (Brave Pookie)", _ => "C (Needs More Practice :3)" };
    
    let body_text = format!(
        "SCORE PAGE:\n\n\
         Shows your score for the current run:\n\
         - Total Damage Dealt: {:.0}\n\
         - Lasers Fired: {} | Lasers Hit: {}\n\
         - Accuracy: {:.1}% (Grade: {})\n\n\
         Are you a proud certified pookie enjoyer???\n\
         If yes, share your score with your friends and flex on them :3\n\n\
          I know it's a small demo game, but I did my best on it all by myself in 9 days.\n\
         If you had fun playing it, or saw potential in the idea and wanna support it, \
         please consider leaving a rating or a small donation on my itch.io page :3\n\n\
         I have a lot of ideas in my head for this game, and I'm hoping to turn it into \
         a full roguelike/roguelite someday with some kinda item and build system, so stay tuned :3\n\n\
         BTW, thank you very much for playing my first game ever :3",
        stats.total_damage, stats.lasers_fired, stats.lasers_hit, accuracy, grade
    );
    spawn_text_screen(&mut commands, "MISSION DEBRIEF", &body_text, "Press [R] for New Run  |  Press [SPACE] to Exit", Color::srgb(0.05, 0.05, 0.1));
}

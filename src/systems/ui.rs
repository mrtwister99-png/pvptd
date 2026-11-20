
use bevy::prelude::*;
use crate::components::{GoldText, DiamondText, LivesText, WaveText, WaveTimerText, Side, GameOverRoot, GameOverMenuButton};
use crate::resources::{Sides, Diamonds, WaveCount, RoundState, RoundPhase, WavePaused, MatchOver};
use crate::wave_def::get_creep_color_and_hp;
use crate::constants::BOSS_EVERY;
use crate::menu::AppState;

pub fn update_ui(
    sides: Res<Sides>,
    diamonds: Res<Diamonds>,
    wave: Res<WaveCount>,
    mut q_esc: Query<&mut Text, (With<LivesText>, Without<GoldText>, Without<DiamondText>, Without<WaveText>, Without<WaveTimerText>)>,
    mut q_gold: Query<&mut Text, (With<GoldText>, Without<LivesText>, Without<DiamondText>, Without<WaveText>, Without<WaveTimerText>)>,
    mut q_diamond: Query<&mut Text, (With<DiamondText>, Without<GoldText>, Without<LivesText>, Without<WaveText>, Without<WaveTimerText>)>,
    mut q_wave: Query<&mut Text, (With<WaveText>, Without<GoldText>, Without<DiamondText>, Without<LivesText>, Without<WaveTimerText>)>,
) {
    for mut t in q_gold.iter_mut() { t.0 = format!("Gold: {}", sides.get(Side::Player).gold); }
    for mut t in q_diamond.iter_mut() { t.0 = format!("Diamonds: {}", diamonds.0); }
    if sides.is_changed() {
        let s = format!("Zivoty: {} | Bot: {}", sides.get(Side::Player).lives, sides.get(Side::Bot).lives);
        for mut t in q_esc.iter_mut() { if t.0 != s { t.0 = s.clone(); } }
    }
    if wave.is_changed() {
        let (_, hp) = get_creep_color_and_hp(wave.0);
        let boss = if wave.0 > 0 && wave.0 % BOSS_EVERY == 0 { " BOSS" } else { "" };
        for mut t in q_wave.iter_mut() { t.0 = format!("Kolo: {} ({}HP){}", wave.0, hp as i32, boss); }
    }
}

// odpocet vlevo nahore: stavba / pauza do dalsiho kola / bezici kolo
pub fn update_wave_timer(
    round: Res<RoundState>,
    paused: Res<WavePaused>,
    wave: Res<WaveCount>,
    over: Res<MatchOver>,
    mut q_timer: Query<&mut Text, (With<WaveTimerText>, Without<GoldText>, Without<DiamondText>, Without<WaveText>, Without<LivesText>)>,
) {
    let remaining = round.timer.remaining_secs().ceil().max(0.0);
    let s = if over.0.is_some() {
        "KONEC ZAPASU".to_string()
    } else if paused.0 {
        "PAUZA".to_string()
    } else {
        match round.phase {
            RoundPhase::Pause if wave.0 == 0 => format!("Stavba: {:.0}s", remaining),
            RoundPhase::Pause => {
                let boss = if (wave.0 + 1) % BOSS_EVERY == 0 { " (BOSS!)" } else { "" };
                format!("Dalsi kolo za {:.0}s{}", remaining, boss)
            }
            RoundPhase::Running => format!("Kolo {} - posilej!", wave.0),
        }
    };
    for mut t in q_timer.iter_mut() { if t.0 != s { t.0 = s.clone(); } }
}

// konec zapasu: zastavit hru a ukazat VYHRA / PROHRA s tlacitkem zpet do menu
pub fn game_over_system(
    mut commands: Commands,
    over: Res<MatchOver>,
    mut paused: ResMut<WavePaused>,
    sides: Res<Sides>,
    q_root: Query<(), With<GameOverRoot>>,
) {
    let Some(winner) = over.0 else { return; };
    if !q_root.is_empty() { return; }
    paused.0 = true;
    let (title, color) = if winner == Side::Player { ("VYHRA!", Color::srgb(0.3, 0.9, 0.35)) } else { ("PROHRA", Color::srgb(0.95, 0.3, 0.3)) };
    let score = format!("Zivoty: {} | Bot: {}", sides.get(Side::Player).lives, sides.get(Side::Bot).lives);
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(18.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        GlobalZIndex(100),
        GameOverRoot,
    )).with_children(|p| {
        p.spawn((Text::new(title), TextFont { font_size: 48.0, ..default() }, TextColor(color)));
        p.spawn((Text::new(score), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
        p.spawn((
            Button,
            Node { width: Val::Px(180.0), height: Val::Px(56.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgb(0.25, 0.35, 0.55)),
            GameOverMenuButton,
        )).with_children(|pp| {
            pp.spawn((Text::new("MENU"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
        });
    });
}

pub fn game_over_button_system(
    q: Query<&Interaction, (Changed<Interaction>, With<GameOverMenuButton>)>,
    mut next: ResMut<NextState<AppState>>,
) {
    if q.iter().any(|i| *i == Interaction::Pressed) { next.set(AppState::Menu); }
}

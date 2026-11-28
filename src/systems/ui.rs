
use bevy::prelude::*;
use crate::components::{GoldText, DiamondText, LivesText, WaveText, WaveTimerText, Side, GameOverRoot, GameOverMenuButton, PauseMenuRoot, PauseOptionsRoot, PauseConfirmRoot, PauseConfirmText, PauseMenuButton, PauseResumeButton, PauseConfirmYesButton, PauseConfirmNoButton};
use crate::resources::{Sides, WaveCount, RoundState, RoundPhase, WavePaused, MatchOver, GameMode};
use crate::wave_def::{wave_def, boss_def, is_boss_wave, LAST_WAVE};
use crate::menu::{AppState, PendingPauseAction, PauseAction, PauseOutcome, resolve_pause_confirmation};

// pravy horni ctverecek: vzdy zivoty soupere (bota) + kam tlacitko prepne
pub fn opponent_hp_system(
    sides: Res<Sides>,
    view: Res<crate::resources::ViewSide>,
    mut q: Query<&mut Text, With<crate::components::SwitchViewLabel>>,
) {
    let target = if view.0 == Side::Player { "SOUPER" } else { "MOJE" };
    let s = format!("{} [TAB]\nBOT\n{} HP", target, sides.get(Side::Bot).lives);
    for mut t in q.iter_mut() { if t.0 != s { t.0 = s.clone(); } }
}

pub fn update_ui(
    sides: Res<Sides>,
    wave: Res<WaveCount>,
    mut q_esc: Query<&mut Text, (With<LivesText>, Without<GoldText>, Without<DiamondText>, Without<WaveText>, Without<WaveTimerText>)>,
    mut q_gold: Query<&mut Text, (With<GoldText>, Without<LivesText>, Without<DiamondText>, Without<WaveText>, Without<WaveTimerText>)>,
    mut q_diamond: Query<&mut Text, (With<DiamondText>, Without<GoldText>, Without<LivesText>, Without<WaveText>, Without<WaveTimerText>)>,
    mut q_wave: Query<&mut Text, (With<WaveText>, Without<GoldText>, Without<DiamondText>, Without<LivesText>, Without<WaveTimerText>)>,
    mode: Res<GameMode>,
) {
    if mode.is_test() {
        for mut t in q_gold.iter_mut() { if t.0 != "Gold: neomezene" { t.0 = "Gold: neomezene".to_string(); } }
        for mut t in q_diamond.iter_mut() { if t.0 != "Diamonds: neomezene" { t.0 = "Diamonds: neomezene".to_string(); } }
        for mut t in q_esc.iter_mut() { if t.0 != "TEST - zivoty se nepocitaji" { t.0 = "TEST - zivoty se nepocitaji".to_string(); } }
    } else {
        for mut t in q_gold.iter_mut() { t.0 = format!("Gold: {}", sides.get(Side::Player).gold); }
        for mut t in q_diamond.iter_mut() { t.0 = format!("Diamonds: {}", sides.get(Side::Player).diamonds); }
    }
    if sides.is_changed() && !mode.is_test() {
        let s = format!("Zivoty: {} | Bot: {}", sides.get(Side::Player).lives, sides.get(Side::Bot).lives);
        for mut t in q_esc.iter_mut() { if t.0 != s { t.0 = s.clone(); } }
    }
    if wave.is_changed() {
        let w = wave_def(wave.0.max(1));
        let boss = w.boss.map_or(String::new(), |(b, _)| format!(" BOSS {}", boss_def(b).name));
        let sd = if wave.0 > LAST_WAVE { " SUDDEN DEATH" } else { "" };
        for mut t in q_wave.iter_mut() { t.0 = format!("Vlna: {}/{} ({}HP){}{}", wave.0, LAST_WAVE, w.small.2 as i32, boss, sd); }
    }
}

// odpocet vlevo nahore: stavba / pauza do dalsiho kola / bezici kolo
pub fn update_wave_timer(
    round: Res<RoundState>,
    paused: Res<WavePaused>,
    wave: Res<WaveCount>,
    over: Res<MatchOver>,
    mut q_timer: Query<&mut Text, (With<WaveTimerText>, Without<GoldText>, Without<DiamondText>, Without<WaveText>, Without<LivesText>)>,
    mode: Res<GameMode>,
) {
    let remaining = round.timer.remaining_secs().ceil().max(0.0);
    let s = if mode.is_test() {
        if paused.0 { "TEST - PAUZA".to_string() } else { "TEST REZIM".to_string() }
    } else if over.0.is_some() {
        "KONEC ZAPASU".to_string()
    } else if paused.0 {
        "PAUZA".to_string()
    } else {
        match round.phase {
            RoundPhase::Pause if wave.0 == 0 => format!("Stavba: {:.0}s", remaining),
            RoundPhase::Pause => {
                let boss = if is_boss_wave(wave.0 + 1) { " (BOSS!)" } else { "" };
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

pub fn pause_menu_system(
    mut paused: ResMut<WavePaused>,
    over: Res<MatchOver>,
    mut pending: ResMut<PendingPauseAction>,
    mut next: ResMut<NextState<AppState>>,
    mut app_exit: EventWriter<AppExit>,
    mut q_root: Query<&mut Node, (With<PauseMenuRoot>, Without<PauseOptionsRoot>, Without<PauseConfirmRoot>)>,
    mut q_options: Query<&mut Node, (With<PauseOptionsRoot>, Without<PauseConfirmRoot>)>,
    mut q_confirm: Query<&mut Node, (With<PauseConfirmRoot>, Without<PauseOptionsRoot>)>,
    mut q_text: Query<&mut Text, With<PauseConfirmText>>,
    q_actions: Query<(&Interaction, &PauseMenuButton), Changed<Interaction>>,
    q_resume: Query<&Interaction, (Changed<Interaction>, With<PauseResumeButton>)>,
    q_yes: Query<&Interaction, (Changed<Interaction>, With<PauseConfirmYesButton>)>,
    q_no: Query<&Interaction, (Changed<Interaction>, With<PauseConfirmNoButton>)>,
) {
    if over.0.is_some() {
        pending.0 = None;
    } else {
        for (interaction, button) in q_actions.iter() {
            if *interaction == Interaction::Pressed { pending.0 = Some(button.0); }
        }
        if q_no.iter().any(|interaction| *interaction == Interaction::Pressed) {
            pending.0 = None;
        }
        if q_resume.iter().any(|interaction| *interaction == Interaction::Pressed) {
            paused.0 = false;
            pending.0 = None;
        }
        if q_yes.iter().any(|interaction| *interaction == Interaction::Pressed) {
            if let Some(action) = pending.0 {
                match resolve_pause_confirmation(action, true) {
                    Some(PauseOutcome::Restart) => next.set(AppState::Restarting),
                    Some(PauseOutcome::Menu) => next.set(AppState::Menu),
                    Some(PauseOutcome::Exit) => { app_exit.send(AppExit::Success); }
                    None => {}
                }
                pending.0 = None;
            }
        }
    }

    let visible = paused.0 && over.0.is_none();
    if !visible { pending.0 = None; }
    for mut node in q_root.iter_mut() {
        node.display = if visible { Display::Flex } else { Display::None };
    }
    for mut node in q_options.iter_mut() {
        node.display = if visible && pending.0.is_none() { Display::Flex } else { Display::None };
    }
    for mut node in q_confirm.iter_mut() {
        node.display = if visible && pending.0.is_some() { Display::Flex } else { Display::None };
    }
    if let Some(action) = pending.0 {
        let text = match action {
            PauseAction::Restart => "Opravdu restartovat zapas?",
            PauseAction::Menu => "Opravdu odejit do menu?",
            PauseAction::Exit => "Opravdu ukoncit hru?",
        };
        for mut label in q_text.iter_mut() {
            if label.0 != text { label.0 = text.to_string(); }
        }
    }
}

// panel se staty vybrane veze; volba stavby je v samostatnem panelu
pub fn tower_info_system(
    selected: Res<crate::resources::SelectedCell>,
    panels: Res<crate::resources::PanelsOpen>,
    towers: Query<(&crate::components::Tower, Option<&crate::components::TowerMods>)>,
    round: Res<RoundState>,
    mut q_panel: Query<&mut Node, With<crate::components::TowerInfoPanel>>,
    mut q_text: Query<&mut Text, With<crate::components::TowerInfoText>>,
) {
    use crate::constants::{ALL_TOWERS, TOWER_POISON};
    let selected_tower = selected.0.map_or(false, |(r, c)| {
        towers.iter().any(|(t, _)| t.side == Side::Player && t.row == r && t.col == c)
    });
    let show = selected_tower && !panels.send;
    for mut n in q_panel.iter_mut() { let d = if show { Display::Flex } else { Display::None }; if n.display != d { n.display = d; } }
    let Some((r, c)) = selected.0 else { return; };
    let name_of = |k: u8| ALL_TOWERS.iter().find(|t| t.0 == k).map_or("?", |t| t.1);
    let s = match towers.iter().find(|(t, _)| t.side == Side::Player && t.row == r && t.col == c) {
        Some((t, mods)) => {
            let def = crate::tower_def::get_tower_def(t.kind, t.level);
            let title = if t.level > 5 { format!("{}+", def.name) } else { def.name.to_string() };
            let mut lines = vec![format!("{}  ({} R{})", title.to_uppercase(), name_of(t.kind), t.level)];
            let rate_mul = mods.map_or(1.0, |m| m.rate_mul);
            let dmg_mul = mods.map_or(1.0, |m| m.dmg_mul);
            if crate::special::shoots(t.kind) {
                lines.push(format!("Poskozeni: {:.0}-{:.0}   Utoku: {:.2}/s", t.damage_min * dmg_mul, t.damage_max * dmg_mul, t.fire_rate * rate_mul));
            }
            if t.range > 0.0 { lines.push(format!("Dosah: {:.0}", t.range)); }
            let mut extra = Vec::new();
            if t.splash > 0.0 { extra.push(format!("plosne {:.0}", t.splash)); }
            if t.slow_max > 0.0 { extra.push(format!("zpomaleni az {:.0}%", t.slow_max * 100.0)); }
            if t.dot_dps > 0.0 { extra.push(format!("{} {:.1}/s na {:.0}s", if t.kind == TOWER_POISON { "jed" } else { "ohen" }, t.dot_dps, t.dot_duration)); }
            if t.chain_targets > 0 { extra.push(format!("preskok na {} dalsi", t.chain_targets)); }
            if t.push_chance > 0.0 { extra.push(format!("{:.0}% odhozeni", t.push_chance * 100.0)); }
            if !extra.is_empty() { lines.push(extra.join(", ")); }
            let special = crate::special::describe(t.kind, t.level);
            if !special.is_empty() { lines.push(special); }
            if rate_mul > 1.0 || dmg_mul > 1.0 { lines.push("Posilena podporou".to_string()); }
            lines.push(format!("Typ: {:?}  Prodej: +{}G", crate::wave_def::tower_dmg_type(t.kind), crate::systems::tower_build::sell_price(t.invested, round.phase)));
            lines.join("
")
        }
        None => {
            "VYBER VEZ Z NABIDKY".to_string()
        }
    };
    // napoveda pro klavesnici
    let s = if towers.iter().any(|(t, _)| t.side == Side::Player && t.row == r && t.col == c) {
        format!("{}\n\nEnter = vylepsit, S = prodat", s)
    } else {
        format!("{}\n\nEnter = postavit", s)
    };
    for mut t in q_text.iter_mut() { if t.0 != s { t.0 = s.clone(); } }
}

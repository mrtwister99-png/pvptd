
use bevy::prelude::*;
use crate::components::{GoldText, DiamondText, LivesText, WaveText, WaveTimerText, Side, GameOverRoot, GameOverMenuButton};
use crate::resources::{Sides, Diamonds, WaveCount, RoundState, RoundPhase, WavePaused, MatchOver};
use crate::wave_def::get_creep_color_and_hp;
use crate::constants::BOSS_EVERY;
use crate::menu::AppState;

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

// panel se staty: vybrana vez = jmeno, rank, poskozeni, rychlost, dosah, schopnost, prodej;
// vybrane prazdne policko = z jakych vezi muze POSTAVIT vybrat
pub fn tower_info_system(
    selected: Res<crate::resources::SelectedCell>,
    sides: Res<Sides>,
    panels: Res<crate::resources::PanelsOpen>,
    kc: Res<crate::resources::KeyControl>,
    towers: Query<(&crate::components::Tower, Option<&crate::components::TowerMods>)>,
    mut q_panel: Query<&mut Node, With<crate::components::TowerInfoPanel>>,
    mut q_text: Query<&mut Text, With<crate::components::TowerInfoText>>,
) {
    use crate::constants::{ALL_TOWERS, SELL_REFUND, TOWER_POISON};
    let show = selected.0.is_some() && !panels.send;
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
            if crate::special::shoots(t.kind) || matches!(t.kind, crate::special::GEYSER | crate::special::SPIKES) {
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
            if rate_mul > 1.0 || dmg_mul > 1.0 { lines.push("Posilena od BELL".to_string()); }
            lines.push(format!("Prodej: +{}G", (t.invested as f32 * SELL_REFUND) as i32));
            lines.join("
")
        }
        None => {
            let deck: Vec<&str> = sides.get(Side::Player).deck.iter().map(|&k| name_of(k)).collect();
            format!("POSTAVIT: nahodna vez R1 z balicku
{}", deck.join(", "))
        }
    };
    // napoveda pro klavesnici
    let s = if let Some(from) = kc.carry {
        let what = towers.iter().find(|(t, _)| t.side == Side::Player && t.row == from.0 && t.col == from.1).map(|(t, _)| format!("{} R{}", name_of(t.kind), t.level)).unwrap_or_default();
        format!("NESES {}: sipkami na stejnou vez,\nEnter = spojit, Esc = zrusit\n\n{}", what, s)
    } else if kc.build_armed == Some((r, c)) {
        format!("Enter znovu = POSTAVIT\n\n{}", s)
    } else { s };
    for mut t in q_text.iter_mut() { if t.0 != s { t.0 = s.clone(); } }
}

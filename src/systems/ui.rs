
use bevy::prelude::*;
use crate::components::{GoldText, DiamondText, LivesText, WaveText, WaveTimerText, Side};
use crate::resources::{Sides, Diamonds, WaveCount, CreepTimer, WavePaused};
use crate::wave_def::get_creep_color_and_hp;
use crate::constants::MAX_WAVES;

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
        for mut t in q_wave.iter_mut() { t.0 = format!("Vlna: {} ({}HP)", wave.0, hp as i32); }
    }
}

pub fn update_wave_timer(
    timer: Res<CreepTimer>,
    paused: Res<WavePaused>,
    wave: Res<WaveCount>,
    mut q_timer: Query<&mut Text, (With<WaveTimerText>, Without<GoldText>, Without<DiamondText>, Without<WaveText>, Without<LivesText>)>,
) {
    let remaining = timer.0.remaining_secs();
    for mut t in q_timer.iter_mut() {
        if wave.0 >= MAX_WAVES { t.0 = "Next: -".to_string(); } // vsechny vlny uz prisly
        else if paused.0 { t.0 = format!("PAUSED ({:.0}s)", remaining.max(0.0)); }
        else { t.0 = format!("Next: {:.0}s", remaining.max(0.0)); }
    }
}

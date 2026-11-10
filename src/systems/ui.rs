
use bevy::prelude::*;
use crate::components::{GoldText, DiamondText, EscapedText, WaveText, WaveTimerText};
use crate::resources::{EscapedCount, Gold, Diamonds, WaveCount, CreepTimer, WavePaused};
use crate::wave_def::get_creep_color_and_hp;
use crate::constants::MAX_WAVES;

pub fn update_ui(
    escaped: Res<EscapedCount>,
    gold: Res<Gold>,
    diamonds: Res<Diamonds>,
    wave: Res<WaveCount>,
    mut q_esc: Query<&mut Text, (With<EscapedText>, Without<GoldText>, Without<DiamondText>, Without<WaveText>, Without<WaveTimerText>)>,
    mut q_gold: Query<&mut Text, (With<GoldText>, Without<EscapedText>, Without<DiamondText>, Without<WaveText>, Without<WaveTimerText>)>,
    mut q_diamond: Query<&mut Text, (With<DiamondText>, Without<GoldText>, Without<EscapedText>, Without<WaveText>, Without<WaveTimerText>)>,
    mut q_wave: Query<&mut Text, (With<WaveText>, Without<GoldText>, Without<DiamondText>, Without<EscapedText>, Without<WaveTimerText>)>,
) {
    for mut t in q_gold.iter_mut() { t.0 = format!("Gold: {}", gold.0); }
    for mut t in q_diamond.iter_mut() { t.0 = format!("Diamonds: {}", diamonds.0); }
    if escaped.is_changed() { for mut t in q_esc.iter_mut() { t.0 = format!("Proslo: {}/15", escaped.0); } }
    if wave.is_changed() {
        let (_, hp) = get_creep_color_and_hp(wave.0);
        for mut t in q_wave.iter_mut() { t.0 = format!("Vlna: {} ({}HP)", wave.0, hp as i32); }
    }
}

pub fn update_wave_timer(
    timer: Res<CreepTimer>,
    paused: Res<WavePaused>,
    wave: Res<WaveCount>,
    mut q_timer: Query<&mut Text, (With<WaveTimerText>, Without<GoldText>, Without<DiamondText>, Without<WaveText>, Without<EscapedText>)>,
) {
    let remaining = timer.0.remaining_secs();
    for mut t in q_timer.iter_mut() {
        if wave.0 >= MAX_WAVES { t.0 = "Next: -".to_string(); } // vsechny vlny uz prisly
        else if paused.0 { t.0 = format!("PAUSED ({:.0}s)", remaining.max(0.0)); }
        else { t.0 = format!("Next: {:.0}s", remaining.max(0.0)); }
    }
}

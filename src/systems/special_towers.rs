// Pasivni specialni veze, ktere nestrili strelou:
// GEYSER - jednou za cas odhodi vsechny pozemni creepy v okruhu zpet po ceste (+ male poskozeni)
// SPIKES - past: prvni pozemni creep, ktery prijde blizko, spusti vybuch; pak se past nabiji
use bevy::prelude::*;
use crate::components::{Air, Creep, Health, Knockback, PoisonDebuff, Resist, SlowDebuff, Tower};
use crate::resources::{CreepGrid, DamageNumberPool, WavePaused};
use crate::special::{GEYSER, SPIKES, spikes_poison, spikes_slow};
use crate::systems::debuff::spawn_damage_number;

pub fn special_towers_system(
    mut commands: Commands,
    time: Res<Time>,
    paused: Res<WavePaused>,
    grid: Res<CreepGrid>,
    mut pool: ResMut<DamageNumberPool>,
    mut near: Local<Vec<(Entity, Vec2)>>,
    mut towers: Query<(&Transform, &mut Tower)>,
    mut creeps: Query<&mut Health, (With<Creep>, Without<Air>)>,
    resists: Query<&Resist>,
    mut waves: ResMut<crate::fx::ShockwavePool>,
    mut wave_mats: ResMut<Assets<crate::fx::ShockwaveMaterial>>,
) {
    if paused.0 { return; }
    let now = time.elapsed_secs_wrapped();
    let dt = time.delta_secs();
    for (tf, mut t) in towers.iter_mut() {
        if t.kind != GEYSER && t.kind != SPIKES { continue; }
        t.cooldown -= dt;
        if t.cooldown > 0.0 { continue; }
        let pos = tf.translation.truncate();
        near.clear();
        grid.query(pos, t.range, &mut near);
        near.retain(|(e, _)| creeps.contains(*e)); // jen pozemni
        if near.is_empty() { continue; } // ceka nabita, dokud nekdo neprijde
        let dmg = (t.damage_min + t.damage_max) * 0.5;
        if t.kind == GEYSER {
            crate::fx::spawn_shockwave(&mut commands, &mut waves, &mut wave_mats, now, pos, t.range, Color::srgba(0.55, 0.85, 1.0, 0.9), 0.6);
            // pulz: vsichni pozemni v okruhu odleti zpet (odolnost vuci odhozeni se pocita)
            for &(e, p) in near.iter() {
                if let Ok(mut hp) = creeps.get_mut(e) { hp.0 -= dmg; }
                let push = t.push_dist * (1.0 - resists.get(e).map_or(0.0, |r| r.push));
                if let Some(mut ec) = commands.get_entity(e) { ec.try_insert(Knockback { remaining: push, speed: 400.0 }); }
                spawn_damage_number(&mut commands, &mut pool, dmg, Color::srgb(0.55, 0.8, 1.0), Vec2::new(p.x, p.y + 18.0), 25.0, 0.7);
            }
        } else {
            // past: vybuch na miste nejblizsiho creepa, zasahne vsechny pozemni kolem
            let center = near.iter().min_by(|a, b| a.1.distance_squared(pos).total_cmp(&b.1.distance_squared(pos))).map(|n| n.1).unwrap_or(pos);
            crate::fx::spawn_shockwave(&mut commands, &mut waves, &mut wave_mats, now, center, t.splash.max(20.0), Color::srgba(1.0, 0.6, 0.25, 0.9), 0.4);
            near.clear();
            grid.query(center, t.splash.max(1.0), &mut near);
            for &(e, p) in near.iter() {
                let Ok(mut hp) = creeps.get_mut(e) else { continue; };
                hp.0 -= dmg;
                spawn_damage_number(&mut commands, &mut pool, dmg, Color::srgb(0.85, 0.6, 0.3), Vec2::new(p.x, p.y + 18.0), 25.0, 0.7);
                let Some(mut ec) = commands.get_entity(e) else { continue; };
                if spikes_slow(t.level) > 0.0 {
                    ec.try_insert(SlowDebuff { timer: Timer::from_seconds(1.0, TimerMode::Once), factor: 1.0 - spikes_slow(t.level) * (1.0 - resists.get(e).map_or(0.0, |r| r.slow)) });
                }
                if spikes_poison(t.level) > 0.0 {
                    ec.try_insert(PoisonDebuff { timer: Timer::from_seconds(3.0, TimerMode::Once), tick: Timer::from_seconds(0.5, TimerMode::Repeating), dps: dmg * spikes_poison(t.level), level: 5 });
                }
            }
        }
        t.cooldown = 1.0 / t.fire_rate.max(0.01); // fire_rate = 1 / interval z tabulky
    }
}

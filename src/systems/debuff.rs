use bevy::prelude::*;
use crate::components::{SlowDebuff, StunDebuff, BurnDebuff, PoisonDebuff, DamageNumber, Health};
use rand::Rng;
use crate::resources::DamageNumberPool;

pub fn update_debuffs(
    mut commands: Commands,
    mut pool: ResMut<DamageNumberPool>,
    time: Res<Time>,
    mut q_slow: Query<(Entity, &mut SlowDebuff)>,
    mut q_stun: Query<(Entity, &mut StunDebuff)>,
    mut debuffs: ParamSet<(
        Query<(Entity, &Transform, &mut BurnDebuff, &mut Health, Option<&crate::components::Resist>)>,
        Query<(Entity, &Transform, &mut PoisonDebuff, &mut Health, Option<&crate::components::Resist>)>,
    )>,
) {
    for (e, mut slow) in q_slow.iter_mut() {
        slow.timer.tick(time.delta());
        if slow.timer.finished() {
            if let Some(mut ec) = commands.get_entity(e) { ec.remove::<SlowDebuff>(); }
        }
    }
    for (e, mut stun) in q_stun.iter_mut() {
        stun.timer.tick(time.delta());
        if stun.timer.finished() {
            if let Some(mut ec) = commands.get_entity(e) { ec.remove::<StunDebuff>(); }
        }
    }
    for (e, tf, mut burn, mut hp, resist) in debuffs.p0().iter_mut() {
        burn.timer.tick(time.delta());
        burn.tick.tick(time.delta());
        if burn.tick.just_finished() {
            let dmg = burn.dps * 0.5 * resist.map_or(1.0, |r| r.dmg_mul[4]); // 4 = Fire
            hp.0 -= dmg;
            let jitter = Vec2::new(rand::thread_rng().gen_range(-6.0..6.0), rand::thread_rng().gen_range(0.0..8.0));
            spawn_damage_number(&mut commands, &mut pool, dmg, Color::srgb(1.0, 0.4, 0.1), Vec2::new(tf.translation.x + jitter.x, tf.translation.y + jitter.y + 18.0), 25.0, 0.7);
        }
        if burn.timer.finished() {
            if let Some(mut ec) = commands.get_entity(e) { ec.remove::<BurnDebuff>(); }
        }
    }
    for (e, tf, mut poison, mut hp, resist) in debuffs.p1().iter_mut() {
        poison.timer.tick(time.delta());
        poison.tick.tick(time.delta());
        if poison.tick.just_finished() {
            let dmg = poison.dps * 0.5 * resist.map_or(1.0, |r| r.dmg_mul[5]); // 5 = Poison
            hp.0 -= dmg;
            let jitter = Vec2::new(rand::thread_rng().gen_range(-6.0..6.0), rand::thread_rng().gen_range(0.0..8.0));
            spawn_damage_number(&mut commands, &mut pool, dmg, Color::srgb(0.4, 0.9, 0.3), Vec2::new(tf.translation.x + jitter.x, tf.translation.y + jitter.y + 18.0), 25.0, 0.7);
        }
        if poison.timer.finished() {
            if let Some(mut ec) = commands.get_entity(e) { ec.remove::<PoisonDebuff>(); }
        }
    }
}

pub fn update_damage_numbers(
    time: Res<Time>,
    mut pool: ResMut<DamageNumberPool>,
    mut q: Query<(Entity, &mut Transform, &mut DamageNumber, &mut TextColor, &mut Visibility)>,
) {
    for (ent, mut tf, mut dmg, mut col, mut vis) in q.iter_mut() {
        if !dmg.active { continue; } // volne cislo z poolu - preskocit (zadna zmena = zadny change detection)
        dmg.timer.tick(time.delta());
        tf.translation.y += dmg.vel.y * time.delta_secs();
        tf.translation.x += dmg.vel.x * time.delta_secs();
        let c = col.0.to_srgba();
        col.0 = Color::srgba(c.red, c.green, c.blue, dmg.timer.fraction_remaining());
        if dmg.timer.finished() {
            // zpet do poolu: primo (bez commands), aby nevznikl zavod s dalsim spawnem ve stejnem snimku
            dmg.active = false;
            *vis = Visibility::Hidden;
            pool.free.push(ent);
        }
    }
}

const DAMAGE_POOL_SIZE: usize = 150; // max cisel naraz; kdyz dojdou, dalsi se zahodi (ochrana FPS na mobilu)

// Startup: predvytvori skryte entity s textem
pub fn setup_damage_pool(mut commands: Commands, mut pool: ResMut<DamageNumberPool>) {
    for _ in 0..DAMAGE_POOL_SIZE {
        let e = commands.spawn((
            Text2d::new(""),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, 0.0, 15.0),
            Visibility::Hidden,
            DamageNumber { timer: Timer::from_seconds(0.7, TimerMode::Once), vel: Vec2::ZERO, active: false },
        )).id();
        pool.free.push(e);
    }
}

// Vezme volnou entitu z poolu a "zapne" ji. Kdyz je pool prazdny, nic se nestane.
const POOL_BUSY_FREE: usize = 20;    // kdyz je v poolu volnych mene nez tolik...
const POOL_BUSY_MIN_DMG: f32 = 10.0; // ...ukazuji se jen zasahy >= tolik (ladit podle hry)
pub fn spawn_damage_number(commands: &mut Commands, pool: &mut DamageNumberPool, dmg: f32, color: Color, pos: Vec2, vel_y: f32, secs: f32) {
    if pool.free.len() < POOL_BUSY_FREE && dmg < POOL_BUSY_MIN_DMG { return; }
    let Some(e) = pool.free.pop() else { return; };
    commands.entity(e).insert((
        Text2d::new(format!("-{:.0}", dmg)),
        TextColor(color),
        Transform::from_xyz(pos.x, pos.y, 15.0),
        Visibility::Visible,
        DamageNumber { timer: Timer::from_seconds(secs, TimerMode::Once), vel: Vec2::new(0.0, vel_y), active: true },
    ));
}
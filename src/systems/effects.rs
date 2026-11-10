use bevy::prelude::*;
use rand::Rng;
use std::collections::HashMap;
use crate::components::{Tower, Creep, SlowDebuff, BurnDebuff, PoisonDebuff};

// =====================================================================
// TABULKY EFEKTU - vsechno k ladeni je tady nahore
// =====================================================================

const FROST_KIND: u8 = 3;                          // druh veze Frost (viz tower_def.rs)
const FROST_TARGETS: [usize; 5] = [3, 4, 5, 6, 7]; // kolik jednotek freeznou veze Frost level 1..5
const FREEZE_SECS: f32 = 2.5;                      // delka freeze (kruh se za tu dobu zmensi 100 % -> 0 %)
const PARTICLE_POOL_SIZE: usize = 80;              // max castic naraz (ochrana FPS na mobilu)

// intenzita castic podle levelu veze 1..5: prumerne sekundy mezi dvema casticemi na jednom kolecku (mensi = vic)
// sila a delka samotne dotky (dps, sekundy) jsou v tower_def.rs - tady jen vzhled
const FLAME_INTERVAL: [f32; 5] = [0.60, 0.45, 0.35, 0.25, 0.15]; // Fire (plaminky)
const DROP_INTERVAL: [f32; 5] = [1.20, 1.00, 0.80, 0.60, 0.40];  // Poison (kapky)

#[derive(Clone, Copy)]
enum ParticleKind { Flame, Drop }

struct ParticleDef {
    color: Color,
    size: Vec2,
    interval: f32, // prumerne kolik sekund mezi dvema casticemi na jednom kolecku
    life: f32,     // jak dlouho castice zije (sekundy)
    vel: Vec2,     // rychlost pohybu castice (jednotek/s)
}

fn particle_def(kind: ParticleKind, level: u32) -> ParticleDef {
    let i = (level as usize).clamp(1, 5) - 1;
    match kind {
        // plaminek: na kolecku se random objevi a stoupa nahoru
        ParticleKind::Flame => ParticleDef { color: Color::WHITE, size: Vec2::new(14.0, 18.0), interval: FLAME_INTERVAL[i], life: 0.5, vel: Vec2::new(0.0, 24.0) },
        // kapicka jedu: jednou za cas, pomalu steka dolu
        ParticleKind::Drop => ParticleDef { color: Color::WHITE, size: Vec2::new(12.0, 16.0), interval: DROP_INTERVAL[i], life: 0.7, vel: Vec2::new(0.0, -14.0) },
    }
}

// =====================================================================
// CASTICE (pool = vsechny entity vytvorene jednou na startu, zadny spawn/despawn za behu)
// =====================================================================

#[derive(Component)]
pub struct Particle {
    follow: Entity, // kolecko, ke kteremu castice patri (jede s nim)
    offset: Vec2,   // pozice vuci kolecku
    vel: Vec2,
    timer: Timer,
    base: Color,
    active: bool,   // false = volny slot v poolu
}

impl Particle {
    fn idle() -> Self {
        Self { follow: Entity::PLACEHOLDER, offset: Vec2::ZERO, vel: Vec2::ZERO, timer: Timer::from_seconds(1.0, TimerMode::Once), base: Color::WHITE, active: false }
    }
}

#[derive(Resource, Default)]
pub struct ParticlePool {
    pub free: Vec<Entity>,
    flame: Handle<Image>, // assets/effects/ohen.png
    drop: Handle<Image>,  // assets/effects/jed.png
}

// Startup: nacte obrazky a predvytvori skryte castice
pub fn setup_particle_pool(mut commands: Commands, mut pool: ResMut<ParticlePool>, asset_server: Res<AssetServer>) {
    pool.flame = asset_server.load("effects/ohen.png");
    pool.drop = asset_server.load("effects/jed.png");
    for _ in 0..PARTICLE_POOL_SIZE {
        let e = commands.spawn((
            Sprite { color: Color::srgba(1.0, 1.0, 1.0, 0.0), custom_size: Some(Vec2::new(6.0, 6.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 9.0),
            Visibility::Hidden,
            Particle::idle(),
        )).id();
        pool.free.push(e);
    }
}

// =====================================================================
// FREEZE - veze Frost pri kazdem vystrelu zpomali N nejblizsich jednotek, ktere jeste slow nemaji
// =====================================================================

pub fn freeze_system(
    mut commands: Commands,
    mut seen: Local<HashMap<Entity, u32>>,   // posledni videny hit_count veze (zmena = veze vystrelila)
    mut cand: Local<Vec<(Entity, f32)>>,     // znovupouzity buffer kandidatu (zadna alokace za snimek)
    towers: Query<(Entity, &Transform, &Tower), Without<Creep>>,
    creeps: Query<(Entity, &Transform, Has<SlowDebuff>), (With<Creep>, Without<Tower>)>,
    resists: Query<&crate::components::Resist>,
) {
    for (t_ent, t_tf, tower) in towers.iter() {
        if tower.kind != FROST_KIND { continue; }
        let last = seen.entry(t_ent).or_insert(tower.hit_count);
        if *last == tower.hit_count { continue; }
        *last = tower.hit_count;

        let t_pos = t_tf.translation.truncate();
        let range2 = tower.range * tower.range;
        cand.clear();
        for (c_ent, c_tf, slowed) in creeps.iter() {
            if slowed { continue; } // uz zpomalene preskocit -> cil je dalsi v rade
            let d2 = t_pos.distance_squared(c_tf.translation.truncate());
            if d2 < range2 { cand.push((c_ent, d2)); }
        }
        cand.sort_by(|a, b| a.1.total_cmp(&b.1)); // nejblizsi veze prvni

        let n = FROST_TARGETS[(tower.level as usize).clamp(1, 5) - 1];
        let factor = 1.0 - tower.slow_per_hit.clamp(0.0, tower.slow_max);
        for &(c_ent, _) in cand.iter().take(n) {
            // get_entity: creep mohl byt mezitim despawnuty
            if let Some(mut ec) = commands.get_entity(c_ent) {
                let f = 1.0 - (1.0 - factor) * (1.0 - resists.get(c_ent).map_or(0.0, |r| r.slow)); // odolnost zmensi silu slow (Ogre 20 %)
                ec.insert(SlowDebuff { timer: Timer::from_seconds(FREEZE_SECS, TimerMode::Once), factor: f });
            }
        }
    }
}

// =====================================================================
// OHEN A JED - nahodne castice na koleckach s debuffem (bez stavu na creepovi, jen pravdepodobnost za snimek)
// =====================================================================

pub fn particle_spawn_system(
    time: Res<Time>,
    mut pool: ResMut<ParticlePool>,
    creeps: Query<(Entity, &Creep, Option<&BurnDebuff>, Option<&PoisonDebuff>)>,
    mut q: Query<(&mut Particle, &mut Sprite, &mut Visibility)>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::thread_rng();
    for (c_ent, creep, burn, poison) in creeps.iter() {
        for (level, kind) in [(burn.map(|b| b.level), ParticleKind::Flame), (poison.map(|p| p.level), ParticleKind::Drop)] {
            let Some(level) = level else { continue; };
            let def = particle_def(kind, level);
            if rng.gen::<f32>() >= dt / def.interval { continue; }
            let image = match kind { ParticleKind::Flame => pool.flame.clone(), ParticleKind::Drop => pool.drop.clone() };
            let Some(p_ent) = pool.free.pop() else { return; }; // pool prazdny -> zadne dalsi castice
            let Ok((mut p, mut sprite, mut vis)) = q.get_mut(p_ent) else { continue; };
            let r = creep.base_radius;
            p.follow = c_ent;
            p.offset = Vec2::new(rng.gen_range(-r * 0.7..r * 0.7), rng.gen_range(-r * 0.2..r * 0.8));
            p.vel = def.vel;
            p.timer = Timer::from_seconds(def.life, TimerMode::Once);
            p.base = def.color;
            p.active = true;
            sprite.image = image;
            sprite.color = def.color;
            sprite.custom_size = Some(def.size);
            *vis = Visibility::Visible;
        }
    }
}

// pohyb + mizeni castic, po dozivoti zpet do poolu
pub fn particle_update_system(
    time: Res<Time>,
    mut pool: ResMut<ParticlePool>,
    creeps: Query<&Transform, (With<Creep>, Without<Particle>)>,
    mut q: Query<(Entity, &mut Transform, &mut Particle, &mut Sprite, &mut Visibility), Without<Creep>>,
) {
    let dt = time.delta_secs();
    for (ent, mut tf, mut p, mut sprite, mut vis) in q.iter_mut() {
        if !p.active { continue; } // volny slot z poolu
        p.timer.tick(time.delta());
        let follow = p.follow;
        let Ok(c_tf) = creeps.get(follow) else {
            // creep uz neexistuje
            p.active = false; *vis = Visibility::Hidden; pool.free.push(ent);
            continue;
        };
        if p.timer.finished() {
            p.active = false; *vis = Visibility::Hidden; pool.free.push(ent);
            continue;
        }
        let v = p.vel;
        let o = p.offset + v * dt;
        p.offset = o;
        tf.translation.x = c_tf.translation.x + o.x;
        tf.translation.y = c_tf.translation.y + o.y;
        let c = p.base.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, p.timer.fraction_remaining());
    }
}

// =====================================================================
// STUN podle druhu veze a levelu 1..5: (sance, sekundy) - zatim jen Shock Turret
// =====================================================================

const SHOCK_KIND: u8 = 7;
const SHOCK_STUN: [(f32, f32); 5] = [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.25, 1.0)];

pub fn stun_for(kind: u8, level: u32) -> (f32, f32) {
    if kind == SHOCK_KIND { SHOCK_STUN[(level as usize).clamp(1, 5) - 1] } else { (0.0, 0.0) }
}

// =====================================================================
// CHAIN BLESK - svetla car mezi zasazenymi creepy (pool car, zadny spawn/despawn za behu)
// =====================================================================

const BOLT_POOL_SIZE: usize = 40;   // max car naraz
const BOLT_SECS: f32 = 0.18;        // jak dlouho car svieti (pak mizi)
const BOLT_THICKNESS: f32 = 3.0;    // tloustka cary
fn bolt_color() -> Color { Color::srgb(0.85, 0.95, 1.0) } // svetle modrobila

#[derive(Component)]
pub struct Bolt { timer: Timer, active: bool }

#[derive(Resource, Default)]
pub struct BoltPool { pub free: Vec<Entity> }

// Startup: predvytvori skryte cary
pub fn setup_bolt_pool(mut commands: Commands, mut pool: ResMut<BoltPool>) {
    for _ in 0..BOLT_POOL_SIZE {
        let e = commands.spawn((
            Sprite { color: Color::srgba(1.0, 1.0, 1.0, 0.0), custom_size: Some(Vec2::new(1.0, BOLT_THICKNESS)), ..default() },
            Transform::from_xyz(0.0, 0.0, 8.0),
            Visibility::Hidden,
            Bolt { timer: Timer::from_seconds(BOLT_SECS, TimerMode::Once), active: false },
        )).id();
        pool.free.push(e);
    }
}

// car z bodu a do bodu b (vola se z bullet_hit); kdyz je pool prazdny, nic se nestane
pub fn spawn_bolt(commands: &mut Commands, pool: &mut BoltPool, a: Vec2, b: Vec2) {
    let Some(e) = pool.free.pop() else { return; };
    let d = b - a;
    let mid = (a + b) * 0.5;
    commands.entity(e).insert((
        Sprite { color: bolt_color(), custom_size: Some(Vec2::new(d.length().max(1.0), BOLT_THICKNESS)), ..default() },
        Transform::from_xyz(mid.x, mid.y, 8.0).with_rotation(Quat::from_rotation_z(d.y.atan2(d.x))),
        Visibility::Visible,
        Bolt { timer: Timer::from_seconds(BOLT_SECS, TimerMode::Once), active: true },
    ));
}

// car pomalu bledne a po dozivoti se vrati do poolu
pub fn bolt_update_system(
    time: Res<Time>,
    mut pool: ResMut<BoltPool>,
    mut q: Query<(Entity, &mut Bolt, &mut Sprite, &mut Visibility)>,
) {
    for (ent, mut bolt, mut sprite, mut vis) in q.iter_mut() {
        if !bolt.active { continue; } // volny slot z poolu
        bolt.timer.tick(time.delta());
        if bolt.timer.finished() {
            bolt.active = false; *vis = Visibility::Hidden; pool.free.push(ent);
            continue;
        }
        let c = bolt_color().to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, bolt.timer.fraction_remaining());
    }
}

// =====================================================================
// IGNITED - horici creep ma pulzujici oranzove telo (rgba(1.0, 0.3, 0.1, 1.0 + sin * 0.2)), po dohoreni se vrati puvodni barva
// =====================================================================

const IGNITE_RGB: (f32, f32, f32) = (1.0, 0.3, 0.1); // barva ohne
const IGNITE_PULSE_SPEED: f32 = 12.0;                // rychlost pulzovani
const IGNITE_PULSE_AMOUNT: f32 = 0.2;                // sila pulzovani (alfa 1.0 +- tolik, nahore se orizne na 1.0)

#[derive(Component)]
pub struct Ignited { base: Color } // puvodni barva tela creepa

pub fn ignite_visual_system(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    // zacatek horeni: creep s BurnDebuff, ktery jeste neni Ignited -> zapamatovat puvodni barvu
    q_start: Query<(Entity, Option<&MeshMaterial2d<ColorMaterial>>, Option<&Sprite>), (With<Creep>, With<BurnDebuff>, Without<Ignited>)>,
    // behem horeni pulzovat, po dohoreni vratit barvu
    mut q_burning: Query<(Entity, &Ignited, Option<&MeshMaterial2d<ColorMaterial>>, Option<&mut Sprite>, Has<BurnDebuff>), With<Creep>>,
) {
    for (e, mat, sprite) in q_start.iter() {
        let base = match (mat, sprite) {
            (Some(m), _) => materials.get(&m.0).map(|m| m.color),
            (None, Some(s)) => Some(s.color),
            _ => None,
        };
        if let Some(base) = base { commands.entity(e).insert(Ignited { base }); }
    }
    let alpha = (1.0 + (time.elapsed_secs() * IGNITE_PULSE_SPEED).sin() * IGNITE_PULSE_AMOUNT).min(1.0);
    let fire = Color::srgba(IGNITE_RGB.0, IGNITE_RGB.1, IGNITE_RGB.2, alpha);
    for (e, ignited, mat, sprite, burning) in q_burning.iter_mut() {
        let color = if burning { fire } else { ignited.base };
        if let Some(m) = mat { if let Some(m) = materials.get_mut(&m.0) { m.color = color; } }
        if let Some(mut s) = sprite { s.color = color; }
        if !burning { commands.entity(e).remove::<Ignited>(); }
    }
}
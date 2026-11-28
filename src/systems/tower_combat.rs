
use bevy::prelude::*;
use std::collections::HashMap;
use rand::Rng;
use crate::components::{Tower, Creep, Bullet, Air, Side};
use crate::constants::TOWER_POISON;

// =====================================================================
// TABULKA UTOKU - jak vypada a leti strela kazde veze (kind viz tower_def.rs)
// =====================================================================

struct AttackDef {
    speed: f32,           // rychlost strely (jednotek/s); zivot strely se pocita z dosahu veze
    size: Vec2,           // velikost strely na levelu 1
    size_per_level: f32,  // o kolik (podil) strela naroste za kazdy dalsi level
    color: Color,         // barva strely (kdyz nema obrazek)
    image: Option<&'static str>, // obrazek strely, None = barevny ctverecek
    rot_offset: f32,      // natoceni obrazku (rad): 0 = obrazek miri doprava, -PI/2 = miri nahoru
    hit_radius: f32,      // jak blizko creepa musi strela byt, aby zasahla
    slow: bool,           // strela zpomaluje zasazeny cil (Frost zpomaluje pres freeze_system, ne strelou)
    chain_radius: f32,    // chain: jak daleko muze preskocit na dalsiho creepa
    chain_falloff: f32,   // chain: nasobek damage za kazdy preskok (0.75 = o 25 % mene)
    push_speed: f32,      // rychlost odhozeni creepa dozadu (jednotek/s)
}

fn hits_air(_kind: u8, _level: u32) -> bool {
    true
}

// odolnost a damage typ zasazeneho creepa (kopie, aby se nemusely drzet reference do dotazu)
fn resist_of(q: &Query<(Option<&crate::components::Resist>, Option<&crate::components::DmgTarget>)>, e: Entity) -> Option<crate::components::Resist> { q.get(e).ok().and_then(|(r, _)| r.copied()) }
fn target_of(q: &Query<(Option<&crate::components::Resist>, Option<&crate::components::DmgTarget>)>, e: Entity) -> Option<crate::components::DmgTarget> { q.get(e).ok().and_then(|(_, t)| t.copied()) }

// damage typ veze proti cili (GDD 5): silny zasah 125 %, slaby 75 %
fn resist_mul(t: Option<crate::components::DmgTarget>, kind: u8) -> f32 {
    t.map_or(1.0, |t| crate::wave_def::damage_mul(kind, &t.0))
}

fn attack_def(kind: u8) -> AttackDef {
    use std::f32::consts::FRAC_PI_2;
    match kind {
        // Bramboromet
        1 => AttackDef { speed: 600.0, size: Vec2::new(14.0, 28.0), size_per_level: 0.10, color: Color::srgb(1.0, 1.0, 0.8), image: Some("bullets/sip.png"), rot_offset: -FRAC_PI_2, hit_radius: 24.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Nerfka
        2 => AttackDef { speed: 380.0, size: Vec2::new(18.0, 18.0), size_per_level: 0.15, color: Color::srgb(1.0, 0.6, 0.1), image: Some("bullets/cannon.png"), rot_offset: 0.0, hit_radius: 30.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Babicka
        3 => AttackDef { speed: 450.0, size: Vec2::new(9.0, 9.0), size_per_level: 0.10, color: Color::srgb(0.2, 0.6, 1.0), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Router
        4 => AttackDef { speed: 420.0, size: Vec2::new(9.0, 9.0), size_per_level: 0.12, color: Color::srgb(1.0, 0.3, 0.1), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Kaktus
        5 => AttackDef { speed: 400.0, size: Vec2::new(8.0, 8.0), size_per_level: 0.10, color: Color::srgb(0.3, 0.9, 0.3), image: None, rot_offset: 0.0, hit_radius: 24.0, slow: true, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Toustovac
        6 => AttackDef { speed: 900.0, size: Vec2::new(8.0, 8.0), size_per_level: 0.10, color: Color::srgb(0.3, 0.4, 1.0), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Muchomurka
        7 => AttackDef { speed: 800.0, size: Vec2::new(10.0, 10.0), size_per_level: 0.10, color: Color::srgb(0.95, 0.9, 0.3), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        8 => AttackDef { speed: 700.0, size: Vec2::new(6.0, 22.0), size_per_level: 0.10, color: Color::srgb(0.2, 0.9, 0.9), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        10 => AttackDef { speed: 500.0, size: Vec2::new(14.0, 14.0), size_per_level: 0.10, color: Color::srgb(0.7, 0.1, 0.1), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        11 => AttackDef { speed: 450.0, size: Vec2::new(9.0, 9.0), size_per_level: 0.10, color: Color::srgb(0.75, 0.35, 0.75), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        12 => AttackDef { speed: 450.0, size: Vec2::new(9.0, 9.0), size_per_level: 0.10, color: Color::srgb(0.55, 0.2, 0.85), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        13 => AttackDef { speed: 420.0, size: Vec2::new(10.0, 10.0), size_per_level: 0.10, color: Color::srgb(1.0, 0.65, 0.9), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        14 => AttackDef { speed: 600.0, size: Vec2::new(8.0, 8.0), size_per_level: 0.10, color: Color::srgb(0.9, 0.8, 0.6), image: None, rot_offset: 0.0, hit_radius: 24.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        _ => AttackDef { speed: 550.0, size: Vec2::new(8.0, 8.0), size_per_level: 0.0, color: Color::srgb(1.0, 1.0, 0.8), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
    }
}

pub fn tower_shoot(
    mut commands: Commands,
    time: Res<Time>,
    wave: Res<crate::resources::WaveCount>,
    asset_server: Res<AssetServer>,
    mut bullet_pool: ResMut<crate::resources::BulletPool>,
    mut tex_cache: Local<HashMap<&'static str, Handle<Image>>>, // textury strel nacist jen jednou
    mut towers: Query<(&Transform, &mut Tower, Option<&crate::components::TowerSlow>, Option<&crate::components::TowerMods>), (With<Tower>, Without<Creep>)>,
    creeps: Query<(&Transform, Has<Air>), (With<Creep>, Without<Tower>)>,
    mut fx: ResMut<crate::fx::FxAssets>,
    mut glow_mats: ResMut<Assets<crate::fx::GlowMaterial>>,
) {
    let mut rng = rand::thread_rng();
    for (t_tf, mut tower, t_slow, mods) in towers.iter_mut() {
        tower.cooldown -= time.delta_secs();
        if tower.cooldown > 0.0 { continue; }
        if mods.map_or(false, |m| m.stun_left > 0.0) { continue; } // omracena vez (stomp Pit Lorda) nestrili
        if !crate::special::shoots(tower.kind) { continue; } // pasivni veze (Mint, Bell, Geyser...) maji vlastni system
        let t_pos = t_tf.translation.truncate();
        let mut target: Option<Vec2> = None;
        let mut best_dist = f32::MAX;
        let can_air = hits_air(tower.kind, tower.level);
        let range = tower.range * mods.map_or(1.0, |m| m.range_mul); // aura creepu muze zkratit dosah
        let mut random_pick: Option<Vec2> = None; // nahodny cil v dosahu (aura Infernal)
        let mut in_range = 0u32;
        for (c_tf, is_air) in creeps.iter() {
            if is_air && !can_air { continue; } // zemni vez nema na letajici cil
            let d = t_pos.distance(c_tf.translation.truncate());
            if d < range {
                in_range += 1;
                if rng.gen_range(0..in_range) == 0 { random_pick = Some(c_tf.translation.truncate()); } // nahodny vyber z cilu v dosahu
                if d < best_dist { best_dist = d; target = Some(c_tf.translation.truncate()); }
            }
        }
        if random_pick.is_some() && mods.map_or(0.0, |m| m.random_target) > rng.gen::<f32>() { target = random_pick; } // sance na nahodny cil (Infernal)
        let Some(target) = target else { continue; };
        let dir = (target - t_pos).normalize_or_zero();
        if dir == Vec2::ZERO { continue; }
        let Some(b_ent) = bullet_pool.free.pop() else { continue; }; // pool prazdny -> cooldown se nespotrebuje, zkusi znovu

        let atk = attack_def(tower.kind);
        let size = atk.size * (1.0 + atk.size_per_level * (tower.level as f32 - 1.0));
        let angle = if atk.image.is_some() { dir.y.atan2(dir.x) + atk.rot_offset } else { 0.0 }; // strela leti rovne, natoceni staci nastavit jednou
        let mut shot_damage = (tower.damage_min + tower.damage_max) * 0.5 * mods.map_or(1.0, |m| m.dmg_mul);
        if tower.kind == crate::special::KATAPULT && tower.level >= 7 && rng.gen::<f32>() < 0.20 {
            shot_damage *= 2.0;
        }
        // strela s obrazkem = sprite; bez obrazku = zarici shader (ctverec s glow materialem, zare 3x vetsi nez strela)
        let mut tf = Transform::from_xyz(t_tf.translation.x, t_tf.translation.y, 6.0).with_rotation(Quat::from_rotation_z(angle));
        match atk.image {
            Some(path) => {
                let sprite = Sprite { image: tex_cache.entry(path).or_insert_with(|| asset_server.load(path)).clone(), custom_size: Some(size), ..default() };
                commands.entity(b_ent).remove::<(Mesh2d, MeshMaterial2d<crate::fx::GlowMaterial>)>().insert(sprite);
            }
            None => {
                let s = (size.x.max(size.y) * 5.0).max(30.0); // zare je vetsi nez samotna strela
                tf.scale = Vec3::new(s, s, 1.0);
                let mat = fx.glow_for(tower.kind, atk.color, &mut glow_mats);
                commands.entity(b_ent).remove::<Sprite>().insert((Mesh2d(fx.quad.clone()), MeshMaterial2d(mat)));
            }
        }
        commands.entity(b_ent).insert((
            Visibility::Visible,
            tf,
            Bullet {
                damage: shot_damage,
                shot_count: tower.hit_count + 1,
                owner_side: Some(tower.side),
                owner_row: tower.row as u8,
                owner_col: tower.col as u8,
                wave: wave.0,
                vel: dir * atk.speed,
                life: Timer::from_seconds(range / atk.speed + 0.2, TimerMode::Once),
                aoe: tower.splash,
                kind: tower.kind,
                level: tower.level,
                slow: if atk.slow || tower.slow_per_hit > 0.0 { tower.slow_per_hit } else { 0.0 },
                slow_max: tower.slow_max,
                push_chance: tower.push_chance,
                push_dist: tower.push_dist,
                chain_left: tower.chain_targets,
                dot_dps: tower.dot_dps,
                dot_dur: tower.dot_duration,
                hit_radius: atk.hit_radius,
                hits_air: hits_air(tower.kind, tower.level),
                chain_radius: atk.chain_radius,
                chain_falloff: atk.chain_falloff,
                push_speed: atk.push_speed,
                active: true,
            },
        ));
        tower.cooldown = 1.0 / (tower.fire_rate * t_slow.map_or(1.0, |s| s.factor) * mods.map_or(1.0, |m| m.rate_mul)); // zpomalena vez (Frost Archer, aura Final Bosse) strili pomaleji
        tower.hit_count += 1;
    }
}

pub fn move_bullets(
    mut q: Query<(&mut Transform, &mut Bullet)>,
    time: Res<Time>,
) {
    for (mut tf, mut b) in q.iter_mut() {
        if !b.active { continue; } // volny slot z poolu
        tf.translation.x += b.vel.x * time.delta_secs();
        tf.translation.y += b.vel.y * time.delta_secs();
        b.life.tick(time.delta());
    }
}

// odhozeni dozadu (Knockback) a omraceni (Stun) zasazeneho creepa - sance z tabulky veze
fn apply_push_stun(commands: &mut Commands, e: Entity, bullet: &Bullet, rng: &mut impl Rng, resist: Option<&crate::components::Resist>) {
    let (stun_chance, stun_secs) = crate::systems::effects::stun_for(bullet.kind, bullet.level);
    let Some(mut ec) = commands.get_entity(e) else { return; }; // creep mohl byt mezitim despawnuty
    if bullet.push_chance > 0.0 && rng.gen::<f32>() < bullet.push_chance {
        ec.try_insert(crate::components::Knockback { remaining: bullet.push_dist * (1.0 - resist.map_or(0.0, |r| r.push)), speed: bullet.push_speed }); // resist 0.5 = odhozeni o pulku mene
    }
    if stun_chance > 0.0 && !resist.map_or(false, |r| r.stun_immune) && rng.gen::<f32>() < stun_chance {
        ec.try_insert(crate::components::StunDebuff { timer: Timer::from_seconds(stun_secs, TimerMode::Once) });
    }
}

fn add_frame_dmg(map: &mut HashMap<Entity, (Vec2, f32, Color)>, e: Entity, pos: Vec2, dmg: f32, col: Color) {
    map.entry(e).or_insert((pos, 0.0, col)).1 += dmg;
}

fn dj_drop_ready(timer: &mut f32, delta_secs: f32) -> bool {
    if *timer > delta_secs {
        *timer -= delta_secs;
        false
    } else {
        *timer = 10.0;
        true
    }
}

pub fn dj_drop_system(
    mut commands: Commands,
    grid: Res<crate::resources::CreepGrid>,
    time: Res<Time>,
    paused: Res<crate::resources::WavePaused>,
    mut towers: Query<&mut Tower>,
    mut creeps: Query<(&mut crate::components::Health, &Transform), With<Creep>>,
    mut pool: ResMut<crate::resources::DamageNumberPool>,
) {
    if paused.0 { return; }
    let mut near: Vec<(Entity, Vec2)> = Vec::new();
    let mut frame_dmg: HashMap<Entity, (Vec2, f32, Color)> = HashMap::new();
    for mut tower in towers.iter_mut() {
        if tower.kind != crate::special::DJ || !dj_drop_ready(&mut tower.dj_drop_timer, time.delta_secs()) { continue; }
        near.clear();
        grid.query(
            Vec2::new(tower.side.lane_x(), crate::constants::LANE_Y),
            crate::constants::LANE_H + crate::constants::SPAWN_ABOVE,
            &mut near,
        );
        for &(entity, pos) in near.iter() {
            if (pos.x - tower.side.lane_x()).abs() > crate::constants::LANE_W * 0.5 { continue; }
            if let Ok((mut hp, _)) = creeps.get_mut(entity) {
                hp.0 -= 100.0;
                add_frame_dmg(&mut frame_dmg, entity, pos, 100.0, Color::srgb(0.8, 0.3, 0.9));
                if let Some(mut ec) = commands.get_entity(entity) {
                    ec.try_insert(crate::components::Knockback { remaining: 3.0 * crate::constants::CELL, speed: 500.0 });
                }
            }
        }
    }
    for (_, (pos, dmg, col)) in frame_dmg {
        crate::systems::debuff::spawn_damage_number(&mut commands, &mut pool, dmg, col, Vec2::new(pos.x, pos.y + 18.0), 25.0, 0.7);
    }
}

pub fn bullet_hit(
    mut commands: Commands,
    mut pool: ResMut<crate::resources::DamageNumberPool>,
    mut bullet_pool: ResMut<crate::resources::BulletPool>,
    mut bolt_pool: ResMut<crate::systems::effects::BoltPool>,
    grid: Res<crate::resources::CreepGrid>,
    mut bullets: Query<(Entity, &Transform, &mut Bullet, &mut Visibility), (Without<Creep>, Without<Tower>)>,
    mut creeps: Query<(&mut crate::components::Health, &mut crate::components::MaxHealth, &mut Creep, Option<&crate::components::Cursed>), With<Creep>>,
    mut creep_positions: Query<&mut Transform, (With<Creep>, Without<Tower>)>,
    mut creep_sides: Query<&mut Side, (With<Creep>, Without<Tower>)>,
    mut poisons: Query<&mut crate::components::PoisonDebuff, With<Creep>>,
    mut tower_abilities: Query<&mut Tower, Without<Creep>>,
    air: Query<(), With<Air>>,
    creep_defs: Query<(Option<&crate::components::Resist>, Option<&crate::components::DmgTarget>)>, // odolnosti a damage typy
    time: Res<Time>,
    mut waves: ResMut<crate::fx::ShockwavePool>,
    mut wave_mats: ResMut<Assets<crate::fx::ShockwaveMaterial>>,
) {
    use crate::special::*;
    let now = time.elapsed_secs_wrapped();
    let mut frame_dmg: HashMap<Entity, (Vec2, f32, Color)> = HashMap::new(); // 1 cislo na creepa za snimek
    let mut near: Vec<(Entity, Vec2)> = Vec::new(); // znovupouzity buffer - zadna alokace na strelu
    let mut rng = rand::thread_rng();
    let mut chained: Vec<Entity> = Vec::new(); // kdo uz byl v chainu zasazen (znovupouzity buffer)
    for (b_ent, b_tf, mut bullet, mut b_vis) in bullets.iter_mut() {
        if !bullet.active { continue; } // volny slot z poolu
        if bullet.life.finished() { bullet.active = false; *b_vis = Visibility::Hidden; bullet_pool.free.push(b_ent); continue; }
        let b_pos = b_tf.translation.truncate();
        // zasah: nejblizsi creep do hit_radius - hleda se jen v okoli (prostorova mrizka)
        near.clear();
        grid.query(b_pos, bullet.hit_radius, &mut near);
        let Some(&(hit_ent, hit_pos)) = near.iter().filter(|(e, _)| bullet.hits_air || !air.contains(*e)).min_by(|a, b| a.1.distance_squared(b_pos).total_cmp(&b.1.distance_squared(b_pos))) else { continue; };
        if bullet.aoe > 0.0 {
            crate::fx::spawn_shockwave(&mut commands, &mut waves, &mut wave_mats, now, hit_pos, bullet.aoe, attack_def(bullet.kind).color, 0.35); // vybuch
            near.clear();
            grid.query(hit_pos, bullet.aoe, &mut near);
            for &(c_ent, c_pos) in near.iter() {
                if !bullet.hits_air && air.contains(c_ent) { continue; } // splash zemni veze letajici nezasahne
                if let Ok((mut hp, _, _, cursed)) = creeps.get_mut(c_ent) {
                    let dmg = bullet.damage * resist_mul(target_of(&creep_defs, c_ent), bullet.kind) * (1.0 + cursed.map_or(0.0, |c| c.mul)); // odolnost + kletba
                    hp.0 -= dmg;
                    add_frame_dmg(&mut frame_dmg, c_ent, c_pos, dmg, Color::srgb(1.0, 0.4, 0.1));
                }
            }
        } else if let Ok((mut hp, mut max_hp, mut creep, cursed)) = creeps.get_mut(hit_ent) {
            let (reduction_flag, reduction) = match bullet.kind {
                CHUCK_NORRIS if bullet.level >= 3 && creep.is_boss => (1, 0.20),
                TCHYNE => (2, 0.20),
                KAREN if bullet.level >= 3 && creep.is_boss => (4, 0.10),
                _ => (0, 0.0),
            };
            if reduction_flag != 0 && creep.max_hp_reductions & reduction_flag == 0 {
                max_hp.0 *= 1.0 - reduction;
                hp.0 = hp.0.min(max_hp.0);
                creep.max_hp_reductions |= reduction_flag;
            }
            let mut dmg = bullet.damage * resist_mul(target_of(&creep_defs, hit_ent), bullet.kind) * (1.0 + cursed.map_or(0.0, |c| c.mul)); // odolnost + kletba
            let mut col = Color::srgb(1.0, 0.4, 0.1);
            if bullet.kind == CHUCK_NORRIS {
            dmg = bullet.damage * resist_mul(target_of(&creep_defs, hit_ent), bullet.kind); // cisty dmg (Mama ve 3. fazi +20 %)
            if hp.0 / max_hp.0.max(1.0) < 0.30 { dmg = hp.0; }
            col = Color::srgb(0.95, 0.85, 0.5);
            } else if bullet.kind == SNIPER {
            let (chance, multiplier) = if bullet.level >= 7 { (0.40, 4.0) } else { (0.20, 3.0) };
            if rng.gen::<f32>() < chance { dmg *= multiplier; }
            if !creep.is_boss && hp.0 / max_hp.0.max(1.0) < 0.20 { dmg = hp.0; }
            } else if bullet.kind == BABICKA && bullet.shot_count % 6 == 0 {
            dmg *= 2.0;
            col = Color::srgb(0.95, 0.75, 0.25);
            } else if bullet.kind == TCHYNE && rng.gen::<f32>() < 0.20 {
            dmg *= 2.0;
            col = Color::srgb(0.9, 0.3, 0.65);
            } else if bullet.kind == BANHAMMER && bullet.shot_count % 5 == 0 {
            dmg += hp.0 * if creep.is_boss { 0.02 } else { 0.12 };
            col = Color::srgb(0.85, 0.8, 0.4);
            }
            hp.0 -= dmg;
            add_frame_dmg(&mut frame_dmg, hit_ent, hit_pos, dmg, col);
        }
        // znacky specialnich vezi na zasazeneho creepa
        if let Some(mut ec) = commands.get_entity(hit_ent) {
            match bullet.kind {
                KAKTUS => { ec.try_insert(crate::components::Cursed { mul: 0.10, timer: Timer::from_seconds(2.0, TimerMode::Once) }); }
                DANOVY_URAD => {
                    // kletba +8 % dmg, rusi stit a heal (GDD 7)
                    ec.try_insert(crate::components::Cursed { mul: 0.08, timer: Timer::from_seconds(2.0, TimerMode::Once) });
                    ec.remove::<crate::components::Shield>();
                }
                BABICKA if bullet.shot_count % 6 == 0 => {
                    ec.try_insert(crate::components::StunDebuff { timer: Timer::from_seconds(1.0, TimerMode::Once) });
                }
                ROUTER if bullet.level >= 7 => {
                    ec.try_insert(crate::components::StunDebuff { timer: Timer::from_seconds(1.5, TimerMode::Once) });
                }
                PEPA => {
                    let dirs = [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y];
                    if rng.gen::<f32>() < 0.20 {
                        ec.try_insert(crate::components::Confused { dir: dirs[rng.gen_range(0..4)], remaining: 2.0 * crate::constants::CELL });
                    }
                }
                HAXOR if rng.gen::<f32>() < 0.15 => {
                    if let Some(owner_side) = bullet.owner_side {
                        let destination = owner_side.other();
                        if let Ok(mut target_side) = creep_sides.get_mut(hit_ent) {
                            let previous = *target_side;
                            if previous != destination {
                                if let Ok(mut tf) = creep_positions.get_mut(hit_ent) {
                                    tf.translation.x += destination.lane_x() - previous.lane_x();
                                }
                                *target_side = destination;
                            }
                        }
                    }
                }
                TROLLFACE if rng.gen::<f32>() < 0.25 => {
                    ec.try_insert(crate::components::Knockback { remaining: 2.0 * crate::constants::CELL, speed: 500.0 });
                }
                MRAZAK if bullet.level >= 7 && rng.gen::<f32>() < 0.25 => {
                    ec.try_insert(crate::components::StunDebuff { timer: Timer::from_seconds(1.2, TimerMode::Once) });
                }
                TCHYNE if bullet.level >= 3 && creeps.get(hit_ent).map_or(false, |creep| creep.2.is_boss) => {
                    ec.try_insert(crate::components::StunDebuff { timer: Timer::from_seconds(3.0, TimerMode::Once) });
                }
                KAREN if creeps.get(hit_ent).map_or(false, |creep| creep.2.is_boss) => {
                    ec.try_insert(crate::components::SlowDebuff { timer: Timer::from_seconds(2.0, TimerMode::Once), factor: 0.85 });
                    ec.try_insert(crate::components::Cursed { mul: 0.0, timer: Timer::from_seconds(2.0, TimerMode::Once) });
                }
                BLACK_HOLE => {
                    if let Some(side) = bullet.owner_side {
                        let dx = side.lane_x() - hit_pos.x;
                        if dx.abs() > 1.0 {
                            ec.try_insert(crate::components::Confused { dir: Vec2::new(dx.signum(), 0.0), remaining: 2.0 * crate::constants::CELL });
                        }
                        if bullet.level >= 3 && rng.gen::<f32>() < 0.10 {
                            let destination = side.other();
                            if let Ok(mut target_side) = creep_sides.get_mut(hit_ent) {
                                if let Ok(mut tf) = creep_positions.get_mut(hit_ent) {
                                    tf.translation.x += destination.lane_x() - target_side.lane_x();
                                    tf.translation.y = crate::constants::LANE_Y + crate::constants::LANE_H * 0.5 + crate::constants::SPAWN_ABOVE;
                                }
                                *target_side = destination;
                            }
                            if let Ok((mut hp, mut max_hp, _, _)) = creeps.get_mut(hit_ent) {
                                hp.0 *= 2.0;
                                max_hp.0 *= 2.0;
                            }
                        }
                    }
                }
                _ => {}
            }
            if bullet.kind == TOUSTOVAC && bullet.level >= 7 {
                ec.try_insert(crate::components::DeathExplosion { damage: bullet.damage, radius: 1.5 * crate::constants::CELL });
            }
        }
        if matches!(bullet.kind, SHREK | HAXOR | TROLLFACE) && bullet.level >= 3 {
            let used = bullet.owner_side.map_or(false, |side| {
                tower_abilities.iter_mut().find(|tower| {
                    tower.kind == bullet.kind
                        && tower.side == side
                        && tower.row == bullet.owner_row as usize
                        && tower.col == bullet.owner_col as usize
                }).map_or(false, |mut tower| {
                    if tower.last_ability_wave == bullet.wave { return false; }
                    tower.last_ability_wave = bullet.wave;
                    true
                })
            });
            if used {
                if bullet.kind == SHREK {
                    let start_y = crate::constants::LANE_Y + crate::constants::LANE_H * 0.5 + crate::constants::SPAWN_ABOVE;
                    if let Some(mut ec) = commands.get_entity(hit_ent) {
                        ec.try_insert(crate::components::Knockback { remaining: (start_y - hit_pos.y).max(0.0), speed: 500.0 });
                    }
                } else if let Some(side) = bullet.owner_side {
                    near.clear();
                    grid.query(
                        Vec2::new(side.lane_x(), crate::constants::LANE_Y + crate::constants::LANE_H * 0.5),
                        crate::constants::LANE_H + crate::constants::SPAWN_ABOVE,
                        &mut near,
                    );
                    if bullet.kind == HAXOR {
                        for &(entity, pos) in near.iter() {
                            if (pos.x - side.lane_x()).abs() <= crate::constants::LANE_W * 0.5 {
                                if let Some(mut ec) = commands.get_entity(entity) {
                                    ec.try_insert(crate::components::StunDebuff { timer: Timer::from_seconds(2.0, TimerMode::Once) });
                                }
                            }
                        }
                    } else {
                        let mut strongest: Option<(Entity, Vec2, f32)> = None;
                        let mut weakest: Option<(Entity, Vec2, f32)> = None;
                        for &(entity, pos) in near.iter() {
                            if (pos.x - side.lane_x()).abs() > crate::constants::LANE_W * 0.5 { continue; }
                            if let Ok((hp, _, _, _)) = creeps.get_mut(entity) {
                                if strongest.map_or(true, |(_, _, value)| hp.0 > value) {
                                    strongest = Some((entity, pos, hp.0));
                                }
                                if weakest.map_or(true, |(_, _, value)| hp.0 < value) {
                                    weakest = Some((entity, pos, hp.0));
                                }
                            }
                        }
                        if let (Some((strong_entity, _, _)), Some((weak_entity, _, _))) = (strongest, weakest) {
                            if strong_entity != weak_entity {
                                let strong_pos = creep_positions.get_mut(strong_entity).ok().map(|tf| tf.translation);
                                let weak_pos = creep_positions.get_mut(weak_entity).ok().map(|tf| tf.translation);
                                if let (Some(strong_pos), Some(weak_pos)) = (strong_pos, weak_pos) {
                                    if let Ok(mut tf) = creep_positions.get_mut(strong_entity) { tf.translation = weak_pos; }
                                    if let Ok(mut tf) = creep_positions.get_mut(weak_entity) { tf.translation = strong_pos; }
                                }
                            }
                        }
                    }
                }
            }
        }
        // get_entity: creep mohl byt mezitim despawnuty
        if bullet.slow > 0.0 {
            if let Some(mut ec) = commands.get_entity(hit_ent) { ec.try_insert(crate::components::SlowDebuff { timer: Timer::from_seconds(2.5, TimerMode::Once), factor: 1.0 - bullet.slow.clamp(0.0, bullet.slow_max) * (1.0 - resist_of(&creep_defs, hit_ent).map_or(0.0, |r| r.slow)) }); }
        }
        if bullet.dot_dps > 0.0 {
            if let Some(mut ec) = commands.get_entity(hit_ent) {
                if bullet.kind == TOWER_POISON {
                    if let Ok(mut poison) = poisons.get_mut(hit_ent) {
                        poison.stacks = (poison.stacks + 1).min(5);
                        poison.dps = bullet.dot_dps * poison.stacks as f32;
                        poison.timer.reset();
                        if poison.stacks == 5 {
                            let dirs = [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y];
                            ec.try_insert(crate::components::Confused { dir: dirs[rng.gen_range(0..4)], remaining: 2.0 * crate::constants::CELL });
                        }
                    } else {
                        ec.try_insert(crate::components::PoisonDebuff { timer: Timer::from_seconds(bullet.dot_dur, TimerMode::Once), tick: Timer::from_seconds(0.5, TimerMode::Repeating), dps: bullet.dot_dps, level: bullet.level, stacks: 1 });
                    }
                } else {
                    ec.try_insert(crate::components::BurnDebuff { timer: Timer::from_seconds(bullet.dot_dur, TimerMode::Once), tick: Timer::from_seconds(0.5, TimerMode::Repeating), dps: bullet.dot_dps, level: bullet.level, kind: bullet.kind });
                }
            }
        }
        apply_push_stun(&mut commands, hit_ent, &bullet, &mut rng, resist_of(&creep_defs, hit_ent).as_ref());
        if bullet.chain_left > 0 {
            // chain: blesk skace vzdy na nejblizsiho jeste nezasazeneho creepa, kazdy skok o chain_falloff mene damage
            chained.clear();
            chained.push(hit_ent);
            let mut cur_pos = hit_pos;
            let mut dmg = bullet.damage;
            for _ in 0..bullet.chain_left {
                near.clear();
                grid.query(cur_pos, bullet.chain_radius, &mut near);
                let next = near.iter().filter(|(e, _)| !chained.contains(e) && (bullet.hits_air || !air.contains(*e))).min_by(|a, b| a.1.distance_squared(cur_pos).total_cmp(&b.1.distance_squared(cur_pos)));
                let Some(&(c_ent, c_pos)) = next else { break; };
                dmg *= bullet.chain_falloff;
                if let Ok((mut hp, _, _, _)) = creeps.get_mut(c_ent) {
                    let shown = dmg * resist_mul(target_of(&creep_defs, c_ent), bullet.kind); // odolnost podle druhu veze (dmg se dal snizuje bez ni)
                    hp.0 -= shown;
                    add_frame_dmg(&mut frame_dmg, c_ent, c_pos, shown, Color::srgb(0.6, 0.9, 1.0));
                    apply_push_stun(&mut commands, c_ent, &bullet, &mut rng, resist_of(&creep_defs, c_ent).as_ref());
                }
                crate::systems::effects::spawn_bolt(&mut commands, &mut bolt_pool, cur_pos, c_pos); // svetla car mezi creepy
                chained.push(c_ent);
                cur_pos = c_pos;
            }
        }
        bullet.active = false; *b_vis = Visibility::Hidden; bullet_pool.free.push(b_ent); // zpet do poolu
    }
    // 1 cislo na creepa za snimek (soucet vsech zasahu)
    for (_, (pos, dmg, col)) in frame_dmg {
        crate::systems::debuff::spawn_damage_number(&mut commands, &mut pool, dmg, col, Vec2::new(pos.x, pos.y + 18.0), 25.0, 0.7);
    }
}

// Pred bullet_hit: prebuduje prostorovou mrizku creepu
pub fn build_creep_grid(mut grid: ResMut<crate::resources::CreepGrid>, creeps: Query<(Entity, &Transform), With<Creep>>) {
    grid.clear();
    for (e, tf) in creeps.iter() { grid.insert(e, tf.translation.truncate()); }
}

const BULLET_POOL_SIZE: usize = 300; // max strel naraz; kdyz dojdou, veze chvili nestrili

// Startup: predvytvori skryte strely
pub fn setup_bullet_pool(mut commands: Commands, mut pool: ResMut<crate::resources::BulletPool>) {
    for _ in 0..BULLET_POOL_SIZE {
        let e = commands.spawn((
            Sprite { color: Color::srgb(1.0, 1.0, 0.8), custom_size: Some(Vec2::new(8.0, 8.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 6.0),
            Visibility::Hidden,
            Bullet::default(),
        )).id();
        pool.free.push(e);
    }
}

#[cfg(test)]
mod tests {
    use super::dj_drop_ready;

    #[test]
    fn dj_drop_triggers_on_elapsed_ten_second_intervals() {
        let mut timer = 10.0;
        assert!(!dj_drop_ready(&mut timer, 9.0));
        assert!(dj_drop_ready(&mut timer, 1.0));
        assert!(!dj_drop_ready(&mut timer, 1.0));
    }
}

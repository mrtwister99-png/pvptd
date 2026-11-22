
use bevy::prelude::*;
use std::collections::HashMap;
use rand::Rng;
use crate::components::{Tower, Creep, Bullet, Air};
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

// Air / Ground: ktere veze umi strilet do vzduchu (podle MD: Cannon jen zem, Fire L3 Meteor Tower jen zem)
fn hits_air(kind: u8, level: u32) -> bool {
    use crate::special::*;
    match kind { 2 => false, 4 => level != 3, ILLUSION => false, _ => true }
}

// odolnost creepa vuci druhu veze (tabulka Resist ma jen puvodnich 8 druhu; nove veze = bez odolnosti)
fn resist_mul(r: Option<&crate::components::Resist>, kind: u8) -> f32 {
    let k = kind as usize;
    if k < 8 { r.map_or(1.0, |r| r.dmg_mul[k]) } else { 1.0 }
}

fn attack_def(kind: u8) -> AttackDef {
    use std::f32::consts::FRAC_PI_2;
    match kind {
        // Arrow: sip.png (miri nahoru -> otocit o -90 stupnu)
        1 => AttackDef { speed: 600.0, size: Vec2::new(14.0, 28.0), size_per_level: 0.10, color: Color::srgb(1.0, 1.0, 0.8), image: Some("bullets/sip.png"), rot_offset: -FRAC_PI_2, hit_radius: 24.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Cannon: cannon.png (kulicka), pomala
        2 => AttackDef { speed: 380.0, size: Vec2::new(18.0, 18.0), size_per_level: 0.15, color: Color::srgb(1.0, 0.6, 0.1), image: Some("bullets/cannon.png"), rot_offset: 0.0, hit_radius: 30.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Frost
        3 => AttackDef { speed: 450.0, size: Vec2::new(9.0, 9.0), size_per_level: 0.10, color: Color::srgb(0.2, 0.6, 1.0), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Fire
        4 => AttackDef { speed: 420.0, size: Vec2::new(9.0, 9.0), size_per_level: 0.12, color: Color::srgb(1.0, 0.3, 0.1), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Poison: zpomaluje strelou
        5 => AttackDef { speed: 400.0, size: Vec2::new(8.0, 8.0), size_per_level: 0.10, color: Color::srgb(0.3, 0.9, 0.3), image: None, rot_offset: 0.0, hit_radius: 24.0, slow: true, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Sniper: nejrychlejsi
        6 => AttackDef { speed: 900.0, size: Vec2::new(8.0, 8.0), size_per_level: 0.10, color: Color::srgb(0.3, 0.4, 1.0), image: None, rot_offset: 0.0, hit_radius: 26.0, slow: false, chain_radius: 150.0, chain_falloff: 0.75, push_speed: 500.0 },
        // Shock Turret: velmi rychla strela, chain skace az 150 u, kazdy skok o 25 % mene, odhozeni 500 u/s
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
        let only_air = tower.kind == crate::special::HARPOON; // Harpoon strili jen na letajici
        let t_pos = t_tf.translation.truncate();
        let mut target: Option<Vec2> = None;
        let mut best_dist = f32::MAX;
        let can_air = hits_air(tower.kind, tower.level);
        let range = tower.range * mods.map_or(1.0, |m| m.range_mul); // aura creepu muze zkratit dosah
        let mut random_pick: Option<Vec2> = None; // nahodny cil v dosahu (aura Infernal)
        let mut in_range = 0u32;
        for (c_tf, is_air) in creeps.iter() {
            if is_air && !can_air { continue; } // zemni vez nema na letajici cil
            if only_air && !is_air { continue; }
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
                damage: (tower.damage_min + tower.damage_max) * 0.5 * mods.map_or(1.0, |m| m.dmg_mul),
                vel: dir * atk.speed,
                life: Timer::from_seconds(range / atk.speed + 0.2, TimerMode::Once),
                aoe: tower.splash,
                kind: tower.kind,
                level: tower.level,
                slow: if atk.slow { tower.slow_per_hit } else { 0.0 },
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

pub fn bullet_hit(
    mut commands: Commands,
    mut pool: ResMut<crate::resources::DamageNumberPool>,
    mut bullet_pool: ResMut<crate::resources::BulletPool>,
    mut bolt_pool: ResMut<crate::systems::effects::BoltPool>,
    grid: Res<crate::resources::CreepGrid>,
    mut bullets: Query<(Entity, &Transform, &mut Bullet, &mut Visibility), Without<Creep>>,
    mut creeps: Query<(&mut crate::components::Health, &crate::components::MaxHealth, &Creep, Option<&crate::components::Cursed>, Option<&crate::components::VoodooMark>), With<Creep>>,
    air: Query<(), With<Air>>,
    resists: Query<&crate::components::Resist>,
    time: Res<Time>,
    mut waves: ResMut<crate::fx::ShockwavePool>,
    mut wave_mats: ResMut<Assets<crate::fx::ShockwaveMaterial>>,
) {
    use crate::special::*;
    let now = time.elapsed_secs_wrapped();
    let mut shares: Vec<(Entity, Vec2, f32)> = Vec::new(); // Voodoo: (oznaceny creep, pozice, poskozeni k rozdeleni okoli)
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
                if let Ok((mut hp, _, _, cursed, voodoo)) = creeps.get_mut(c_ent) {
                    let dmg = bullet.damage * resist_mul(resists.get(c_ent).ok(), bullet.kind) * (1.0 + cursed.map_or(0.0, |c| c.mul)); // odolnost + kletba
                    hp.0 -= dmg;
                    add_frame_dmg(&mut frame_dmg, c_ent, c_pos, dmg, Color::srgb(1.0, 0.4, 0.1));
                    if let Some(v) = voodoo { shares.push((c_ent, c_pos, dmg * v.share)); }
                }
            }
        } else if let Ok((mut hp, max_hp, creep, cursed, voodoo)) = creeps.get_mut(hit_ent) {
            let mut dmg = bullet.damage * resist_mul(resists.get(hit_ent).ok(), bullet.kind) * (1.0 + cursed.map_or(0.0, |c| c.mul)); // odolnost + kletba
            let mut col = Color::srgb(1.0, 0.4, 0.1);
            if bullet.kind == EXECUTIONER && hp.0 > 0.0 {
                // % chybejiciho HP navic; pod hranici poprava (bossove nikdy)
                dmg += (max_hp.0 - hp.0).max(0.0) * executioner_missing_pct(bullet.level);
                if !creep.is_boss && hp.0 / max_hp.0.max(1.0) < executioner_threshold(bullet.level) { dmg = dmg.max(hp.0); col = Color::srgb(0.9, 0.1, 0.1); }
            }
            hp.0 -= dmg;
            add_frame_dmg(&mut frame_dmg, hit_ent, hit_pos, dmg, col);
            if let Some(v) = voodoo { shares.push((hit_ent, hit_pos, dmg * v.share)); }
        }
        // znacky specialnich vezi na zasazeneho creepa
        if let Some(mut ec) = commands.get_entity(hit_ent) {
            match bullet.kind {
                HARPOON => { ec.remove::<Air>(); } // srazen na zem: dal musi bludistem
                CURSE => { ec.try_insert(crate::components::Cursed { mul: curse_mul(bullet.level), timer: Timer::from_seconds(CURSE_SECS, TimerMode::Once) }); }
                VOODOO => { ec.try_insert(crate::components::VoodooMark { share: voodoo_share(bullet.level), timer: Timer::from_seconds(VOODOO_SECS, TimerMode::Once) }); }
                ILLUSION => {
                    // nahodny smer (vlevo / vpravo / nahoru / dolu), nahodna vzdalenost podle ranku
                    let dirs = [Vec2::X, -Vec2::X, Vec2::Y, -Vec2::Y];
                    let dir = dirs[rng.gen_range(0..4)];
                    let boss = creeps.get(hit_ent).map_or(false, |c| c.2.is_boss);
                    let dist = illusion_dist(bullet.level) * rng.gen_range(0.5..1.0) * if boss { 0.5 } else { 1.0 };
                    ec.try_insert(crate::components::Confused { dir, remaining: dist });
                }
                _ => {}
            }
        }
        // get_entity: creep mohl byt mezitim despawnuty
        if bullet.slow > 0.0 {
            if let Some(mut ec) = commands.get_entity(hit_ent) { ec.try_insert(crate::components::SlowDebuff { timer: Timer::from_seconds(2.5, TimerMode::Once), factor: 1.0 - bullet.slow.clamp(0.0, bullet.slow_max) * (1.0 - resists.get(hit_ent).map_or(0.0, |r| r.slow)) }); }
        }
        if bullet.dot_dps > 0.0 {
            if let Some(mut ec) = commands.get_entity(hit_ent) {
                // poison veze davaji jed (zelena cisla), ostatni burn
                if bullet.kind == TOWER_POISON {
                    ec.try_insert(crate::components::PoisonDebuff { timer: Timer::from_seconds(bullet.dot_dur, TimerMode::Once), tick: Timer::from_seconds(0.5, TimerMode::Repeating), dps: bullet.dot_dps, level: bullet.level });
                } else {
                    ec.try_insert(crate::components::BurnDebuff { timer: Timer::from_seconds(bullet.dot_dur, TimerMode::Once), tick: Timer::from_seconds(0.5, TimerMode::Repeating), dps: bullet.dot_dps, level: bullet.level });
                }
            }
        }
        apply_push_stun(&mut commands, hit_ent, &bullet, &mut rng, resists.get(hit_ent).ok());
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
                if let Ok((mut hp, _, _, _, _)) = creeps.get_mut(c_ent) {
                    let shown = dmg * resist_mul(resists.get(c_ent).ok(), bullet.kind); // odolnost podle druhu veze (dmg se dal snizuje bez ni)
                    hp.0 -= shown;
                    add_frame_dmg(&mut frame_dmg, c_ent, c_pos, shown, Color::srgb(0.6, 0.9, 1.0));
                    apply_push_stun(&mut commands, c_ent, &bullet, &mut rng, resists.get(c_ent).ok());
                }
                crate::systems::effects::spawn_bolt(&mut commands, &mut bolt_pool, cur_pos, c_pos); // svetla car mezi creepy
                chained.push(c_ent);
                cur_pos = c_pos;
            }
        }
        bullet.active = false; *b_vis = Visibility::Hidden; bullet_pool.free.push(b_ent); // zpet do poolu
    }
    // Voodoo: poskozeni oznacenych creepu se prenese na creepy kolem (dal uz se neprenasi)
    for (src, pos, dmg) in shares {
        near.clear();
        grid.query(pos, VOODOO_RADIUS, &mut near);
        for &(c_ent, c_pos) in near.iter() {
            if c_ent == src { continue; }
            if let Ok((mut hp, _, _, _, _)) = creeps.get_mut(c_ent) {
                hp.0 -= dmg;
                add_frame_dmg(&mut frame_dmg, c_ent, c_pos, dmg, Color::srgb(0.8, 0.4, 0.8));
            }
        }
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

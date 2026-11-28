
use bevy::prelude::*;
use rand::Rng;
use crate::components::{Creep, CreepMainCircle, Health, MaxHealth, CreepHpText, CreepStatusText, SlowOutline, SlowDebuff, BurnDebuff, HpBarBg, HpBarFill, HpBarText, SentUnit, SendPanelItem, IncomeText, Knockback, Enrage, EnrageFlash, Resist, Heal, Tower, Aura, AuraBuff, ShootBack, TowerSlow, FrostOverlay, Shimmer, GuardianMsg, GuardianMsgText, CheatButton, TowerAura, TowerMods, DmgTarget, Shield, Regen, CopyOnDeath, SpeedAura, AuraSpeed, RandomSpeed, Boss, Selling};
use crate::resources::{CreepAssets, RoundState, RoundPhase, MatchOver, WavePaused, WaveCount, Sides, SelectedCell, GameMode, MatchStats, TEST_GOLD, TEST_DIAMONDS};
use crate::components::Side;
use crate::constants::{LANE_Y, LANE_W, LANE_H, CREEP_BASE_SPEED, FROST_OUTLINE_MAX_EXTRA, COLS, ROWS, CELL, SPAWN_ABOVE, TOTAL_LIVES, ROUND_PAUSE_SECS, ROUND_MAX_SECS};
use crate::wave_def::{self as gdd, CreepType, BossKind, wave_def, creep_def, boss_def, send_def, send_hp, Target};
use crate::utils::{world_to_grid, grid_to_world, compute_path};

// === KOLA ===
// Pauza (prvni je delsi = stavba) -> kolo: na obe lajny prijde stejna vlna z GDD (mali + velci creepove, kazda 5. vlna boss).
// Kolo skonci, az z obou lajn zmizi vsichni creepi automaticke vlny (zabiti nebo prosli), nejpozdeji po ROUND_MAX_SECS
// (vlna 30 a dal ceka, az dojdou vsichni). Pak gold za prezitou vlnu a pauza ROUND_PAUSE_SECS. Posilat jde jen behem kola.
pub fn round_system(
    mut commands: Commands,
    time: Res<Time>,
    mut round: ResMut<RoundState>,
    mut wave: ResMut<WaveCount>,
    paused: Res<WavePaused>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<CreepAssets>,
    asset_server: Res<AssetServer>,
    wave_creeps: Query<(), (With<Creep>, Without<SentUnit>)>,
    towers: Query<&Tower>,
    mut sides: ResMut<Sides>,
    mut stats: ResMut<MatchStats>,
    mut over: ResMut<MatchOver>,
    mut msg: Query<(&mut GuardianMsg, &mut Visibility)>,
    mut msg_text: Query<&mut Text, With<GuardianMsgText>>,
) {
    if paused.0 || over.0.is_some() { return; }
    match round.phase {
        RoundPhase::Pause => {
            if !round.timer.tick(time.delta()).just_finished() { return; }
            wave.0 += 1;
            round.phase = RoundPhase::Running;
            round.elapsed = 0.0;
            stats.sends = [0; 2];
            stats.mini_bosses = [0; 2];
            spawn_round_wave(&mut commands, &mut meshes, &mut materials, &assets, &asset_server, wave.0, &Side::ALL);
            if let Some((b, _)) = wave_def(wave.0).boss {
                let d = boss_def(b);
                show_msg(&mut msg, &mut msg_text, &format!("BOSS: {} - \"{}\"", d.name, d.quote));
            }
            // log ekonomiky na zacatku kola (ladeni): gold, income a hodnota vezi kazde strany
            for side in Side::ALL {
                let st = sides.get(side);
                let invested: i32 = towers.iter().filter(|t| t.side == side).map(|t| t.invested).sum();
                println!("EKONOMIKA kolo {} {:?}: gold {} income {:.0} veze {}G zivoty {} diamanty {}", wave.0, side, st.gold, st.income.amount, invested, st.lives, st.diamonds);
            }
        }
        RoundPhase::Running => {
            round.elapsed += time.delta_secs();
            let cleared = round.elapsed > 1.0 && wave_creeps.is_empty(); // 1 s rezerva, nez se spawnuti projevi
            let timeout = wave.0 < gdd::LAST_WAVE && round.elapsed >= ROUND_MAX_SECS;
            if !(cleared || timeout) { return; }
            round.phase = RoundPhase::Pause;
            round.timer = Timer::from_seconds(ROUND_PAUSE_SECS, TimerMode::Once);
            // gold za prezitou vlnu (GDD 10) + ekonomicke veze
            let reward = wave_def(wave.0).reward;
            for side in Side::ALL { sides.get_mut(side).gold += reward; }
            for t in towers.iter() {
                if t.kind == crate::special::KASICKA {
                    let bonus = if wave.0 % 5 == 0 { 104 } else { 4 };
                    sides.get_mut(t.side).gold += bonus;
                } else if t.kind == crate::special::DOGE_MINER {
                    let st = sides.get_mut(t.side);
                    st.gold += (8.0 * crate::special::special_ability_multiplier(t.level)).round() as i32;
                    if t.level >= 3 && !st.doge_moon_used {
                        st.doge_moon_used = true;
                        st.gold_boost_timer = 10.0;
                    }
                }
            }
            println!("Kolo {} skonceno, +{}G za prezitou vlnu, pauza {} s", wave.0, reward, ROUND_PAUSE_SECS);
            // konec 30. vlny: vic zivotu vyhrava, pri shode vic zpusobenych leaku, pak rychlejsi zabiti posledniho bosse,
            // jinak sudden death (dalsi leak = konec)
            if wave.0 == gdd::LAST_WAVE {
                match decide_after_last_wave(&sides, &stats) {
                    Some(winner) => { over.0 = Some(winner); println!("KONEC PO 30. VLNE - vitez {:?}", winner); }
                    None => { stats.sudden_death = true; println!("SUDDEN DEATH: dalsi leak = konec"); }
                }
            }
        }
    }
}

// rozhodnuti po vlne 30 (GDD 2): None = uplna shoda -> sudden death
pub fn decide_after_last_wave(sides: &Sides, stats: &MatchStats) -> Option<Side> {
    let (p, b) = (Side::Player, Side::Bot);
    let lives = (sides.get(p).lives, sides.get(b).lives);
    if lives.0 != lives.1 { return Some(if lives.0 > lives.1 { p } else { b }); }
    // zpusobene leaky = kolik creepu proslo souperi
    let caused = (stats.leaks[b.idx()], stats.leaks[p.idx()]);
    if caused.0 != caused.1 { return Some(if caused.0 > caused.1 { p } else { b }); }
    match (stats.last_boss_kill[p.idx()], stats.last_boss_kill[b.idx()]) {
        (Some(x), Some(y)) if (x - y).abs() > 0.01 => Some(if x < y { p } else { b }),
        (Some(_), None) => Some(p),
        (None, Some(_)) => Some(b),
        _ => None,
    }
}

fn show_msg(msg: &mut Query<(&mut GuardianMsg, &mut Visibility)>, msg_text: &mut Query<&mut Text, With<GuardianMsgText>>, text: &str) {
    for mut t in msg_text.iter_mut() { t.0 = text.to_string(); }
    for (mut m, mut vis) in msg.iter_mut() {
        m.timer = Timer::from_seconds(4.0, TimerMode::Once);
        *vis = Visibility::Visible;
    }
}

// jak velky je creep podle role (polomer)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CreepRole { Small, Big, MiniBoss, Boss }

fn role_radius(role: CreepRole) -> f32 {
    match role { CreepRole::Small => 13.0, CreepRole::Big => 19.0, CreepRole::MiniBoss => 24.0, CreepRole::Boss => 30.0 }
}

// vse, co je potreba vedet pro vytvoreni creepa (automaticka vlna, poslana jednotka i kopie)
#[derive(Clone, Copy)]
pub struct CreepSpec {
    pub ctype: CreepType,
    pub role: CreepRole,
    pub boss: Option<BossKind>,
    pub hp: f32,
    pub leak: u32,
    pub kill_gold: i32,
    pub sent: bool,
    pub copy: bool,      // kopie Kopirovaciho Bota uz se dal nekopiruje
    pub pos: Vec2,
}

// vytvori creepa podle GDD: telo (obrazek), HP bar, schopnosti podle typu / bosse
pub fn spawn_creep(commands: &mut Commands, assets: &CreepAssets, meshes: &mut Assets<Mesh>, materials: &mut Assets<ColorMaterial>, asset_server: &AssetServer, side: Side, s: CreepSpec) {
    let def = creep_def(s.ctype);
    let radius = role_radius(s.role);
    let mut target = Target::of(s.ctype);
    let (speed, image, name) = match s.boss {
        Some(b) => { let d = boss_def(b); target.armored = d.armored; target.boss = true; (d.speed, d.image, Some(d.name)) }
        None => (def.speed, def.image, None),
    };
    let mut ec = commands.spawn((
        Transform::from_xyz(s.pos.x, s.pos.y, 4.0),
        Creep { is_boss: s.boss.is_some(), is_mega: false, max_hp_reductions: 0, base_radius: radius, speed_mul: speed, leak_cost: s.leak, kill_gold: s.kill_gold },
        CreepMainCircle,
        Health(s.hp),
        MaxHealth(s.hp),
        DmgTarget(target),
        Sprite { image: asset_server.load(image), custom_size: Some(Vec2::splat(radius * 2.0)), ..default() },
        side,
    ));
    if s.sent { ec.insert(SentUnit); }
    // schopnosti typu (GDD 9)
    match s.ctype {
        _ if s.boss.is_some() => {}
        CreepType::AdBlock => { ec.insert(Shield { hp: s.hp * gdd::ADBLOCK_SHIELD, prev: s.hp }); }
        CreepType::WifiUpir => { ec.insert(Regen { idle: 0.0, prev: s.hp }); }
        CreepType::KopirBot if !s.copy => { ec.insert(CopyOnDeath); }
        CreepType::Influencer => { ec.insert(SpeedAura { bonus: gdd::INFLUENCER_AURA, radius: gdd::INFLUENCER_RADIUS * CELL }); }
        CreepType::Brambora => { ec.insert(RandomSpeed { timer: Timer::from_seconds(gdd::BRAMBORA_EVERY, TimerMode::Repeating), mul: 1.0 }); }
        CreepType::Chad => { ec.insert(Enrage { hp_frac: gdd::CHAD_ENRAGE.0, speed_bonus: gdd::CHAD_ENRAGE.1, size_bonus: 0.0, flash: Color::srgba(1.0, 0.1, 0.1, 0.65) }); }
        _ => {}
    }
    // bossove (GDD 11): odolnost na push / stun, aury a debuffy na veze
    if let Some(b) = s.boss {
        let r = gdd::BOSS_AURA_RADIUS * CELL;
        let aura = |rate: f32, dmg: f32, every: f32, stun: f32, single: bool| TowerAura { radius: r, random_target: 0.0, dmg_mul: dmg, range_mul: 1.0, rate_mul: rate, stomp_every: every, stomp_stun: stun, stomp_single: single, timer: Timer::from_seconds(every.max(0.1), TimerMode::Repeating) };
        ec.insert((Resist { stun_immune: false, push: 0.5, slow: 0.0 }, Boss { kind: b, a: Timer::from_seconds(gdd::UCITEL_GUARD.0, TimerMode::Repeating), b: Timer::from_seconds(gdd::MAMA_SUMMON_EVERY, TimerMode::Repeating), phase: 1, guard_left: 0.0 }));
        match b {
            BossKind::Ucitel => { ec.insert(aura(gdd::UCITEL_TOWER_SLOW, 1.0, 0.0, 0.0, false)); }
            BossKind::Sef => { ec.insert((Enrage { hp_frac: gdd::SEF_ENRAGE.0, speed_bonus: gdd::SEF_ENRAGE.1, size_bonus: 0.0, flash: Color::srgba(1.0, 0.1, 0.1, 0.65) }, SpeedAura { bonus: gdd::SEF_SPEED_AURA, radius: r })); }
            BossKind::Tchyne => { ec.insert(aura(1.0, gdd::TCHYNE_TOWER_DMG, 0.0, 0.0, false)); }
            BossKind::DanovyUrad => { ec.insert(aura(1.0, 1.0, gdd::URAD_DISABLE.0, gdd::URAD_DISABLE.1, true)); }
            BossKind::Babicka => { ec.insert(aura(1.0, 1.0, gdd::BABICKA_STUN.0, gdd::BABICKA_STUN.1, false)); }
            BossKind::Mama => { ec.insert(aura(1.0, 1.0, gdd::MAMA_CALL.0, gdd::MAMA_CALL.1, true)); }
        }
    }
    let unit_mesh = assets.unit_circle.clone();
    let slow_mat = assets.slow_mat.clone();
    let label = format!("{:.0}", s.hp);
    let aura_ring = if s.ctype == CreepType::Influencer && s.boss.is_none() {
        Some((meshes.add(Annulus::new(0.99, 1.0)), materials.add(ColorMaterial::from_color(Color::srgba(1.0, 0.5, 0.8, 0.35)))))
    } else { None };
    let shield_ring = if s.ctype == CreepType::AdBlock && s.boss.is_none() { Some(materials.add(ColorMaterial::from_color(Color::srgba(0.7, 0.85, 1.0, 0.9)))) } else { None };
    ec.with_children(|parent| {
        if let Some((mesh, mat)) = aura_ring {
            let rr = gdd::INFLUENCER_RADIUS * CELL;
            parent.spawn((Mesh2d(mesh), MeshMaterial2d(mat), Transform::from_xyz(0.0, 0.0, -0.3).with_scale(Vec3::new(rr, rr, 1.0))));
        }
        if let Some(mat) = shield_ring {
            parent.spawn((Mesh2d(assets.unit_circle.clone()), MeshMaterial2d(mat), Transform::from_xyz(0.0, 0.0, 0.05).with_scale(Vec3::new(radius + 3.0, radius + 3.0, 1.0)), ShieldRing));
        }
        if let Some(name) = name {
            parent.spawn((Text2d::new(name), TextFont { font_size: 11.0, ..default() }, TextColor(Color::srgb(1.0, 0.85, 0.2)), Transform::from_xyz(0.0, radius + 48.0, 0.4)));
        }
        parent.spawn((Sprite { color: Color::srgb(0.1, 0.1, 0.1), custom_size: Some(Vec2::new(34.0, 6.0)), ..default() }, Transform::from_xyz(0.0, radius + 14.0, 0.2), HpBarBg));
        parent.spawn((Sprite { color: Color::srgb(0.2, 0.9, 0.2), custom_size: Some(Vec2::new(32.0, 4.0)), ..default() }, Transform::from_xyz(0.0, radius + 14.0, 0.3), HpBarFill));
        parent.spawn((Text2d::new(label), TextFont { font_size: if s.role == CreepRole::Small { 10.0 } else { 12.0 }, ..default() }, TextColor(Color::WHITE), Transform::from_xyz(0.0, radius + 26.0, 0.4), HpBarText, CreepHpText));
        parent.spawn((Text2d::new(""), TextFont { font_size: 9.0, ..default() }, TextColor(Color::srgb(0.4, 0.8, 1.0)), Transform::from_xyz(0.0, radius + 36.0, 0.4), CreepStatusText));
        let outline_radius = radius + FROST_OUTLINE_MAX_EXTRA;
        parent.spawn((Mesh2d(unit_mesh), MeshMaterial2d(slow_mat), Transform::from_xyz(0.0, 0.0, -0.1).with_scale(Vec3::new(outline_radius, outline_radius, 1.0)), SlowOutline, Visibility::Hidden));
    });
}

// modry krouzek stitu (AdBlock); zmizi, kdyz stit dojde
#[derive(Component)] pub struct ShieldRing;

// automaticka vlna z GDD 10 na zadane lajny: mali creepove, za nimi velci, uplne vzadu boss
pub fn spawn_round_wave(commands: &mut Commands, meshes: &mut Assets<Mesh>, materials: &mut Assets<ColorMaterial>, assets: &CreepAssets, asset_server: &AssetServer, wave: u32, lanes: &[Side]) {
    let w = wave_def(wave);
    let mut lineup: Vec<(CreepType, CreepRole, Option<BossKind>, f32, u32)> = Vec::new();
    let small_leak = if wave >= gdd::LAST_WAVE { gdd::LEAK_BIG } else { gdd::LEAK_SMALL }; // vlna 30+: "smisena elite"
    for i in 0..w.small.1 {
        let t = if w.mixed { gdd::mixed_type(i) } else { w.small.0 };
        lineup.push((t, CreepRole::Small, None, w.small.2, small_leak));
    }
    if let Some((t, n, hp)) = w.big { for _ in 0..n { lineup.push((t, CreepRole::Big, None, hp, gdd::LEAK_BIG)); } }
    if let Some((b, hp)) = w.boss { lineup.push((CreepType::Wojak, CreepRole::Boss, Some(b), hp, gdd::LEAK_BOSS)); }
    println!("Kolo {}: {}x {} {} HP{}{}", wave, w.small.1, creep_def(w.small.0).name, w.small.2,
        w.big.map_or(String::new(), |(t, n, hp)| format!(" + {}x {} {} HP", n, creep_def(t).name, hp)),
        w.boss.map_or(String::new(), |(b, hp)| format!(" + BOSS {} {} HP", boss_def(b).name, hp)));
    for &side in lanes {
        let mut rng = rand::thread_rng();
        for (i, &(t, role, boss, hp, leak)) in lineup.iter().enumerate() {
            let x = side.lane_x() + rng.gen_range(-LANE_W * 0.35..LANE_W * 0.35);
            let y = LANE_Y + LANE_H / 2.0 + SPAWN_ABOVE + i as f32 * 20.0 + rng.gen_range(-6.0..6.0);
            let kill_gold = if boss.is_some() { gdd::BOSS_KILL_GOLD } else { creep_def(t).kill_gold };
            spawn_creep(commands, assets, meshes, materials, asset_server, side, CreepSpec { ctype: t, role, boss, hp, leak, kill_gold, sent: false, copy: false, pos: Vec2::new(x, y) });
        }
    }
}

pub fn move_creeps_maze(
    mut commands: Commands,
    mut creeps: Query<(Entity, &mut Transform, &Creep, &Side, Option<&SlowDebuff>, Option<&crate::components::StunDebuff>, Option<&mut Knockback>, Option<&mut crate::components::Confused>, Option<&AuraSpeed>, Option<&RandomSpeed>), (With<Creep>, Without<crate::components::Tower>)>,
    sides: Res<Sides>,
    time: Res<Time>,
    paused: Res<WavePaused>,
) {
    if paused.0 { return; }
    for (e, mut tf, creep, &side, slow_opt, stun_opt, knock_opt, confused, aura_speed, random_speed) in creeps.iter_mut() {
        let grid = &sides.get(side).grid; // kazdy creep chodi bludistem sve lajny
        let path = &sides.get(side).path;
        // odhozeni dozadu (smerem na start): leti nahoru, dokud neurazi zadanou vzdalenost
        if let Some(mut k) = knock_opt {
            let step = (k.speed * time.delta_secs()).min(k.remaining);
            tf.translation.y += step;
            k.remaining -= step;
            if k.remaining <= 0.0 { commands.entity(e).remove::<Knockback>(); }
            continue;
        }
        if let Some(stun) = stun_opt { if !stun.timer.finished() { continue; } }
        // rychlost: typ creepa x zpomaleni (spolecny limit 50 %) x aura rychlosti x nahodna rychlost Brambory
        let slow = slow_opt.map_or(1.0, |s| s.factor.max(1.0 - gdd::SLOW_CAP));
        let speed_factor = creep.speed_mul * slow * (1.0 + aura_speed.map_or(0.0, |a| a.0)) * random_speed.map_or(1.0, |r| r.mul);
        // ILLUSION: zmateny creep jde svym nahodnym smerem, dokud neujde vzdalenost nebo nenarazi na vez / okraj lajny
        if let Some(mut c) = confused {
            let step = (CREEP_BASE_SPEED * creep.speed_mul * time.delta_secs()).min(c.remaining);
            let next = tf.translation.truncate() + c.dir * step;
            let free = world_to_grid(side, next.x, next.y).map_or(false, |(r, cc)| !grid.blocked[r][cc]);
            if free && step > 0.0 {
                tf.translation.x = next.x;
                tf.translation.y = next.y;
                c.remaining -= step;
                continue;
            }
            commands.entity(e).remove::<crate::components::Confused>();
        }
        let pos = tf.translation.truncate();
        let Some((row, col)) = world_to_grid(side, pos.x, pos.y) else {
            // nad lajnou: dojit vodorovne nad nejblizsi otevreny vstup (horni bunka s cestou), pak dolu; zadny vstup = cekat (Pepa to resi)
            let step = CREEP_BASE_SPEED * speed_factor * time.delta_secs();
            let mut target_x: Option<f32> = None;
            if pos.y > LANE_Y + LANE_H / 2.0 {
                for c in 0..COLS {
                    if grid.blocked[ROWS - 1][c] || path.dist[ROWS - 1][c].is_none() { continue; }
                    let cx = grid_to_world(side, ROWS - 1, c).x;
                    if target_x.map_or(true, |t| (cx - pos.x).abs() < (t - pos.x).abs()) { target_x = Some(cx); }
                }
            }
            match target_x {
                Some(tx) if (tx - pos.x).abs() > 3.0 => { tf.translation.x += (tx - pos.x).signum() * step.min((tx - pos.x).abs()); }
                None if pos.y > LANE_Y + LANE_H / 2.0 => { tf.translation.y = (pos.y - step).max(LANE_Y + LANE_H / 2.0 + creep.base_radius); }
                _ => { tf.translation.y -= step; }
            }
            continue;
        };
        let mut best: Option<(usize, usize, i32)> = None;
        if row == 0 { tf.translation.y -= CREEP_BASE_SPEED * speed_factor * time.delta_secs(); continue; } // posledni rada: rovne ven pod lajnu
        for (dr, dc) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
            let nr = row as isize + dr;
            let nc = col as isize + dc;
            if nr < 0 || nr >= ROWS as isize || nc < 0 || nc >= COLS as isize { continue; }
            let (nr, nc) = (nr as usize, nc as usize);
            if grid.blocked[nr][nc] { continue; }
            if let Some(d) = path.dist[nr][nc] {
                if best.map_or(true, |b| d < b.2) { best = Some((nr, nc, d)); }
            }
        }
        if let Some((nr, nc, _)) = best {
            let target = grid_to_world(side, nr, nc);
            let dir = (target - pos).normalize_or_zero();
            tf.translation.x += dir.x * CREEP_BASE_SPEED * speed_factor * time.delta_secs();
            tf.translation.y += dir.y * CREEP_BASE_SPEED * speed_factor * time.delta_secs();
        } else if row > 0 && !grid.blocked[row - 1][col] {
            tf.translation.y -= CREEP_BASE_SPEED * speed_factor * time.delta_secs();
        }
    }
}

// === SCHOPNOSTI CREEPU (GDD 9) ===

// stit pohlti damage (AdBlock, Mama): kolik HP od minula ubylo, tolik se vrati ze stitu, dokud nedojde.
// Bez stitu (dosel / Danovy Urad ho zrusil) uz pruzarne zbrane nejsou oslabene.
pub fn shield_system(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Health, Option<&mut Shield>, &mut DmgTarget, Option<&Children>)>,
    rings: Query<(), With<ShieldRing>>,
) {
    for (e, mut hp, shield, mut target, children) in q.iter_mut() {
        let Some(mut sh) = shield else { if target.0.shielded { target.0.shielded = false; } continue; };
        let lost = sh.prev - hp.0;
        if lost > 0.0 {
            let absorbed = lost.min(sh.hp);
            hp.0 += absorbed;
            sh.hp -= absorbed;
        }
        sh.prev = hp.0;
        target.0.shielded = sh.hp > 0.0;
        if sh.hp <= 0.0 {
            commands.entity(e).remove::<Shield>();
            if let Some(children) = children { for &c in children.iter() { if rings.contains(c) { commands.entity(c).despawn(); } } }
        }
    }
}

// regenerace Wi-Fi Upira: kdyz 2 s nedostal zasah, leci 1,5 % max HP za sekundu
pub fn regen_system(time: Res<Time>, paused: Res<WavePaused>, mut q: Query<(&mut Health, &MaxHealth, &mut Regen), Without<crate::components::Cursed>>) {
    if paused.0 { return; }
    let dt = time.delta_secs();
    for (mut hp, max_hp, mut r) in q.iter_mut() {
        if hp.0 < r.prev - 0.001 { r.idle = 0.0; } else { r.idle += dt; }
        if r.idle >= gdd::WIFI_REGEN_IDLE && hp.0 > 0.0 && hp.0 < max_hp.0 {
            hp.0 = (hp.0 + max_hp.0 * gdd::WIFI_REGEN * dt).min(max_hp.0);
        }
        r.prev = hp.0;
    }
}

// aura rychlosti (Influencer, Sef z Prace): creepove v radiu jdou rychleji (bere se nejvetsi bonus)
pub fn speed_aura_system(
    mut commands: Commands,
    grid: Res<crate::resources::CreepGrid>,
    mut near: Local<Vec<(Entity, Vec2)>>,
    mut bonus: Local<std::collections::HashMap<Entity, f32>>,
    auras: Query<(Entity, &Transform, &SpeedAura), Without<crate::components::Cursed>>,
    mut creeps: Query<(Entity, Option<&mut AuraSpeed>), With<Creep>>,
) {
    bonus.clear();
    for (a_ent, tf, aura) in auras.iter() {
        near.clear();
        grid.query(tf.translation.truncate(), aura.radius, &mut near);
        for &(e, _) in near.iter() {
            if e == a_ent { continue; }
            let b = bonus.entry(e).or_insert(0.0);
            *b = b.max(aura.bonus);
        }
    }
    for (e, cur) in creeps.iter_mut() {
        let want = bonus.get(&e).copied().unwrap_or(0.0);
        match cur {
            Some(mut c) => { if c.0 != want { c.0 = want; } }
            None if want > 0.0 => { commands.entity(e).try_insert(AuraSpeed(want)); }
            None => {}
        }
    }
}

// Lagujici Brambora: kazde 3 s nahodna rychlost 0,8-1,3x
pub fn random_speed_system(time: Res<Time>, paused: Res<WavePaused>, mut q: Query<&mut RandomSpeed>) {
    if paused.0 { return; }
    let mut rng = rand::thread_rng();
    for mut r in q.iter_mut() {
        if r.timer.tick(time.delta()).just_finished() { r.mul = rng.gen_range(gdd::BRAMBORA_SPEED.0..gdd::BRAMBORA_SPEED.1); }
    }
}

// bossove (GDD 11): Pan Ucitel si kazdych 8 s na 2 s zvedne fyzickou odolnost, Mama ma tri faze
pub fn boss_system(
    mut commands: Commands,
    time: Res<Time>,
    paused: Res<WavePaused>,
    wave: Res<WaveCount>,
    assets: Res<CreepAssets>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut bosses: Query<(Entity, &Transform, &Side, &mut Boss, &mut Creep, &Health, &MaxHealth, &mut DmgTarget, Option<&mut Shield>), Without<crate::components::Cursed>>,
) {
    if paused.0 { return; }
    let dt = time.delta_secs();
    for (e, tf, &side, mut boss, mut creep, hp, max_hp, mut target, shield) in bosses.iter_mut() {
        match boss.kind {
            BossKind::Ucitel => {
                if boss.a.tick(time.delta()).just_finished() { boss.guard_left = gdd::UCITEL_GUARD.1; }
                boss.guard_left = (boss.guard_left - dt).max(0.0);
                target.0.phys_guard = if boss.guard_left > 0.0 { gdd::UCITEL_GUARD.2 } else { 0.0 };
            }
            BossKind::Mama => {
                let frac = hp.0 / max_hp.0.max(1.0);
                let phase = if frac > gdd::MAMA_PHASES.0 { 1 } else if frac > gdd::MAMA_PHASES.1 { 2 } else { 3 };
                if phase == 3 && boss.phase != 3 {
                    creep.speed_mul += gdd::MAMA_RAGE.0;
                    target.0.true_bonus = gdd::MAMA_RAGE.1;
                }
                if phase != boss.phase {
                    boss.phase = phase;
                    let every = if phase == 2 { gdd::MAMA_SHIELD.0 } else { gdd::MAMA_SUMMON_EVERY };
                    boss.b = Timer::from_seconds(every, TimerMode::Repeating);
                }
                if !boss.b.tick(time.delta()).just_finished() { continue; }
                if phase == 1 {
                    // privola maleho Kopirovaciho Bota (20 % HP maleho creepa vlny)
                    let small = wave_def(wave.0.max(1)).small.2;
                    let pos = tf.translation.truncate() + Vec2::new(0.0, 20.0);
                    spawn_creep(&mut commands, &assets, &mut meshes, &mut materials, &asset_server, side, CreepSpec { ctype: CreepType::KopirBot, role: CreepRole::Small, boss: None, hp: small * gdd::KOPIR_COPY_HP, leak: gdd::LEAK_SMALL, kill_gold: 1, sent: false, copy: true, pos });
                } else if phase == 2 {
                    let add = max_hp.0 * gdd::MAMA_SHIELD.1;
                    match shield {
                        Some(mut s) => { s.hp += add; }
                        None => { commands.entity(e).try_insert(Shield { hp: add, prev: hp.0 }); }
                    }
                }
            }
            _ => {}
        }
    }
}

const ENRAGE_FLASH_SECS: f32 = 0.6; // jak dlouho cervene probliknuti trva
const ENRAGE_BLINKS: f32 = 3.0;     // kolikrat za tu dobu probliknou

// blikani cerveneho kolecka pres telo; po dobehnuti casu se smaze (jen pri enrage, ne kazdy snimek u vsech)
pub fn enrage_flash_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut EnrageFlash, &mut Visibility)>,
) {
    for (e, mut f, mut vis) in q.iter_mut() {
        f.timer.tick(time.delta());
        if f.timer.finished() {
            if let Some(mut ec) = commands.get_entity(e) { ec.despawn(); } // get_entity: rodic mohl byt mezitim smazan
            continue;
        }
        let on = (f.timer.fraction() * ENRAGE_BLINKS * 2.0) as u32 % 2 == 0; // sudy = svici, lichy = zhasnuto
        let want = if on { Visibility::Visible } else { Visibility::Hidden };
        if *vis != want { *vis = want; }
    }
}

// ENRAGE: kdyz HP klesne pod hranici z tabulky (wave_def::enrage_for), jednotka jednou zrychli a zvetsi se; pak se komponenta odebere (system ji dal nehlida)
pub fn enrage_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<CreepAssets>,
    mut q: Query<(Entity, &Health, &MaxHealth, &Enrage, &mut Creep, Option<&mut Sprite>, Option<&mut Mesh2d>)>,
) {
    for (e, hp, max_hp, enrage, mut creep, sprite, mesh) in q.iter_mut() {
        if hp.0 <= 0.0 || hp.0 > max_hp.0 * enrage.hp_frac { continue; } // mrtva se preskoci, zdrava ceka
        creep.speed_mul += enrage.speed_bonus; // +30 % = 1.0 -> 1.3
        // zvetseni JEN tela (HP bar a text zustavaji): obrazek pres custom_size, kolecko novym meshem (jen pri enrage = zridka)
        if enrage.size_bonus > 0.0 {
            let new_r = creep.base_radius * (1.0 + enrage.size_bonus);
            creep.base_radius = new_r; // obrys slow a castice se ridi touhle hodnotou
            if let Some(mut s) = sprite { s.custom_size = Some(Vec2::splat(new_r * 2.0)); }
            else if let Some(mut m) = mesh { m.0 = meshes.add(Circle::new(new_r)); }
        }
        // cervene probliknuti: kolecko pres telo (sdileny vyplneny kruh), barva z tabulky
        let flash_r = creep.base_radius;
        let flash_mat = materials.add(ColorMaterial::from_color(enrage.flash));
        commands.entity(e).with_children(|p| {
            p.spawn((
                Mesh2d(assets.shadow_mesh.clone()),
                MeshMaterial2d(flash_mat),
                Transform::from_xyz(0.0, 0.0, 0.1).with_scale(Vec3::new(flash_r, flash_r, 1.0)),
                EnrageFlash { timer: Timer::from_seconds(ENRAGE_FLASH_SECS, TimerMode::Once) },
            ));
        });
        if let Some(mut ec) = commands.get_entity(e) { ec.remove::<Enrage>(); }
    }
}

pub fn update_outline(
    q_creeps: Query<(&Children, &Creep, Option<&SlowDebuff>), With<Creep>>,
    mut q_slow_outline: Query<(&mut Visibility, &mut Transform), With<SlowOutline>>,
) {
    for (children, creep, slow_opt) in q_creeps.iter() {
        let current_radius = creep.base_radius;
        for &child in children.iter() {
            if let Ok((mut vis, mut tf)) = q_slow_outline.get_mut(child) {
                if let Some(slow) = slow_opt {
                    // sdileny kruh o polomeru 1 -> scale = polomer (zadny novy mesh kazdy snimek)
                    let total_radius = (current_radius + FROST_OUTLINE_MAX_EXTRA * slow.timer.fraction_remaining()).max(0.1);
                    tf.scale = Vec3::new(total_radius, total_radius, 1.0);
                    if *vis != Visibility::Visible { *vis = Visibility::Visible; }
                } else if *vis != Visibility::Hidden { *vis = Visibility::Hidden; }
            }
        }
    }
}

pub fn update_hp_display(
    q_creeps: Query<(Ref<Health>, &Children, &Creep, Option<&SlowDebuff>, Option<&BurnDebuff>), With<Creep>>,
    mut q_hp: Query<&mut Text2d, (With<CreepHpText>, Without<CreepStatusText>, Without<SlowOutline>, Without<HpBarBg>, Without<HpBarFill>)>,
    mut q_status: Query<&mut Text2d, (With<CreepStatusText>, Without<CreepHpText>)>,
) {
    for (hp, children, creep, slow_opt, burn_opt) in q_creeps.iter() {
        let hp_changed = hp.is_changed(); // HP text prepisovat jen kdyz se zmenilo HP
        let status = match (slow_opt.is_some(), burn_opt.is_some()) { (true, true) => "SLOW BURN ", (true, false) => "SLOW ", (false, true) => "BURN ", _ => "" };
        for &child in children.iter() {
            if hp_changed {
                if let Ok(mut txt) = q_hp.get_mut(child) {
                    let s = if creep.is_mega { format!("MEGA {:.0}", hp.0.max(0.0)) } else { format!("{:.0}", hp.0.max(0.0)) };
                    if txt.0 != s { txt.0 = s; } // stejny text = zadny zapis = zadny relayout textu
                }
            }
            if let Ok(mut txt) = q_status.get_mut(child) {
                if txt.0 != status { txt.0 = status.to_string(); }
            }
        }
    }
}

pub fn update_hp_bar(
    q_creeps: Query<(Ref<Health>, &MaxHealth, &Children), With<Creep>>,
    mut q_fill: Query<(&mut Sprite, &mut Transform), (With<HpBarFill>, Without<HpBarBg>)>,
) {
    for (hp, max_hp, children) in q_creeps.iter() {
        if !hp.is_changed() { continue; } // bar prekreslit jen kdyz se zmenilo HP
        let frac = (hp.0 / max_hp.0.max(1.0)).clamp(0.0, 1.0);
        for &child in children.iter() {
            if let Ok((mut sprite, _tf)) = q_fill.get_mut(child) {
                let full_w = 32.0;
                sprite.custom_size = Some(Vec2::new(full_w * frac, 4.0));
                // barva: zelena -> zluta -> cervena
                let col = if frac > 0.6 { Color::srgb(0.2, 0.9, 0.2) } else if frac > 0.3 { Color::srgb(0.9, 0.9, 0.2) } else { Color::srgb(0.9, 0.2, 0.2) };
                sprite.color = col;
            }
        }
    }
}

// smrt a pruchod creepu (GDD 2 a 4): kill gold podle typu, boss = diamant (nebo 400 g) + 150 g, Kopirovaci Bot se rozpadne,
// pruchod = pretahovana o zivoty (od vlny 26 bere aspon 2 a dava o 1 mene), propusteny boss da diamant souperi,
// po vlne 30 pri sudden death konci zapas prvnim pruchodem
pub fn despawn_dead_and_count(
    mut commands: Commands,
    mut sides: ResMut<Sides>,
    mut over: ResMut<MatchOver>,
    mut stats: ResMut<MatchStats>,
    wave: Res<WaveCount>,
    time: Res<Time>,
    assets: Res<CreepAssets>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    towers: Query<&Tower>,
    q_creeps: Query<(Entity, &Transform, &Health, &MaxHealth, &Creep, &Side, Has<SentUnit>, Option<&crate::components::DeathExplosion>, Has<CopyOnDeath>, Option<&Boss>), With<Creep>>,
    mode: Res<GameMode>,
) {
    let mut blasts = Vec::new();
    let now = time.elapsed_secs();
    for (e, tf, hp, max_hp, creep, &side, sent, death_explosion, copies, boss) in q_creeps.iter() {
        let pos = tf.translation.truncate();
        if hp.0 <= 0.0 {
            commands.entity(e).despawn_recursive();
            if let Some(blast) = death_explosion { blasts.push((e, side, pos, blast.damage, blast.radius)); }
            // Kopirovaci Bot: 2 kopie s 20 % max HP (kopie se uz dal nekopiruji)
            if copies {
                for i in 0..gdd::KOPIR_COPIES {
                    let off = Vec2::new(if i == 0 { -8.0 } else { 8.0 }, 4.0);
                    spawn_creep(&mut commands, &assets, &mut meshes, &mut materials, &asset_server, side, CreepSpec { ctype: CreepType::KopirBot, role: CreepRole::Small, boss: None, hp: max_hp.0 * gdd::KOPIR_COPY_HP, leak: gdd::LEAK_SMALL, kill_gold: if sent { 0 } else { 1 }, sent, copy: true, pos: pos + off });
                }
            }
            // gold dostane ten, na ci lajne creep padl; poslane jednotky gold nedavaji
            if sent { continue; }
            let st = sides.get_mut(side);
            let mul = if st.gold_boost_timer > 0.0 { 2 } else { 1 };
            st.gold += creep.kill_gold * mul;
            if boss.is_some() {
                award_boss_diamond(&mut sides, &towers, side);
                if wave.0 >= gdd::LAST_WAVE { stats.last_boss_kill[side.idx()] = Some(now); }
            }
        } else if tf.translation.y < LANE_Y - LANE_H / 2.0 - 10.0 {
            commands.entity(e).despawn_recursive();
            if over.0.is_some() || mode.is_test() { continue; } // po konci zapasu / v testu se uz nic nepocita
            let (loser, winner) = (side, side.other());
            stats.leaks[loser.idx()] += 1;
            if stats.sudden_death {
                over.0 = Some(winner);
                println!("SUDDEN DEATH: prosel creep na lajne {:?} - vitez {:?}", loser, winner);
                continue;
            }
            let (lost, gained) = gdd::leak_lives(creep.leak_cost, wave.0);
            let lost = lost.min(sides.get(loser).lives);
            sides.get_mut(loser).lives -= lost;
            sides.get_mut(winner).lives = (sides.get(winner).lives + gained.min(lost)).min(TOTAL_LIVES);
            if let Some(b) = boss {
                award_boss_diamond(&mut sides, &towers, winner); // propusteny boss: diamant dostane souper
                if b.kind == BossKind::DanovyUrad { let st = sides.get_mut(loser); st.gold = (st.gold - gdd::URAD_LEAK_GOLD).max(0); }
            }
            if sides.get(loser).lives == 0 && std::env::var("SIM_ENDLESS").is_err() { // SIM_ENDLESS=1: simulace bot vs bot nekonci
                over.0 = Some(winner);
                println!("KONEC ZAPASU - vitez {:?}", winner);
            }
            println!("PROSLO na lajne {:?}! Zivoty: hrac {} / bot {}", side, sides.get(Side::Player).lives, sides.get(Side::Bot).lives);
        }
    }
    // smrtici exploze (Toustovac R7): zasahne okolni creepy te same lajny
    for (origin, side, pos, damage, radius) in blasts {
        for (target, tf, hp, _, _, &target_side, _, _, _, _) in q_creeps.iter() {
            if target == origin || hp.0 <= 0.0 || target_side != side { continue; }
            if tf.translation.truncate().distance_squared(pos) <= radius * radius {
                commands.entity(target).insert(Health(hp.0 - damage));
            }
        }
    }
}

// boss (zabity nebo propusteny) da diamant; kdo uz ma diamanty na obe Special veze az na R3, dostane misto nej 400 goldu
pub fn award_boss_diamond(sides: &mut Sides, towers: &Query<&Tower>, side: Side) {
    let st = sides.get_mut(side);
    let specials: Vec<u8> = st.deck.iter().copied().filter(|&k| crate::constants::is_special_tower(k)).collect();
    let ranks: Vec<u32> = specials.iter().map(|&k| towers.iter().filter(|t| t.side == side && t.kind == k).map(|t| t.level).max().unwrap_or(0)).collect();
    if st.diamonds < gdd::diamonds_needed(&ranks) {
        st.diamonds += 1;
        println!("{:?} ziskal diamant (ma {})", side, st.diamonds);
    } else {
        st.gold += gdd::DIAMOND_GOLD;
        println!("{:?} uz diamant nepotrebuje: +{}G", side, gdd::DIAMOND_GOLD);
    }
}

// Startup: sdilene meshe/materialy creepu
pub fn setup_creep_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(CreepAssets {
        unit_circle: meshes.add(Annulus::new(0.88, 1.0)), // tenky kruh (obrys), velikost se dela pres scale
        slow_mat: materials.add(ColorMaterial::from_color(Color::srgba(0.2, 0.6, 1.0, 0.85))),
        shadow_mesh: meshes.add(Circle::new(1.0)), // vyplneny kruh, z nej se scale dela elipsa
    });
}

// === POSILANE JEDNOTKY (GDD 4.4, panel S) ===

// muze strana poslat jednotku? (limity 12 za vlnu a 2 Mini-Bossove za vlnu; v testu bez limitu)
pub fn can_send(stats: &MatchStats, side: Side, kind: u8) -> bool {
    stats.sends[side.idx()] < gdd::MAX_SENDS_PER_WAVE
        && (!send_def(kind).mini_boss || stats.mini_bosses[side.idx()] < gdd::MAX_MINI_BOSSES_PER_WAVE)
}

// zaplati, zapocita limit a income (vyplati se az pri dalsim ticku) a posle jednotku na lajnu `target`
pub fn send_unit(commands: &mut Commands, assets: &CreepAssets, meshes: &mut Assets<Mesh>, materials: &mut Assets<ColorMaterial>, asset_server: &AssetServer, sides: &mut Sides, stats: &mut MatchStats, from: Side, target: Side, kind: u8, wave: u32) {
    let def = send_def(kind);
    let st = sides.get_mut(from);
    st.gold -= def.cost;
    st.income.amount += def.income as f32;
    stats.sends[from.idx()] += 1;
    if def.mini_boss { stats.mini_bosses[from.idx()] += 1; }
    let x = target.lane_x() + rand::thread_rng().gen_range(-LANE_W * 0.35..LANE_W * 0.35);
    let pos = Vec2::new(x, LANE_Y + LANE_H / 2.0 + SPAWN_ABOVE);
    let role = if def.mini_boss { CreepRole::MiniBoss } else if def.leak >= 2 { CreepRole::Big } else { CreepRole::Small };
    spawn_creep(commands, assets, meshes, materials, asset_server, target, CreepSpec { ctype: def.creep, role, boss: None, hp: send_hp(kind, wave), leak: def.leak, kill_gold: 0, sent: true, copy: false, pos });
}

// klepnuti na jednotku v panelu S: odesle se az PO PUSTENI prstu (a jen kdyz se prst netahl = neposouval panel)
const TAP_MOVE_MAX: f32 = 8.0; // vic pixelu pohybu prstu nez tohle = tah (posouvani), ne klepnuti
pub fn send_unit_system(
    mut commands: Commands,
    q_items: Query<(&Interaction, &SendPanelItem), Changed<Interaction>>,
    touches: Res<Touches>,
    mut pressed: Local<Option<u8>>, // ktera jednotka je prave stisknuta
    mut moved: Local<f32>,          // kolik se prst pohnul od stisku
    mut sides: ResMut<Sides>,
    mut stats: ResMut<MatchStats>,
    round: Res<RoundState>,
    wave: Res<WaveCount>,
    over: Res<MatchOver>,
    assets: Res<CreepAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mode: Res<GameMode>,
) {
    for t in touches.iter() { *moved += t.delta().length(); }
    for (interaction, item) in q_items.iter() {
        let mut kind: Option<u8> = None;
        if *interaction == Interaction::Pressed {
            *pressed = Some(item.0);
            *moved = 0.0;
        } else if *pressed == Some(item.0) {
            *pressed = None;
            if *moved <= TAP_MOVE_MAX { kind = Some(item.0); }
        }
        let Some(kind) = kind else { continue; };
        if over.0.is_some() { continue; }
        // v testu kdykoli, bez limitu a na vlastni lajnu (zkouseni vezi)
        if mode.is_test() {
            let mut st = MatchStats::default();
            send_unit(&mut commands, &assets, &mut meshes, &mut materials, &asset_server, &mut sides, &mut st, Side::Player, Side::Player, kind, wave.0);
            continue;
        }
        if round.phase != RoundPhase::Running { println!("Posilat jde jen behem kola"); continue; }
        let def = send_def(kind);
        if sides.get(Side::Player).gold < def.cost { println!("Malo gold na {}: potreba {}", def.name, def.cost); continue; }
        if !can_send(&stats, Side::Player, kind) { println!("Limit posilani v teto vlne (max {}, Mini-Boss max {})", gdd::MAX_SENDS_PER_WAVE, gdd::MAX_MINI_BOSSES_PER_WAVE); continue; }
        send_unit(&mut commands, &assets, &mut meshes, &mut materials, &asset_server, &mut sides, &mut stats, Side::Player, Side::Bot, kind, wave.0);
        println!("Odeslana jednotka {} za {}G, income {:.0} / tick", def.name, def.cost, sides.get(Side::Player).income.amount);
    }
}

// === ZELENE KOLECKO KOLEM LECENEHO CREEPA (pool, zadny spawn/despawn za behu) ===
const HEAL_RING_POOL_SIZE: usize = 60;
const HEAL_RING_SECS: f32 = 0.45;  // jak dlouho kolecko zije
const HEAL_RING_EXTRA: f32 = 14.0; // o kolik je na zacatku vetsi nez telo (pak se zmensi na telo)

#[derive(Component)]
pub struct HealRing { follow: Entity, timer: Timer, base_r: f32, active: bool }

#[derive(Resource, Default)]
pub struct HealRingPool { pub free: Vec<Entity> }

// Startup: predvytvori skryta kolecka (vlastni mesh/material, nezavisi na CreepAssets)
pub fn setup_heal_ring_pool(mut commands: Commands, mut pool: ResMut<HealRingPool>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    let mesh = meshes.add(Annulus::new(0.85, 1.0));
    let mat = materials.add(ColorMaterial::from_color(Color::srgba(0.3, 1.0, 0.4, 0.85)));
    for _ in 0..HEAL_RING_POOL_SIZE {
        let e = commands.spawn((
            Mesh2d(mesh.clone()),
            MeshMaterial2d(mat.clone()),
            Transform::from_xyz(0.0, 0.0, 4.7),
            Visibility::Hidden,
            HealRing { follow: Entity::PLACEHOLDER, timer: Timer::from_seconds(HEAL_RING_SECS, TimerMode::Once), base_r: 1.0, active: false },
        )).id();
        pool.free.push(e);
    }
}

// kolecko jede s creepem a zmensuje se na jeho telo; po case (nebo kdyz creep zmizi) zpet do poolu
pub fn heal_ring_update_system(
    time: Res<Time>,
    mut pool: ResMut<HealRingPool>,
    creeps: Query<&Transform, (With<Creep>, Without<HealRing>)>,
    mut rings: Query<(Entity, &mut Transform, &mut HealRing, &mut Visibility)>,
) {
    for (e, mut tf, mut ring, mut vis) in rings.iter_mut() {
        if !ring.active { continue; } // volny slot z poolu
        ring.timer.tick(time.delta());
        let Ok(c_tf) = creeps.get(ring.follow) else { ring.active = false; *vis = Visibility::Hidden; pool.free.push(e); continue; };
        if ring.timer.finished() { ring.active = false; *vis = Visibility::Hidden; pool.free.push(e); continue; }
        let r = ring.base_r + HEAL_RING_EXTRA * ring.timer.fraction_remaining();
        tf.translation.x = c_tf.translation.x;
        tf.translation.y = c_tf.translation.y;
        tf.scale = Vec3::new(r, r, 1.0);
    }
}

// === LECENI (Priest) ===
const HEAL_TICK_SECS: f32 = 0.5; // lecit po davkach (mene zmen HP = mene prekreslovani HP baru)

pub fn heal_system(
    time: Res<Time>,
    paused: Res<WavePaused>,
    grid: Res<crate::resources::CreepGrid>,
    mut near: Local<Vec<(Entity, Vec2)>>, // znovupouzity buffer
    mut commands: Commands,
    mut ring_pool: ResMut<HealRingPool>,
    mut priests: Query<(Entity, &Transform, &mut Heal), Without<crate::components::Cursed>>, // prokleti lecitele neleci
    mut creeps: Query<(&mut Health, &MaxHealth, &Creep), With<Creep>>,
) {
    if paused.0 { return; }
    for (p_ent, p_tf, mut heal) in priests.iter_mut() {
        heal.tick.tick(time.delta());
        if !heal.tick.just_finished() { continue; }
        let amount = heal.hps * HEAL_TICK_SECS;
        near.clear();
        grid.query(p_tf.translation.truncate(), heal.radius, &mut near); // prostorova mrizka z minuleho snimku staci
        for &(c_ent, c_pos) in near.iter() {
            if c_ent == p_ent { continue; } // sebe nelecit
            if let Ok((mut hp, max_hp, creep)) = creeps.get_mut(c_ent) {
                if hp.0 > 0.0 && hp.0 < max_hp.0 {
                    hp.0 = (hp.0 + amount).min(max_hp.0); // zapis jen kdyz je co lecit
                    // zelene kolecko kolem lecene jednotky (z poolu; pozice se nastavi hned, aby neblikla na (0,0))
                    if let Some(ring) = ring_pool.free.pop() {
                        commands.entity(ring).insert((
                            Visibility::Visible,
                            Transform::from_xyz(c_pos.x, c_pos.y, 4.7).with_scale(Vec3::new(creep.base_radius, creep.base_radius, 1.0)),
                            HealRing { follow: c_ent, timer: Timer::from_seconds(HEAL_RING_SECS, TimerMode::Once), base_r: creep.base_radius, active: true },
                        ));
                    }
                }
            }
        }
    }
}

// === AURA (Captain) ===
const AURA_TICK_SECS: f32 = 0.5; // jak casto se aura prepocitava

// kazdych 0.5 s: creepi v radiu Captaina dostanou +hp_bonus max HP (jednou); kdo z aury vyjde / Captain zmizi, bonus ztrati
pub fn aura_system(
    mut commands: Commands,
    time: Res<Time>,
    paused: Res<WavePaused>,
    grid: Res<crate::resources::CreepGrid>,
    mut acc: Local<f32>,
    mut near: Local<Vec<(Entity, Vec2)>>,   // znovupouzity buffer
    mut covered: Local<Vec<(Entity, f32)>>, // (creep v aure, bonus)
    captains: Query<(Entity, &Transform, &Aura), Without<crate::components::Cursed>>, // prokleta aura nefunguje
    mut creeps: Query<(Entity, &mut Health, &mut MaxHealth, Option<&AuraBuff>), With<Creep>>,
) {
    if paused.0 { return; }
    *acc += time.delta_secs();
    if *acc < AURA_TICK_SECS { return; }
    *acc = 0.0;
    covered.clear();
    for (c_ent, c_tf, aura) in captains.iter() {
        near.clear();
        grid.query(c_tf.translation.truncate(), aura.radius, &mut near);
        for &(e, _) in near.iter() {
            if e != c_ent && !covered.iter().any(|(ce, _)| *ce == e) { covered.push((e, aura.hp_bonus)); } // Captain sam bonus nema
        }
    }
    for (e, mut hp, mut max_hp, buff) in creeps.iter_mut() {
        let bonus = covered.iter().find(|(ce, _)| *ce == e).map(|(_, b)| *b);
        match (bonus, buff) {
            // vesel do aury a jeste bonus nema: max HP nahoru, bonus HP rovnou plne (mrtvou jednotku neoziveni)
            (Some(b), None) if hp.0 > 0.0 => {
                let delta = max_hp.0 * b;
                max_hp.0 += delta;
                hp.0 += delta;
                if let Some(mut ec) = commands.get_entity(e) { ec.try_insert(AuraBuff { delta }); }
            }
            // vysel z aury / Captain zmizel: bonus pryc, HP se oseknou na nove max
            (None, Some(buff)) => {
                max_hp.0 -= buff.delta;
                if hp.0 > max_hp.0 { hp.0 = max_hp.0; }
                if let Some(mut ec) = commands.get_entity(e) { ec.remove::<AuraBuff>(); }
            }
            _ => {}
        }
    }
}

// === ZPETNA STRELBA (Frost Archer) ===
// kazdych interval s: nejblizsi vez v dosahu dostane TowerSlow (pomalejsi strelba) + svetla car archer -> vez
pub fn shoot_back_system(
    mut commands: Commands,
    time: Res<Time>,
    paused: Res<WavePaused>,
    mut bolt_pool: ResMut<crate::systems::effects::BoltPool>,
    mut archers: Query<(&Transform, &mut ShootBack)>,
    assets: Res<CreepAssets>,
    towers: Query<(Entity, &Transform, Has<TowerSlow>), (With<Tower>, Without<ShootBack>)>,
) {
    if paused.0 { return; }
    for (a_tf, mut sb) in archers.iter_mut() {
        sb.tick.tick(time.delta());
        if !sb.tick.just_finished() { continue; }
        let a_pos = a_tf.translation.truncate();
        let mut best: Option<(Entity, Vec2, f32, bool)> = None; // (vez, pozice, vzdalenost, uz zpomalena)
        for (t_ent, t_tf, slowed) in towers.iter() {
            let t_pos = t_tf.translation.truncate();
            let d = a_pos.distance(t_pos);
            if d <= sb.range && best.map_or(true, |b| d < b.2) { best = Some((t_ent, t_pos, d, slowed)); }
        }
        let Some((t_ent, t_pos, _, slowed)) = best else { continue; };
        if let Some(mut ec) = commands.get_entity(t_ent) {
            ec.try_insert(TowerSlow { timer: Timer::from_seconds(sb.secs, TimerMode::Once), factor: 1.0 - sb.slow });
            if !slowed {
                // znazorneni zmrazeni: ledovy ctverec + modry kruh kolem veze (jen pri prvnim zasahu; obnova jen prodlouzi cas)
                let ring = assets.unit_circle.clone();
                let ring_mat = assets.slow_mat.clone();
                ec.with_children(|p| {
                    p.spawn((Sprite { color: Color::srgba(0.6, 0.85, 1.0, 0.4), custom_size: Some(Vec2::splat(CELL - 4.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 0.5), FrostOverlay));
                    p.spawn((Mesh2d(ring), MeshMaterial2d(ring_mat), Transform::from_xyz(0.0, 0.0, 0.6).with_scale(Vec3::new(CELL * 0.55, CELL * 0.55, 1.0)), FrostOverlay));
                });
            }
        }
        crate::systems::effects::spawn_bolt(&mut commands, &mut bolt_pool, a_pos, t_pos);
    }
}

// po uplynuti casu se slow veze odebere
pub fn tower_slow_system(
    mut commands: Commands,
    time: Res<Time>,
    overlays: Query<(), With<FrostOverlay>>,
    mut q: Query<(Entity, &mut TowerSlow, Option<&Children>)>,
) {
    for (e, mut s, children) in q.iter_mut() {
        s.timer.tick(time.delta());
        if s.timer.finished() {
            if let Some(mut ec) = commands.get_entity(e) { ec.remove::<TowerSlow>(); }
            // zmrazeni konci: smazat ledove znazorneni (child entity)
            if let Some(children) = children {
                for &c in children.iter() {
                    if overlays.contains(c) {
                        if let Some(mut ec) = commands.get_entity(c) { ec.despawn(); }
                    }
                }
            }
        }
    }
}

// === SHIMMER: pulzovani kruhu okolo tela (jen scale, zadny nowy mesh) ===
pub fn shimmer_system(time: Res<Time>, mut q: Query<(&Shimmer, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (s, mut tf) in q.iter_mut() {
        let k = 1.15 + 0.15 * (t * std::f32::consts::TAU / s.period.max(0.1)).sin(); // pulzuje 1.0 .. 1.3 nasobek polomeru
        let r = s.base_r * k;
        tf.scale = Vec3::new(r, r, 1.0);
    }
}
// === HLIDAC PEPA (GDD 3) ===
// Pepa kontroluje (BFS), ze z horni rady vede cesta do cile. Kdyz to nekdo zazdi, ozve se a ma 10 s na opravu.
// Kdyz to neopravi, hra mu sama proda nejdrazsi vez, ktera cestu blokuje (za normalni vykupni cenu).
pub const PEPA_FIX_SECS: f32 = 10.0;
const PEPA_QUOTES: [&str; 4] = ["HELE, TADY TO NEJDE!", "Tady by neprosla ani babicka s nakupem.", "Kdo to tady zastavel, to je fakt troll.", "Tudy fakt ne, sefe."];

// je lajna zazdena? (zadna volna horni bunka nema cestu do cile)
pub fn lane_blocked(grid: &crate::resources::Grid, path: &crate::resources::PathMap) -> bool {
    !(0..COLS).any(|c| !grid.blocked[ROWS - 1][c] && path.dist[ROWS - 1][c].is_some())
}

// veze na nejlevnejsi ceste z horni rady do cile, kde pruchod vezi stoji 1 (volna bunka 0): tyhle cestu blokuji
pub fn blocking_cells(blocked: &[[bool; COLS]; ROWS]) -> Vec<(usize, usize)> {
    use std::collections::VecDeque;
    let mut cost = [[i32::MAX; COLS]; ROWS];
    let mut parent: [[Option<(usize, usize)>; COLS]; ROWS] = [[None; COLS]; ROWS];
    let mut dq: VecDeque<(usize, usize)> = VecDeque::new();
    for c in 0..COLS {
        let w = if blocked[0][c] { 1 } else { 0 };
        cost[0][c] = w;
        if w == 0 { dq.push_front((0, c)); } else { dq.push_back((0, c)); }
    }
    while let Some((r, c)) = dq.pop_front() {
        for (dr, dc) in [(1isize, 0isize), (-1, 0), (0, 1), (0, -1)] {
            let (nr, nc) = (r as isize + dr, c as isize + dc);
            if nr < 0 || nr >= ROWS as isize || nc < 0 || nc >= COLS as isize { continue; }
            let (nr, nc) = (nr as usize, nc as usize);
            let w = if blocked[nr][nc] { 1 } else { 0 };
            let nd = cost[r][c] + w;
            if nd < cost[nr][nc] {
                cost[nr][nc] = nd;
                parent[nr][nc] = Some((r, c));
                if w == 0 { dq.push_front((nr, nc)); } else { dq.push_back((nr, nc)); }
            }
        }
    }
    let Some(best_c) = (0..COLS).min_by_key(|&c| cost[ROWS - 1][c]) else { return Vec::new(); };
    let mut out = Vec::new();
    let mut cur = Some((ROWS - 1, best_c));
    while let Some((r, c)) = cur {
        if blocked[r][c] { out.push((r, c)); }
        cur = parent[r][c];
    }
    out
}

pub fn pepa_system(
    mut commands: Commands,
    time: Res<Time>,
    paused: Res<WavePaused>,
    round: Res<RoundState>,
    mut sides: ResMut<Sides>,
    mut selected: ResMut<SelectedCell>,
    mut timers: Local<[Option<f32>; 2]>, // od kdy je lajna zazdena (sekundy do prodeje)
    towers: Query<(Entity, &Tower), Without<Selling>>,
    mut msg: Query<(&mut GuardianMsg, &mut Visibility)>,
    mut msg_text: Query<&mut Text, With<GuardianMsgText>>,
) {
    // hlaska po case zmizi
    for (mut m, mut vis) in msg.iter_mut() {
        if *vis == Visibility::Visible {
            m.timer.tick(time.delta());
            if m.timer.finished() { *vis = Visibility::Hidden; }
        }
    }
    if paused.0 { return; }
    for side in Side::ALL {
        let st = sides.get(side);
        if !lane_blocked(&st.grid, &st.path) { timers[side.idx()] = None; continue; }
        let left = timers[side.idx()].get_or_insert_with(|| {
            if side == Side::Player {
                let q = PEPA_QUOTES[rand::thread_rng().gen_range(0..PEPA_QUOTES.len())];
                show_msg(&mut msg, &mut msg_text, &format!("Hlidac Pepa: {} Mas {:.0} s na opravu.", q, PEPA_FIX_SECS));
            }
            PEPA_FIX_SECS
        });
        *left -= time.delta_secs();
        if *left > 0.0 { continue; }
        // cas vyprsel: prodat nejdrazsi blokujici vez
        let cells = blocking_cells(&st.grid.blocked);
        let pick = towers.iter().filter(|(_, t)| t.side == side && cells.contains(&(t.row, t.col))).max_by_key(|(_, t)| t.invested);
        let Some((e, t)) = pick else { timers[side.idx()] = None; continue; };
        let price = crate::systems::tower_build::sell_price(t.invested, round.phase);
        let st = sides.get_mut(side);
        st.grid.blocked[t.row][t.col] = false;
        st.path.dist = compute_path(&st.grid.blocked);
        st.gold += price;
        commands.entity(e).despawn_recursive();
        if side == Side::Player && selected.0 == Some((t.row, t.col)) { selected.0 = None; }
        if side == Side::Player { show_msg(&mut msg, &mut msg_text, &format!("Hlidac Pepa prodal blokujici vez za {}G. Tudy fakt ne, sefe.", price)); }
        println!("Pepa prodal blokujici vez {:?} na [{},{}] za {}G", side, t.col, t.row, price);
        timers[side.idx()] = Some(0.0); // kdyby cesta byla zazdena vic vezemi, dalsi se proda hned
    }
}


// Creeps and support towers modify tower stats through TowerMods.
pub fn tower_aura_system(
    mut commands: Commands,
    time: Res<Time>,
    paused: Res<WavePaused>,
    mut stomps: Local<Vec<(Vec2, f32, f32, bool)>>, // (stred, radius, sekundy omraceni, jen 1 nahodna vez) pro tento snimek
    mut auras: Query<(&Transform, &mut TowerAura), Without<crate::components::Cursed>>, // Karen umlci schopnosti bosse
    mut towers: Query<(Entity, &Transform, &Tower, Option<&mut TowerMods>)>,
    supports: Query<(Entity, &Transform, &Tower)>,
) {
    if paused.0 { return; }
    let dt = time.delta_secs();
    stomps.clear();
    for (tf, mut a) in auras.iter_mut() {
        if a.stomp_every > 0.0 {
            a.timer.tick(time.delta());
            if a.timer.just_finished() { stomps.push((tf.translation.truncate(), a.radius, a.stomp_stun, a.stomp_single)); }
        }
    }
    // stomp na 1 nahodnou vez (Danovy Urad, Mama): predem vybrat, kterou
    let mut single_hits: Vec<(Entity, f32)> = Vec::new();
    for &(c, r, s, single) in stomps.iter() {
        if !single { continue; }
        let candidates: Vec<Entity> = towers.iter().filter(|(_, t_tf, _, _)| t_tf.translation.truncate().distance(c) <= r).map(|(e, _, _, _)| e).collect();
        if !candidates.is_empty() { single_hits.push((candidates[rand::thread_rng().gen_range(0..candidates.len())], s)); }
    }
    for (t_ent, t_tf, tower, mods) in towers.iter_mut() {
        let Some(mut mods) = mods else {
            if let Some(mut ec) = commands.get_entity(t_ent) { ec.try_insert(TowerMods::default()); } // vez dostane komponentu, pocita se od pristiho snimku
            continue;
        };
        let t_pos = t_tf.translation.truncate();
        let (mut dmg, mut range, mut rate, mut random) = (1.0_f32, 1.0_f32, 1.0_f32, 0.0_f32);
        for (a_tf, a) in auras.iter() {
            if a_tf.translation.truncate().distance(t_pos) <= a.radius {
                dmg *= a.dmg_mul;
                range *= a.range_mul;
                rate *= a.rate_mul;
                random = random.max(a.random_target);
            }
        }
        let mut support_dmg = 0.0_f32;
        let mut support_rate = 0.0_f32;
        for (support_entity, support_tf, support) in supports.iter() {
            if support.side != tower.side || support_entity == t_ent { continue; }
            if support.kind == crate::special::GIGACHAD
                && support_tf.translation.truncate().distance(t_pos) <= 2.0 * crate::constants::CELL
            {
                support_dmg = support_dmg.max(if support.level >= 3 { 0.20 } else { 0.12 * crate::special::special_ability_multiplier(support.level) });
                support_rate = support_rate.max(if support.level >= 3 { 0.08 } else { 0.0 });
            }
            if support.kind == crate::special::NERFKA
                && support.row.abs_diff(tower.row) <= 1
                && support.col.abs_diff(tower.col) <= 1
            {
                support_rate += 0.05;
            }
        }
        dmg *= 1.0 + support_dmg;
        rate *= 1.0 + support_rate;
        mods.dmg_mul = dmg;
        mods.range_mul = range;
        mods.rate_mul = rate;
        mods.random_target = random;
        mods.stun_left = (mods.stun_left - dt).max(0.0);
        for &(c, r, s, single) in stomps.iter() {
            if !single && c.distance(t_pos) <= r { mods.stun_left = mods.stun_left.max(s); }
        }
        for &(e, s) in single_hits.iter() { if e == t_ent { mods.stun_left = mods.stun_left.max(s); } }
    }
}

// CHEAT (jen pro testovani): stisknuti pricte +50 do income (gold za tik 7 s)
const CHEAT_INCOME: f32 = 50.0;
pub fn cheat_button_system(mut sides: ResMut<Sides>, q: Query<&Interaction, (Changed<Interaction>, With<CheatButton>)>) {
    for i in q.iter() {
        if *i == Interaction::Pressed { sides.get_mut(Side::Player).income.amount += CHEAT_INCOME; }
    }
}

// income: kazdych 7s pripise gold, text nahore
pub fn income_system(
    time: Res<Time>,
    paused: Res<WavePaused>,
    mut sides: ResMut<Sides>,
    mut q_text: Query<&mut Text, With<IncomeText>>,
) {
    // income dostavaji obe strany, kazda podle sveho
    for st in sides.0.iter_mut() {
        let boosted = st.gold_boost_timer > 0.0;
        st.gold_boost_timer = (st.gold_boost_timer - time.delta_secs()).max(0.0);
        if !paused.0 && st.income.timer.tick(time.delta()).just_finished() {
            // GDD 4.1: base + sendy, max 40 za tick (Doge Miner R3 docasne zdvoji)
            let multiplier = if boosted { 2 } else { 1 };
            st.gold += gdd::tick_income(st.income.amount as i32) * multiplier;
        }
    }
    let income = &sides.get(Side::Player).income;
    let s = format!("Income za {:.0}s: +{} (max {})", income.timer.remaining_secs().ceil(), gdd::tick_income(income.amount as i32), gdd::INCOME_CAP); // za kolik s prijde a kolik
    for mut t in q_text.iter_mut() { if t.0 != s { t.0 = s.clone(); } }
}

// TEST rezim: neomezene penize (gold i diamanty se porad doplnuji) a tlacitko VLNA,
// ktere na vlastni lajnu posle dalsi automatickou vlnu (kolo +1)
pub fn test_mode_system(
    mut commands: Commands,
    mut sides: ResMut<Sides>,
    mut wave: ResMut<WaveCount>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<CreepAssets>,
    asset_server: Res<AssetServer>,
    q_wave: Query<&Interaction, (Changed<Interaction>, With<crate::components::TestWaveButton>)>,
) {
    let st = sides.get_mut(Side::Player);
    if st.gold != TEST_GOLD { st.gold = TEST_GOLD; }
    if st.diamonds != TEST_DIAMONDS { st.diamonds = TEST_DIAMONDS; }
    if q_wave.iter().any(|i| *i == Interaction::Pressed) {
        wave.0 += 1;
        spawn_round_wave(&mut commands, &mut meshes, &mut materials, &assets, &asset_server, wave.0, &[Side::Player]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pepa_finds_the_towers_that_wall_off_the_lane() {
        let mut blocked = [[false; COLS]; ROWS];
        assert!(blocking_cells(&blocked).is_empty() || blocking_cells(&blocked).iter().all(|&(r, c)| blocked[r][c]));
        for c in 0..COLS { blocked[5][c] = true; } // cela rada zazdena
        let cells = blocking_cells(&blocked);
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].0, 5);
    }

    #[test]
    fn after_wave_30_more_lives_win_then_leaks_then_boss_time() {
        let mut sides = Sides::default();
        let mut stats = MatchStats::default();
        sides.get_mut(Side::Player).lives = 16;
        sides.get_mut(Side::Bot).lives = 14;
        assert_eq!(decide_after_last_wave(&sides, &stats), Some(Side::Player));
        sides.get_mut(Side::Player).lives = 15;
        sides.get_mut(Side::Bot).lives = 15;
        stats.leaks = [3, 5]; // hraci proslo 3, botovi 5 -> hrac zpusobil vic leaku
        assert_eq!(decide_after_last_wave(&sides, &stats), Some(Side::Player));
        stats.leaks = [4, 4];
        stats.last_boss_kill = [Some(100.0), Some(90.0)];
        assert_eq!(decide_after_last_wave(&sides, &stats), Some(Side::Bot));
        stats.last_boss_kill = [None, None];
        assert_eq!(decide_after_last_wave(&sides, &stats), None); // uplna shoda -> sudden death
    }
}

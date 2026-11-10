
use bevy::prelude::*;
use rand::Rng;
use crate::components::{Creep, CreepMainCircle, Health, MaxHealth, CreepHpText, CreepStatusText, SlowOutline, SlowDebuff, BurnDebuff, HpBarBg, HpBarFill, HpBarText, SentUnit, SendPanelItem, IncomeText, Knockback, Air, Enrage, EnrageFlash, Resist, Guardian, Heal, Tower, Aura, AuraBuff, ShootBack, TowerSlow, FrostOverlay, Shimmer, GuardianMsg, CheatButton, TowerAura, TowerMods};
use crate::resources::{CreepAssets, CreepTimer, WavePaused, WaveCount, Gold, Diamonds, EscapedCount, Grid, PathMap, Income, SelectedCell};
use crate::constants::{LANE_X, LANE_Y, LANE_W, LANE_H, CREEP_BASE_SPEED, CREEP_GROWTH_PER_WAVE, CREEP_GROWTH_MAX_SCALE, FROST_OUTLINE_MAX_EXTRA, COLS, ROWS, MAX_WAVES, LEAK_GOLD_RATIO, LEAK_LIFE_GAIN, CELL, SPAWN_ABOVE};
use crate::wave_def::{get_wave_def, get_unit_def, income_multiplier, is_air, enrage_for, resist_for, heal_for, aura_for, shoot_back_for, shimmer_for, tower_aura_for};
use crate::utils::{world_to_grid, grid_to_world, compute_path};

pub fn spawn_wave_system(
    mut commands: Commands,
    mut timer: ResMut<CreepTimer>,
    time: Res<Time>,
    mut wave: ResMut<WaveCount>,
    paused: Res<WavePaused>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut assets: ResMut<CreepAssets>,
) {
    if paused.0 || wave.0 >= MAX_WAVES { return; } // po posledni vlne uz nic nechodi
    if timer.0.tick(time.delta()).just_finished() {
        wave.0 += 1;
        if wave.0 == 1 { timer.0.set_duration(std::time::Duration::from_secs_f32(15.0)); }
        let wdef = get_wave_def(wave.0);
        println!("Vlna {} - {} count {} scale {:.2}", wave.0, wdef.name, wdef.count, 1.0 + wave.0 as f32 * CREEP_GROWTH_PER_WAVE);

        // HP creepu roste s vlnou (velikost uz ne)
        let wave_scale = (1.0 + wave.0 as f32 * CREEP_GROWTH_PER_WAVE).min(CREEP_GROWTH_MAX_SCALE);

        for i in 0..wdef.count {
            let mut rng = rand::thread_rng();
            let jitter = Vec2::new(rng.gen_range(-18.0..18.0), rng.gen_range(-8.0..8.0));
            let base_x = LANE_X + rng.gen_range(-LANE_W*0.35..LANE_W*0.35);
            let base_y = LANE_Y + LANE_H/2.0 + SPAWN_ABOVE + i as f32 * 22.0;

            let is_boss = i == wdef.count-1 && wdef.boss_hp > 0.0;
            let is_mega = wdef.is_mega && is_boss;

            // vsichni bezni creepi jedna barva, boss/mega maji vlastni
            let creep_color = if is_mega {
                Color::srgb(0.95, 0.2, 0.95)
            } else if is_boss {
                Color::srgb(1.0, 0.2, 0.2)
            } else {
                Color::srgb(0.65, 0.4, 0.9)
            };

            // velikost je pevna (neroste s vlnou), boss je velky
            let (hp, base_radius) = if is_mega {
                (wdef.boss_hp, 40.0)
            } else if is_boss {
                (wdef.boss_hp, 30.0)
            } else {
                let mut h = wdef.hp;
                if wdef.is_sudden { h *= 2.0; }
                (h, 11.0)
            };

            let radius = base_radius;
            let hp_scaled = hp * (if is_boss || is_mega {1.0} else {wave_scale});

            // mesh hlavniho kruhu se sdili podle polomeru, outline meshe/materialy jsou sdilene vsechny
            let circle_mesh = assets.main_meshes.entry((radius * 10.0) as u32).or_insert_with(|| meshes.add(Circle::new(radius))).clone();
            let circle_mat = materials.add(ColorMaterial::from_color(creep_color));
            let unit_mesh = assets.unit_circle.clone();
            let slow_mat = assets.slow_mat.clone();

            commands.spawn((
                Mesh2d(circle_mesh),
                MeshMaterial2d(circle_mat),
                Transform::from_xyz(base_x + jitter.x, base_y + jitter.y, 4.0),
                              Creep { is_boss, is_mega, base_radius, speed_mul: 1.0 },
                CreepMainCircle,
                Health(hp_scaled),
                MaxHealth(hp_scaled),
            )).with_children(|parent| {
                // HP BAR - pozadi
                parent.spawn((
                    Sprite { color: Color::srgb(0.1, 0.1, 0.1), custom_size: Some(Vec2::new(34.0, 6.0)), ..default() },
                    Transform::from_xyz(0.0, radius + 14.0, 0.2),
                    HpBarBg,
                ));
                // HP BAR - fill (zeleny)
                parent.spawn((
                    Sprite { color: Color::srgb(0.2, 0.9, 0.2), custom_size: Some(Vec2::new(32.0, 4.0)), ..default() },
                    Transform::from_xyz(0.0, radius + 14.0, 0.3),
                    HpBarFill,
                ));
                // HP text citelne nad barem
                let label = if is_mega { format!("MEGA {:.0}", hp_scaled) } else if is_boss { format!("{:.0}", hp_scaled) } else { format!("{:.0}", hp_scaled) };
                parent.spawn((
                    Text2d::new(label),
                    TextFont { font_size: if is_mega || is_boss { 12.0 } else { 10.0 }, ..default() },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, radius + 26.0, 0.4),
                    HpBarText,
                    CreepHpText,
                ));
                parent.spawn((Text2d::new(""), TextFont { font_size: 9.0, ..default() }, TextColor(Color::srgb(0.4, 0.8, 1.0)), Transform::from_xyz(0.0, radius + 36.0, 0.4), CreepStatusText));

                let outline_radius = radius + FROST_OUTLINE_MAX_EXTRA;
                parent.spawn((
                    Mesh2d(unit_mesh),
                    MeshMaterial2d(slow_mat),
                    Transform::from_xyz(0.0, 0.0, -0.1).with_scale(Vec3::new(outline_radius, outline_radius, 1.0)),
                    SlowOutline,
                    Visibility::Hidden,
                ));
            });
        }
    }
}

pub fn move_creeps_maze(
    mut commands: Commands,
    mut creeps: Query<(Entity, &mut Transform, &Creep, Option<&SlowDebuff>, Option<&crate::components::StunDebuff>, Option<&mut Knockback>, Has<Air>), (With<Creep>, Without<crate::components::Tower>)>,
    path: Res<PathMap>,
    grid: Res<Grid>,
    time: Res<Time>,
    paused: Res<WavePaused>,
) {
    if paused.0 { return; }
    for (e, mut tf, creep, slow_opt, stun_opt, knock_opt, air) in creeps.iter_mut() {
        // odhozeni dozadu (smerem na start): leti nahoru, dokud neurazi zadanou vzdalenost
        if let Some(mut k) = knock_opt {
            let step = (k.speed * time.delta_secs()).min(k.remaining);
            tf.translation.y += step;
            k.remaining -= step;
            if k.remaining <= 0.0 { commands.entity(e).remove::<Knockback>(); }
            continue;
        }
        if let Some(stun) = stun_opt { if !stun.timer.finished() { continue; } }
        let speed_factor = (if let Some(slow) = slow_opt { slow.factor } else { 1.0 }) * creep.speed_mul; // zpomaleni * rychlost typu jednotky
        // letajici jednotka leti rovne dolu pres bludiste
        if air { tf.translation.y -= CREEP_BASE_SPEED * speed_factor * time.delta_secs(); continue; }
        let pos = tf.translation.truncate();
        let Some((row, col)) = world_to_grid(pos.x, pos.y) else {
            // nad lajnou: dojit vodorovne nad nejblizsi otevreny vstup (horni bunka s cestou), pak dolu; zadny vstup = rovne dolu (zablokovano)
            let step = CREEP_BASE_SPEED * speed_factor * time.delta_secs();
            let mut target_x: Option<f32> = None;
            if pos.y > LANE_Y + LANE_H / 2.0 {
                for c in 0..COLS {
                    if grid.blocked[ROWS - 1][c] || path.dist[ROWS - 1][c].is_none() { continue; }
                    let cx = grid_to_world(ROWS - 1, c).x;
                    if target_x.map_or(true, |t| (cx - pos.x).abs() < (t - pos.x).abs()) { target_x = Some(cx); }
                }
            }
            match target_x {
                Some(tx) if (tx - pos.x).abs() > 3.0 => { tf.translation.x += (tx - pos.x).signum() * step.min((tx - pos.x).abs()); } // vodorovne nad lajnou
                None if pos.y > LANE_Y + LANE_H / 2.0 => { tf.translation.y = (pos.y - step).max(LANE_Y + LANE_H / 2.0 + creep.base_radius); } // zadny vstup (horni rada zazdena): zastavit pred lajnou, creepy nesmi projit vezi
                _ => { tf.translation.y -= step; }
            }
            continue;
        };
        let mut best: Option<(usize, usize, i32)> = None;
        if row == 0 { tf.translation.y -= CREEP_BASE_SPEED * speed_factor * time.delta_secs(); continue; } // posledni rada: rovne ven pod lajnu (jinak by se motal do stran)
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        for (dr, dc) in dirs.iter() {
            let nr = row as isize + dr;
            let nc = col as isize + dc;
            if nr < 0 || nr >= ROWS as isize || nc < 0 || nc >= COLS as isize { continue; }
            let nr = nr as usize; let nc = nc as usize;
            if grid.blocked[nr][nc] { continue; }
            if let Some(d) = path.dist[nr][nc] {
                if best.is_none() || d < best.unwrap().2 { best = Some((nr, nc, d)); }
            }
        }
        if let Some((nr, nc, _)) = best {
            let target = grid_to_world(nr, nc);
            let dir = (target - pos).normalize_or_zero();
            tf.translation.x += dir.x * CREEP_BASE_SPEED * speed_factor * time.delta_secs();
            tf.translation.y += dir.y * CREEP_BASE_SPEED * speed_factor * time.delta_secs();
        } else if row > 0 && !grid.blocked[row - 1][col] {
            tf.translation.y -= CREEP_BASE_SPEED * speed_factor * time.delta_secs();
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

pub fn despawn_dead_and_count(
    mut commands: Commands,
    mut escaped: ResMut<EscapedCount>,
    mut gold: ResMut<Gold>,
    mut diamonds: ResMut<Diamonds>,
    q_creeps: Query<(Entity, &Transform, &Health, &Creep, Option<&SentUnit>), With<Creep>>,
) {
    let mut dead: Vec<(Entity, bool, bool, bool)> = Vec::new(); // (entity, boss, mega, odeslana jednotka)
    let mut out: Vec<(Entity, Option<i32>)> = Vec::new();       // (entity, cena odeslane jednotky; None = bezny creep)
    for (e, tf, hp, creep, unit) in q_creeps.iter() {
        if hp.0 <= 0.0 { dead.push((e, creep.is_boss, creep.is_mega, unit.is_some())); }
        else if tf.translation.y < crate::constants::LANE_Y - crate::constants::LANE_H / 2.0 - 10.0 {
            out.push((e, unit.map(|u| u.cost)));
        }
    }
    for (e, is_boss, is_mega, is_unit) in dead {
        commands.entity(e).despawn_recursive();
        if is_unit { continue; } // odeslana jednotka: za zabiti zadny gold
        if is_mega { gold.0 += 50; diamonds.0 += 1; println!("MEGA BOSS zabit! +50G +1 diamant"); }
        else if is_boss { gold.0 += 25; } else { gold.0 += 8; }
    }
    for (e, sent_cost) in out {
        commands.entity(e).despawn_recursive();
        match sent_cost {
            // tvoje odeslana jednotka projde: +20 % ceny zpet jako gold a +1 zivot
            Some(cost) => {
                gold.0 += (cost as f32 * LEAK_GOLD_RATIO).round() as i32;
                escaped.0 = escaped.0.saturating_sub(LEAK_LIFE_GAIN);
                println!("Tvoje jednotka prosla! +{}G, zivot +{}", (cost as f32 * LEAK_GOLD_RATIO).round() as i32, LEAK_LIFE_GAIN);
            }
            // bezny creep projde: ubyde zivot
            None => { escaped.0 += 1; println!("PROSLO! {}/15", escaped.0); }
        }
    }
}

// Startup: sdilene meshe/materialy creepu
pub fn setup_creep_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(CreepAssets {
        unit_circle: meshes.add(Annulus::new(0.88, 1.0)), // tenky kruh (obrys), velikost se dela pres scale
        slow_mat: materials.add(ColorMaterial::from_color(Color::srgba(0.2, 0.6, 1.0, 0.85))),
        shadow_mesh: meshes.add(Circle::new(1.0)), // vyplneny kruh, z nej se scale dela elipsa
        shadow_mat: materials.add(ColorMaterial::from_color(Color::srgba(0.0, 0.0, 0.0, 0.4))),
        main_meshes: std::collections::HashMap::new(),
    });
}

// === ODESLANE JEDNOTKY (panel S) ===

// klepnuti na jednotku v panelu S: odesle se az PO PUSTENI prstu (a jen kdyz se prst netahl = neposouval panel); zaplatit, +income, vyslat
const TAP_MOVE_MAX: f32 = 8.0; // vic pixelu pohybu prstu nez tohle = tah (posouvani), ne klepnuti
pub fn send_unit_system(
    mut commands: Commands,
    q_items: Query<(&Interaction, &SendPanelItem), Changed<Interaction>>,
    touches: Res<Touches>,
    mut pressed: Local<Option<u8>>, // ktera jednotka je prave stisknuta
    mut moved: Local<f32>,          // kolik se prst pohnul od stisku
    mut gold: ResMut<Gold>,
    mut income: ResMut<Income>,
    assets: Res<CreepAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for t in touches.iter() { *moved += t.delta().length(); }
    for (interaction, item) in q_items.iter() {
        let mut kind: Option<u8> = None;
        if *interaction == Interaction::Pressed {
            *pressed = Some(item.0);
            *moved = 0.0;
        } else if *pressed == Some(item.0) {
            // pusteni: odeslat jen pri klepnuti, ne pri tahu
            *pressed = None;
            if *moved <= TAP_MOVE_MAX { kind = Some(item.0); }
        }
        let Some(kind) = kind else { continue; };
        let def = get_unit_def(kind);
        if gold.0 < def.cost { println!("Malo gold na {}: potreba {} mas {}", def.name, def.cost, gold.0); continue; }
        gold.0 -= def.cost;
        let idx = (kind as usize).min(31); // pocitadlo odeslani zvlast pro kazdou jednotku
        income.amount += def.income as f32 * income_multiplier(income.sent[idx]); // scaling podle poctu odeslani TETO jednotky (wave_def::INCOME_TIERS)
        income.sent[idx] += 1;
        let x = LANE_X + rand::thread_rng().gen_range(-LANE_W*0.35..LANE_W*0.35);
        let y = LANE_Y + LANE_H/2.0 + SPAWN_ABOVE;
        let radius = def.radius;
        let label = format!("{:.0}", def.hp);

        let air = is_air(kind);
        let mut ec = commands.spawn((
            Transform::from_xyz(x, y, if air { 4.6 } else { 4.0 }), // letajici nad zemnimi
            Creep { is_boss: false, is_mega: false, base_radius: radius, speed_mul: def.speed_mul },
            CreepMainCircle,
            Health(def.hp),
            MaxHealth(def.hp),
            SentUnit { cost: def.cost },
        ));
        if air { ec.insert(Air); }
        if let Some(en) = enrage_for(kind) { ec.insert(Enrage { hp_frac: en.hp_frac, speed_bonus: en.speed_bonus, size_bonus: en.size_bonus, flash: en.flash }); } // jen jednotky s enrage v tabulce
        if let Some(r) = resist_for(kind) { ec.insert(Resist { stun_immune: r.stun_immune, push: r.push, slow: r.slow, dmg_mul: r.dmg_mul }); } // jen jednotky s odolnosti v tabulce
        if let Some(h) = heal_for(kind) { ec.insert(Heal { radius: h.radius, hps: h.hps, tick: Timer::from_seconds(HEAL_TICK_SECS, TimerMode::Repeating) }); } // jen lecitel
        if let Some(a) = aura_for(kind) { ec.insert(Aura { radius: a.radius, hp_bonus: a.hp_bonus }); } // jen Captain
        if let Some(s) = shoot_back_for(kind) { ec.insert(ShootBack { range: s.range, slow: s.slow, secs: s.secs, tick: Timer::from_seconds(s.interval, TimerMode::Repeating) }); } // jen Frost Archer
        if let Some(a) = tower_aura_for(kind) { ec.insert(TowerAura { radius: a.radius, random_target: a.random_target, dmg_mul: a.dmg_mul, range_mul: a.range_mul, rate_mul: a.rate_mul, stomp_every: a.stomp_every, stomp_stun: a.stomp_stun, timer: Timer::from_seconds(a.stomp_every.max(0.1), TimerMode::Repeating) }); } // jen creepi s aurou na veze
        match def.image {
            // jednotky s obrazkem
            Some(path) => { ec.insert(Sprite { image: asset_server.load(path), custom_size: Some(Vec2::splat(radius * 2.0)), ..default() }); }
            // ostatni jen kolecko
            None => { ec.insert((Mesh2d(meshes.add(Circle::new(radius))), MeshMaterial2d(materials.add(ColorMaterial::from_color(def.color))))); }
        }
        let unit_mesh = assets.unit_circle.clone();
        let slow_mat = assets.slow_mat.clone();
        let shadow = if air { Some((assets.shadow_mesh.clone(), assets.shadow_mat.clone())) } else { None }; // stin jen pro letajici
        // fialovy shimmer okolo tela (jen Sorceress): data z wave_def::shimmer_for
        let shimmer = shimmer_for(kind).map(|s| (assets.shadow_mesh.clone(), materials.add(ColorMaterial::from_color(s.color)), s.period));
        // tenky zeleny kruh o polomeru lecebneho radiu (jen lecitel)
        let heal_ring = heal_for(kind).map(|h| (meshes.add(Annulus::new(0.99, 1.0)), materials.add(ColorMaterial::from_color(Color::srgba(0.3, 1.0, 0.4, 0.4))), h.radius));
        // dvojity kruh aury Captaina: modry v radiu aury, zlaty o kousek dal (jen Captain)
        let aura_ring = aura_for(kind).map(|a| (meshes.add(Annulus::new(0.99, 1.0)), materials.add(ColorMaterial::from_color(Color::srgba(0.3, 0.5, 0.95, 0.45))), materials.add(ColorMaterial::from_color(Color::srgba(0.9, 0.8, 0.3, 0.35))), a.radius));
        ec.with_children(|parent| {
            // shimmer: vyplneny kruh za telem, pulzuje ho shimmer_system (scale)
            if let Some((mesh, mat, period)) = shimmer {
                parent.spawn((
                    Mesh2d(mesh),
                    MeshMaterial2d(mat),
                    Transform::from_xyz(0.0, 0.0, -0.15).with_scale(Vec3::new(radius * 1.15, radius * 1.15, 1.0)),
                    Shimmer { base_r: radius, period },
                ));
            }
            // dvojity kruh aury kolem Captaina (child, jede s nim; scale = polomer)
            if let Some((mesh, mat_in, mat_out, r)) = aura_ring {
                parent.spawn((Mesh2d(mesh.clone()), MeshMaterial2d(mat_in), Transform::from_xyz(0.0, 0.0, -0.3).with_scale(Vec3::new(r, r, 1.0))));
                parent.spawn((Mesh2d(mesh), MeshMaterial2d(mat_out), Transform::from_xyz(0.0, 0.0, -0.31).with_scale(Vec3::new(r + 6.0, r + 6.0, 1.0))));
            }
            // kruh lecebneho radiu kolem Priesta (child, jede s nim; scale = polomer)
            if let Some((mesh, mat, r)) = heal_ring {
                parent.spawn((
                    Mesh2d(mesh),
                    MeshMaterial2d(mat),
                    Transform::from_xyz(0.0, 0.0, -0.3).with_scale(Vec3::new(r, r, 1.0)),
                ));
            }
            // maly stin pod letajici jednotkou (z -0.2 = pod telem, ale nad zemnimi jednotkami)
            if let Some((mesh, mat)) = shadow {
                parent.spawn((
                    Mesh2d(mesh),
                    MeshMaterial2d(mat),
                    Transform::from_xyz(0.0, -(radius + 2.0), -0.2).with_scale(Vec3::new(radius * 0.8, radius * 0.3, 1.0)),
                ));
            }
            // HP bar + text + outline: stejne jako u beznych creepu (na ty se vazou update_hp_* a update_outline)
            parent.spawn((
                Sprite { color: Color::srgb(0.1, 0.1, 0.1), custom_size: Some(Vec2::new(34.0, 6.0)), ..default() },
                Transform::from_xyz(0.0, radius + 14.0, 0.2),
                HpBarBg,
            ));
            parent.spawn((
                Sprite { color: Color::srgb(0.2, 0.9, 0.2), custom_size: Some(Vec2::new(32.0, 4.0)), ..default() },
                Transform::from_xyz(0.0, radius + 14.0, 0.3),
                HpBarFill,
            ));
            parent.spawn((
                Text2d::new(label),
                TextFont { font_size: 10.0, ..default() },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, radius + 26.0, 0.4),
                HpBarText,
                CreepHpText,
            ));
            parent.spawn((Text2d::new(""), TextFont { font_size: 9.0, ..default() }, TextColor(Color::srgb(0.4, 0.8, 1.0)), Transform::from_xyz(0.0, radius + 36.0, 0.4), CreepStatusText));
            let outline_radius = radius + FROST_OUTLINE_MAX_EXTRA;
            parent.spawn((
                Mesh2d(unit_mesh),
                MeshMaterial2d(slow_mat),
                Transform::from_xyz(0.0, 0.0, -0.1).with_scale(Vec3::new(outline_radius, outline_radius, 1.0)),
                SlowOutline,
                Visibility::Hidden,
            ));
        });
        println!("Odeslana jednotka {} za {}G, income {:.1} / 7s", def.name, def.cost, income.amount);
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
    mut priests: Query<(Entity, &Transform, &mut Heal)>,
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
    captains: Query<(Entity, &Transform, &Aura)>,
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
                if let Some(mut ec) = commands.get_entity(e) { ec.insert(AuraBuff { delta }); }
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
            ec.insert(TowerSlow { timer: Timer::from_seconds(sb.secs, TimerMode::Once), factor: 1.0 - sb.slow });
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

// === HLIDAC (Guardian) ===
const GUARDIAN_LEAD_SECS: f32 = 2.0;          // kolik sekund pred startem vlny vybehne
const GUARDIAN_SPEED_MUL: f32 = 1.5;          // rychlost = 150 % beznych creepu
const GUARDIAN_RADIUS: f32 = 14.0;
const GUARDIAN_ATTACK_DIST: f32 = CELL * 0.9; // z jake vzdalenosti od veze ji znici

// vybehne 2 s pred kazdou vlnou; sloupec: prednostne horni bunka bez cesty (kapsa), jinak nahodne jako creepi
pub fn guardian_spawn_system(
    mut commands: Commands,
    timer: Res<CreepTimer>,
    wave: Res<WaveCount>,
    paused: Res<WavePaused>,
    grid: Res<Grid>,
    path: Res<PathMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut last_wave: Local<u32>, // pro kterou vlnu uz hlidac vybehl
) {
    let next = wave.0 + 1;
    if paused.0 || next > MAX_WAVES || *last_wave == next { return; }
    if timer.0.remaining_secs() > GUARDIAN_LEAD_SECS { return; }
    *last_wave = next;
    let mut rng = rand::thread_rng();
    // sloupce, odkud hlidac vybehne: otevrene vstupy (horni bunka s cestou); uplny block = kterakoli volna horni bunka (pak boří veze)
    let open: Vec<usize> = (0..COLS).filter(|&c| !grid.blocked[ROWS - 1][c] && path.dist[ROWS - 1][c].is_some()).collect();
    let pockets: Vec<usize> = if open.is_empty() { (0..COLS).filter(|&c| !grid.blocked[ROWS - 1][c]).collect() } else { open };
    let x = if pockets.is_empty() {
        LANE_X + rng.gen_range(-LANE_W * 0.35..LANE_W * 0.35)
    } else {
        grid_to_world(ROWS - 1, pockets[rng.gen_range(0..pockets.len())]).x
    };
    let y = LANE_Y + LANE_H / 2.0 + SPAWN_ABOVE;
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(GUARDIAN_RADIUS))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(1.0, 0.85, 0.2)))),
        Transform::from_xyz(x, y, 4.2),
        Guardian,
    )).with_children(|p| {
        p.spawn((Text2d::new("HLIDAC"), TextFont { font_size: 9.0, ..default() }, TextColor(Color::WHITE), Transform::from_xyz(0.0, GUARDIAN_RADIUS + 10.0, 0.4)));
    });
    println!("Hlidac vybehl pred vlnou {}", next);
}

// zbourat vez na policku jednou ranou (bez refundu): despawn, uvolnit bunku, prepocitat cestu, zrusit vyber
fn guardian_destroy(commands: &mut Commands, grid: &mut Grid, path: &mut PathMap, selected: &mut SelectedCell, towers: &Query<(Entity, &Tower)>, row: usize, col: usize) {
    if let Some((t_ent, _)) = towers.iter().find(|(_, t)| t.row == row && t.col == col) {
        commands.entity(t_ent).despawn_recursive();
    }
    grid.blocked[row][col] = false;
    path.dist = compute_path(&grid.blocked);
    if selected.0 == Some((row, col)) { selected.0 = None; }
    println!("Hlidac znicil vez na [{},{}]", col, row);
}

// chuze hlidace: jako creep po BFS vzdalenostech; kdyz nema dalsi bod (kapsa / uplny block), zni vez pod sebou a jde dal; v cili zmizi
pub fn guardian_system(
    mut commands: Commands,
    time: Res<Time>,
    paused: Res<WavePaused>,
    mut grid: ResMut<Grid>,
    mut path: ResMut<PathMap>,
    mut selected: ResMut<SelectedCell>,
    towers: Query<(Entity, &Tower)>,
    mut guards: Query<(Entity, &mut Transform), With<Guardian>>,
) {
    if paused.0 { return; }
    let step = CREEP_BASE_SPEED * GUARDIAN_SPEED_MUL * time.delta_secs();
    for (g, mut tf) in guards.iter_mut() {
        let pos = tf.translation.truncate();
        if pos.y < LANE_Y - LANE_H / 2.0 - 10.0 { commands.entity(g).despawn_recursive(); continue; } // dosel do cile
        let Some((row, col)) = world_to_grid(pos.x, pos.y) else { tf.translation.y -= step; continue; }; // nad lajnou: rovne dolu
        if row == 0 { tf.translation.y -= step; continue; } // posledni rada: rovne ven pod lajnu
        // vstoupil shora do bunky s vezi (zablokovana horni rada): znicit ji
        if grid.blocked[row][col] {
            guardian_destroy(&mut commands, &mut grid, &mut path, &mut selected, &towers, row, col);
            continue;
        }
        let cur = path.dist[row][col];
        // soused blize k cili (z bunky bez cesty libovolny soused s cestou)
        let mut best: Option<(usize, usize, i32)> = None;
        for (dr, dc) in [(-1isize, 0isize), (0, 1), (0, -1), (1, 0)] {
            let (nr, nc) = (row as isize + dr, col as isize + dc);
            if nr < 0 || nr >= ROWS as isize || nc < 0 || nc >= COLS as isize { continue; }
            let (nr, nc) = (nr as usize, nc as usize);
            if grid.blocked[nr][nc] { continue; }
            let Some(d) = path.dist[nr][nc] else { continue; };
            if cur.map_or(false, |c| d >= c) { continue; }
            if best.map_or(true, |b| d < b.2) { best = Some((nr, nc, d)); }
        }
        if let Some((nr, nc, _)) = best {
            let dir = (grid_to_world(nr, nc) - pos).normalize_or_zero();
            tf.translation.x += dir.x * step;
            tf.translation.y += dir.y * step;
            continue;
        }
        // zadny dostupny dalsi bod: dole stoji vez = znicit ji jednou ranou (bez refundu), jinak jit rovne dolu
        if grid.blocked[row - 1][col] && pos.distance(grid_to_world(row - 1, col)) <= GUARDIAN_ATTACK_DIST {
            guardian_destroy(&mut commands, &mut grid, &mut path, &mut selected, &towers, row - 1, col);
            continue;
        }
        tf.translation.y -= step;
    }
}

// === HLIDAC STRILI NA BLOK ===
const SNIPE_STALL_SECS: f32 = 1.0; // po jak dlouhe dobe, kdy creepy nemuzou projit (nehybou se), hlidac vystreli
const SNIPE_MSG_SECS: f32 = 4.0;   // jak dlouho je videt hlaska

// nejlevnejsi cesta z horni rady na spodek, kde pruchod vezi stoji 1 (volna bunka 0); vrati nejvyssi vez na te ceste (tu, co creepum brani nejdriv)
fn find_blocker(blocked: &[[bool; COLS]; ROWS], near: Vec2) -> Option<(usize, usize)> {
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
    // z nejlevnejsich vstupu vezme ten nejblize stredu creepu, na jeho ceste pak vez nejblize creepum (cena ani level veze nehraje roli)
    let min_cost = (0..COLS).map(|c| cost[ROWS - 1][c]).min()?;
    let best_c = (0..COLS)
        .filter(|&c| cost[ROWS - 1][c] == min_cost)
        .min_by(|&a, &b| (grid_to_world(ROWS - 1, a).x - near.x).abs().total_cmp(&(grid_to_world(ROWS - 1, b).x - near.x).abs()))?;
    let mut cur = Some((ROWS - 1, best_c));
    let mut pick: Option<((usize, usize), f32)> = None; // (policko veze, vzdalenost od creepu)
    while let Some((r, c)) = cur {
        if blocked[r][c] {
            let d = grid_to_world(r, c).distance(near);
            if pick.map_or(true, |p| d < p.1) { pick = Some(((r, c), d)); }
        }
        cur = parent[r][c];
    }
    pick.map(|p| p.0)
}

// kdyz je lajna zablokovana a zadny nelitajici creep se 1 s nehybe: hlidac vystreli na vez v ceste (instantne) a ukaze hlasku
pub fn guardian_snipe_system(
    mut commands: Commands,
    time: Res<Time>,
    paused: Res<WavePaused>,
    mut grid: ResMut<Grid>,
    mut path: ResMut<PathMap>,
    mut selected: ResMut<SelectedCell>,
    mut bolt_pool: ResMut<crate::systems::effects::BoltPool>,
    mut stall: Local<f32>,                                   // jak dlouho uz creepy stoji
    mut prev: Local<std::collections::HashMap<Entity, Vec2>>, // pozice creepu v minulem snimku
    towers: Query<(Entity, &Tower)>,
    creeps: Query<(Entity, &Transform), (With<Creep>, Without<Air>)>,
    mut msg: Query<(&mut GuardianMsg, &mut Visibility)>,
) {
    // hlaska po case zmizi
    for (mut m, mut vis) in msg.iter_mut() {
        if *vis == Visibility::Visible {
            m.timer.tick(time.delta());
            if m.timer.finished() { *vis = Visibility::Hidden; }
        }
    }
    if paused.0 { return; }
    // hybe se nejaky creep? (novy creep = hybe se)
    let mut any = false;
    let mut sum = Vec2::ZERO; // soucet pozic creepu (pro stred davky)
    let mut count = 0.0_f32;
    let mut moving = false;
    for (e, tf) in creeps.iter() {
        any = true;
        sum += tf.translation.truncate();
        count += 1.0;
        let p = tf.translation.truncate();
        match prev.insert(e, p) {
            Some(old) => { if old.distance_squared(p) > 0.01 { moving = true; } }
            None => { moving = true; }
        }
    }
    if prev.len() > 256 { prev.retain(|e, _| creeps.contains(*e)); } // uklid po smazanych creepech
    // lajna je zablokovana, kdyz zadna horni bunka nema cestu
    let blocked_lane = !(0..COLS).any(|c| !grid.blocked[ROWS - 1][c] && path.dist[ROWS - 1][c].is_some());
    if !any || moving || !blocked_lane { *stall = 0.0; return; }
    *stall += time.delta_secs();
    if *stall < SNIPE_STALL_SECS { return; }
    *stall = 0.0;
    let near = sum / count.max(1.0); // stred creepu: strili se na vez nejblize jim
    let Some((row, col)) = find_blocker(&grid.blocked, near) else { return; };
    // vystrel: svetla car z pasu nad lajnou na vez, vez zmizi hned
    let tower_pos = grid_to_world(row, col);
    crate::systems::effects::spawn_bolt(&mut commands, &mut bolt_pool, Vec2::new(tower_pos.x, LANE_Y + LANE_H / 2.0 + 60.0), tower_pos);
    guardian_destroy(&mut commands, &mut grid, &mut path, &mut selected, &towers, row, col);
    for (mut m, mut vis) in msg.iter_mut() {
        m.timer = Timer::from_seconds(SNIPE_MSG_SECS, TimerMode::Once);
        *vis = Visibility::Visible;
    }
}

// === AURY CREEPU NA VEZE (Infernal, Pit Lord, Phoenix, Knight, Final Boss) ===
// kazdy snimek: kazda vez spocita upravy od vsech aur v dosahu (TowerMods) a stomp omraceni; tower_shoot je pouzije
pub fn tower_aura_system(
    mut commands: Commands,
    time: Res<Time>,
    paused: Res<WavePaused>,
    mut stomps: Local<Vec<(Vec2, f32, f32)>>, // (stred, radius, sekundy omraceni) pro tento snimek, znovupouzity buffer
    mut auras: Query<(&Transform, &mut TowerAura)>,
    mut towers: Query<(Entity, &Transform, Option<&mut TowerMods>), With<Tower>>,
) {
    if paused.0 { return; }
    let dt = time.delta_secs();
    stomps.clear();
    for (tf, mut a) in auras.iter_mut() {
        if a.stomp_every > 0.0 {
            a.timer.tick(time.delta());
            if a.timer.just_finished() { stomps.push((tf.translation.truncate(), a.radius, a.stomp_stun)); }
        }
    }
    for (t_ent, t_tf, mods) in towers.iter_mut() {
        let Some(mut mods) = mods else {
            if let Some(mut ec) = commands.get_entity(t_ent) { ec.insert(TowerMods::default()); } // vez dostane komponentu, pocita se od pristiho snimku
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
        mods.dmg_mul = dmg;
        mods.range_mul = range;
        mods.rate_mul = rate;
        mods.random_target = random;
        mods.stun_left = (mods.stun_left - dt).max(0.0);
        for &(c, r, s) in stomps.iter() {
            if c.distance(t_pos) <= r { mods.stun_left = mods.stun_left.max(s); }
        }
    }
}

// CHEAT (jen pro testovani): stisknuti pricte +50 do income (gold za tik 7 s)
const CHEAT_INCOME: f32 = 50.0;
pub fn cheat_button_system(mut income: ResMut<Income>, q: Query<&Interaction, (Changed<Interaction>, With<CheatButton>)>) {
    for i in q.iter() {
        if *i == Interaction::Pressed { income.amount += CHEAT_INCOME; }
    }
}

// income: kazdych 7s pripise gold, text nahore
pub fn income_system(
    time: Res<Time>,
    paused: Res<WavePaused>,
    mut gold: ResMut<Gold>,
    mut income: ResMut<Income>,
    mut q_text: Query<&mut Text, With<IncomeText>>,
) {
    if !paused.0 && income.timer.tick(time.delta()).just_finished() {
        let total = income.amount + income.carry; // desetinny zbytek se necha na pristi tik
        let whole = total.floor();
        income.carry = total - whole;
        gold.0 += whole as i32;
    }
    let s = format!("Income in: {:.0}s (+{:.1})", income.timer.remaining_secs().ceil(), income.amount); // za kolik s prijde, v zavorce celkovy income
    for mut t in q_text.iter_mut() { if t.0 != s { t.0 = s.clone(); } }
}

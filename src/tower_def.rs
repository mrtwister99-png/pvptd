use bevy::prelude::*;

#[derive(Clone)]
#[allow(dead_code)] // name a gold zustavaji v tabulce pro prehled (gold uz se nepouziva, stavba stoji 10/20/30...)
pub struct TowerDef {
    pub name: &'static str,
    pub gold: i32,
    pub dmg_min: f32,
    pub dmg_max: f32,
    pub range: f32,
    pub interval: f32,
    pub splash: f32,
    pub slow_per: f32,
    pub slow_max: f32,
    pub push_chance: f32,
    pub push_dist: f32,
    pub dot_dps: f32,
    pub dot_dur: f32,
    pub chain: u32,
    pub color: Color,
}

// dosah vsech veze se nasobi TOWER_RANGE_SCALE (jedno misto k ladeni v constants.rs)
// level = rank veze v zapase (1..MAX_TOWER_LEVEL). Ranky 1-5 jsou z tabulky,
// 6 a 7 vychazeji z ranku 5 a poskozeni roste dal (RANK_EXTRA_DMG_MUL za kazdy rank navic).
pub fn get_tower_def(kind: u8, level: u32) -> TowerDef {
    let level = level.clamp(1, crate::constants::MAX_TOWER_LEVEL);
    let mut def = base_tower_def(kind, level.min(5));
    if level > 5 {
        let m = crate::constants::RANK_EXTRA_DMG_MUL.powi((level - 5) as i32);
        def.dmg_min *= m;
        def.dmg_max *= m;
        def.dot_dps *= m;
        def.range *= 1.0 + 0.05 * (level - 5) as f32;
    }
    def.range *= crate::constants::TOWER_RANGE_SCALE;
    def
}

fn base_tower_def(kind: u8, level: u32) -> TowerDef {
    let lvl = level.clamp(1,5) as usize -1;
    match kind {
        1 => {
            let defs = [
                TowerDef { name:"Basic Tower", gold:10, dmg_min:2.0, dmg_max:4.0, range:400.0, interval:1.0, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.9,0.2,0.2) },
                TowerDef { name:"Piercing Arrow", gold:40, dmg_min:5.0, dmg_max:8.0, range:500.0, interval:0.85, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.9,0.2,0.2) },
                TowerDef { name:"Longbow Sentry", gold:180, dmg_min:16.0, dmg_max:19.0, range:700.0, interval:1.2, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.9,0.2,0.2) },
                TowerDef { name:"Deadeye Marksman", gold:450, dmg_min:39.0, dmg_max:41.0, range:900.0, interval:0.6, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.9,0.2,0.2) },
                TowerDef { name:"Phantom Archer", gold:1200, dmg_min:110.0, dmg_max:125.0, range:1000.0, interval:0.9, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.9,0.2,0.2) },
            ];
            defs[lvl].clone()
        },
        2 => {
            let defs = [
                TowerDef { name:"Cannon Tower", gold:40, dmg_min:10.0, dmg_max:14.0, range:300.0, interval:2.0, splash:150.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.9,0.85,0.15) },
                TowerDef { name:"Mortar Pit", gold:220, dmg_min:28.0, dmg_max:29.0, range:350.0, interval:2.5, splash:200.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.9,0.85,0.15) },
                TowerDef { name:"Siege Cannon", gold:650, dmg_min:88.0, dmg_max:166.0, range:400.0, interval:2.5, splash:300.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.9,0.85,0.15) },
                TowerDef { name:"Howitzer Bastion", gold:1400, dmg_min:280.0, dmg_max:320.0, range:450.0, interval:2.5, splash:350.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.9,0.85,0.15) },
                TowerDef { name:"Apocalypse Cannon", gold:2600, dmg_min:637.0, dmg_max:640.0, range:450.0, interval:2.5, splash:350.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.9,0.85,0.15) },
            ];
            defs[lvl].clone()
        },
        3 => {
            let defs = [
                TowerDef { name:"Frost Obelisk", gold:75, dmg_min:3.0, dmg_max:4.0, range:500.0, interval:1.5, splash:50.0, slow_per:0.075, slow_max:0.3, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.2,0.6,1.0) },
                TowerDef { name:"Runic Frost", gold:280, dmg_min:7.0, dmg_max:8.0, range:600.0, interval:1.5, splash:50.0, slow_per:0.1, slow_max:0.4, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.2,0.6,1.0) },
                TowerDef { name:"Glacial Watcher", gold:600, dmg_min:12.0, dmg_max:13.0, range:600.0, interval:1.25, splash:75.0, slow_per:0.15, slow_max:0.45, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.2,0.6,1.0) },
                TowerDef { name:"Frost Citadel", gold:1300, dmg_min:28.0, dmg_max:29.0, range:600.0, interval:1.0, splash:100.0, slow_per:0.2, slow_max:0.5, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.2,0.6,1.0) },
                TowerDef { name:"Absolute Zero", gold:2400, dmg_min:60.0, dmg_max:67.0, range:650.0, interval:1.0, splash:125.0, slow_per:0.25, slow_max:0.6, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.2,0.6,1.0) },
            ];
            defs[lvl].clone()
        },
        4 => {
            let defs = [
                TowerDef { name:"Fire Pit", gold:150, dmg_min:4.0, dmg_max:5.0, range:400.0, interval:2.5, splash:150.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:1.5, dot_dur:7.0, chain:0, color:Color::srgb(1.0,0.3,0.1) },
                TowerDef { name:"Ember Spire", gold:400, dmg_min:11.0, dmg_max:13.0, range:400.0, interval:2.0, splash:150.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:3.5, dot_dur:7.0, chain:0, color:Color::srgb(1.0,0.3,0.1) },
                TowerDef { name:"Meteor Tower", gold:800, dmg_min:22.0, dmg_max:23.0, range:800.0, interval:2.5, splash:300.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:11.0, dot_dur:2.0, chain:0, color:Color::srgb(1.0,0.3,0.1) },
                TowerDef { name:"Inferno Core", gold:1500, dmg_min:47.0, dmg_max:55.0, range:400.0, interval:0.5, splash:50.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:6.0, dot_dur:7.0, chain:0, color:Color::srgb(1.0,0.3,0.1) },
                TowerDef { name:"Hellforge", gold:2500, dmg_min:197.0, dmg_max:207.0, range:400.0, interval:1.0, splash:150.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:10.0, dot_dur:7.0, chain:0, color:Color::srgb(1.0,0.3,0.1) },
            ];
            defs[lvl].clone()
        },
        5 => {
            let defs = [
                TowerDef { name:"Venom Sprout", gold:80, dmg_min:2.0, dmg_max:3.0, range:400.0, interval:1.2, splash:0.0, slow_per:0.1, slow_max:0.1, push_chance:0.0, push_dist:0.0, dot_dps:1.0, dot_dur:10.0, chain:0, color:Color::srgb(0.2,0.7,0.2) },
                TowerDef { name:"Toxic Bloom", gold:300, dmg_min:5.0, dmg_max:7.0, range:450.0, interval:1.1, splash:0.0, slow_per:0.15, slow_max:0.15, push_chance:0.0, push_dist:0.0, dot_dps:2.0, dot_dur:12.0, chain:0, color:Color::srgb(0.2,0.7,0.2) },
                TowerDef { name:"Plague Vine", gold:650, dmg_min:10.0, dmg_max:12.0, range:500.0, interval:1.0, splash:0.0, slow_per:0.18, slow_max:0.18, push_chance:0.0, push_dist:0.0, dot_dps:3.5, dot_dur:14.0, chain:0, color:Color::srgb(0.2,0.7,0.2) },
                TowerDef { name:"Deathroot", gold:1350, dmg_min:20.0, dmg_max:25.0, range:550.0, interval:0.9, splash:50.0, slow_per:0.2, slow_max:0.2, push_chance:0.0, push_dist:0.0, dot_dps:5.0, dot_dur:16.0, chain:0, color:Color::srgb(0.2,0.7,0.2) },
                TowerDef { name:"Worldbane Blight", gold:2600, dmg_min:40.0, dmg_max:50.0, range:600.0, interval:0.8, splash:100.0, slow_per:0.25, slow_max:0.25, push_chance:0.0, push_dist:0.0, dot_dps:7.0, dot_dur:20.0, chain:0, color:Color::srgb(0.2,0.7,0.2) },
            ];
            defs[lvl].clone()
        },
        6 => {
            let defs = [
                TowerDef { name:"Scout Post", gold:30, dmg_min:4.0, dmg_max:6.0, range:600.0, interval:1.5, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.1,0.15,0.55) },
                TowerDef { name:"Sniper Nest", gold:150, dmg_min:12.0, dmg_max:18.0, range:800.0, interval:1.8, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.1,0.15,0.55) },
                TowerDef { name:"Eagle Eye", gold:400, dmg_min:35.0, dmg_max:45.0, range:950.0, interval:1.6, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.1,0.15,0.55) },
                TowerDef { name:"Ghost Sniper", gold:900, dmg_min:85.0, dmg_max:105.0, range:1100.0, interval:1.4, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.1,0.15,0.55) },
                TowerDef { name:"Void Sniper", gold:2000, dmg_min:220.0, dmg_max:260.0, range:1200.0, interval:1.2, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.1,0.15,0.55) },
            ];
            defs[lvl].clone()
        },
        7 => {
            let defs = [
                TowerDef { name:"Shock Turret", gold:60, dmg_min:2.5, dmg_max:2.8, range:350.0, interval:1.0, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:2, color:Color::srgb(0.95,0.85,0.2) },
                TowerDef { name:"Static Coil", gold:220, dmg_min:5.7, dmg_max:6.3, range:380.0, interval:0.9, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:2, color:Color::srgb(0.95,0.85,0.2) },
                TowerDef { name:"Storm Spire", gold:550, dmg_min:13.5, dmg_max:14.5, range:420.0, interval:0.8, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.3, push_dist:120.0, dot_dps:0.0, dot_dur:0.0, chain:2, color:Color::srgb(0.95,0.85,0.2) },
                TowerDef { name:"Thunder Bastion", gold:1300, dmg_min:30.0, dmg_max:31.6, range:450.0, interval:0.7, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.4, push_dist:160.0, dot_dps:0.0, dot_dur:0.0, chain:3, color:Color::srgb(0.95,0.85,0.2) },
                TowerDef { name:"God of Thunder", gold:2700, dmg_min:72.0, dmg_max:73.8, range:500.0, interval:0.6, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.5, push_dist:200.0, dot_dps:0.0, dot_dur:0.0, chain:4, color:Color::srgb(0.95,0.85,0.2) },
            ];
            defs[lvl].clone()
        },
        8..=18 => special_def(kind, lvl),
        _ => TowerDef { name:"Unknown", gold:999, dmg_min:1.0, dmg_max:1.0, range:100.0, interval:1.0, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.5,0.5,0.5) },
    }
}



// specialni veze 8-18: 5 radku = rank 1..5 (6 a 7 dopocita get_tower_def).
// (dmg_min, dmg_max, dosah pred TOWER_RANGE_SCALE, interval, splash, push_dist)
fn special_def(kind: u8, lvl: usize) -> TowerDef {
    let (name, color, rows): (&'static str, Color, [(f32, f32, f32, f32, f32, f32); 5]) = match kind {
        // HARPOON: jen na letajici, silna rana
        8 => ("Harpoon", Color::srgb(0.2, 0.75, 0.75), [(14.0, 16.0, 500.0, 1.5, 0.0, 0.0), (36.0, 40.0, 550.0, 1.4, 0.0, 0.0), (90.0, 100.0, 600.0, 1.3, 0.0, 0.0), (220.0, 240.0, 650.0, 1.2, 0.0, 0.0), (540.0, 580.0, 700.0, 1.1, 0.0, 0.0)]),
        // GEYSER: dosah = okruh pulzu, interval = jak casto, push = jak daleko odhodi
        9 => ("Geyser", Color::srgb(0.55, 0.8, 1.0), [(3.0, 3.0, 240.0, 6.0, 0.0, 60.0), (8.0, 8.0, 270.0, 5.5, 0.0, 80.0), (20.0, 20.0, 300.0, 5.0, 0.0, 100.0), (50.0, 50.0, 330.0, 4.5, 0.0, 120.0), (120.0, 120.0, 360.0, 4.0, 0.0, 150.0)]),
        // EXECUTIONER: pomala silna rana + % chybejiciho HP
        10 => ("Executioner", Color::srgb(0.55, 0.1, 0.1), [(7.0, 9.0, 320.0, 2.0, 0.0, 0.0), (18.0, 22.0, 340.0, 1.9, 0.0, 0.0), (45.0, 55.0, 360.0, 1.8, 0.0, 0.0), (110.0, 130.0, 380.0, 1.7, 0.0, 0.0), (280.0, 320.0, 400.0, 1.6, 0.0, 0.0)]),
        // VOODOO: slaba rana, hlavne znacka
        11 => ("Voodoo", Color::srgb(0.6, 0.3, 0.6), [(2.0, 2.0, 420.0, 1.6, 0.0, 0.0), (5.0, 5.0, 450.0, 1.5, 0.0, 0.0), (12.0, 12.0, 480.0, 1.4, 0.0, 0.0), (30.0, 30.0, 510.0, 1.3, 0.0, 0.0), (70.0, 70.0, 540.0, 1.2, 0.0, 0.0)]),
        // CURSE: slaba rana, kletba
        12 => ("Curse", Color::srgb(0.45, 0.15, 0.7), [(1.0, 1.0, 450.0, 1.3, 0.0, 0.0), (3.0, 3.0, 480.0, 1.2, 0.0, 0.0), (7.0, 7.0, 510.0, 1.1, 0.0, 0.0), (15.0, 15.0, 540.0, 1.0, 0.0, 0.0), (35.0, 35.0, 570.0, 0.9, 0.0, 0.0)]),
        // ILLUSION: slaba rana, zmateni
        13 => ("Illusion", Color::srgb(0.95, 0.6, 0.85), [(2.0, 2.0, 400.0, 2.5, 0.0, 0.0), (5.0, 5.0, 420.0, 2.3, 0.0, 0.0), (12.0, 12.0, 440.0, 2.1, 0.0, 0.0), (30.0, 30.0, 460.0, 1.9, 0.0, 0.0), (70.0, 70.0, 480.0, 1.7, 0.0, 0.0)]),
        // TWIN: slabsi strilna (vyhoda je pri spojovani)
        14 => ("Twin", Color::srgb(0.85, 0.75, 0.55), [(1.5, 3.0, 400.0, 1.0, 0.0, 0.0), (4.0, 6.0, 450.0, 0.9, 0.0, 0.0), (11.0, 13.0, 500.0, 0.9, 0.0, 0.0), (27.0, 29.0, 550.0, 0.8, 0.0, 0.0), (80.0, 90.0, 600.0, 0.8, 0.0, 0.0)]),
        // MINT: nestrili
        15 => ("Mint", Color::srgb(1.0, 0.82, 0.1), [(0.0, 0.0, 0.0, 99.0, 0.0, 0.0); 5]),
        // BELL: dosah = okruh, ve kterem posiluje veze
        16 => ("Bell", Color::srgb(0.9, 0.9, 0.95), [(0.0, 0.0, 220.0, 99.0, 0.0, 0.0), (0.0, 0.0, 260.0, 99.0, 0.0, 0.0), (0.0, 0.0, 300.0, 99.0, 0.0, 0.0), (0.0, 0.0, 340.0, 99.0, 0.0, 0.0), (0.0, 0.0, 380.0, 99.0, 0.0, 0.0)]),
        // SPIKES: dosah = jak blizko musi creep prijit, interval = nabijeni, splash = vybuch
        17 => ("Spikes", Color::srgb(0.6, 0.45, 0.3), [(20.0, 20.0, 170.0, 3.0, 70.0, 0.0), (50.0, 50.0, 170.0, 2.7, 75.0, 0.0), (120.0, 120.0, 180.0, 2.4, 80.0, 0.0), (300.0, 300.0, 180.0, 2.1, 85.0, 0.0), (700.0, 700.0, 190.0, 1.8, 90.0, 0.0)]),
        // BARRACKS: nestrili
        _ => ("Barracks", Color::srgb(0.45, 0.5, 0.35), [(0.0, 0.0, 0.0, 99.0, 0.0, 0.0); 5]),
    };
    let (dmin, dmax, range, interval, splash, push) = rows[lvl];
    TowerDef { name, gold: 0, dmg_min: dmin, dmg_max: dmax, range, interval, splash, slow_per: 0.0, slow_max: 0.0, push_chance: 0.0, push_dist: push, dot_dps: 0.0, dot_dur: 0.0, chain: 0, color }
}

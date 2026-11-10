use bevy::prelude::*;

#[derive(Clone)]
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
pub fn get_tower_def(kind: u8, level: u32) -> TowerDef {
    let mut def = base_tower_def(kind, level);
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
        
        _ => TowerDef { name:"Unknown", gold:999, dmg_min:1.0, dmg_max:1.0, range:100.0, interval:1.0, splash:0.0, slow_per:0.0, slow_max:0.0, push_chance:0.0, push_dist:0.0, dot_dps:0.0, dot_dur:0.0, chain:0, color:Color::srgb(0.5,0.5,0.5) },
    }
}

pub fn get_total_spent_gold(kind: u8, level: u32) -> i32 {
    let mut total = 0;
    for l in 1..=level {
        total += get_tower_def(kind, l).gold;
    }
    total
}

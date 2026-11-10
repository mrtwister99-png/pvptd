use bevy::prelude::*;

#[derive(Clone)]
#[allow(dead_code)]
pub struct WaveDef {
    pub name: &'static str,
    pub code: &'static str,
    pub hp: f32,
    pub count: usize,
    pub total_hp: f32,
    pub req_dps: f32,
    pub time_est: f32,
    pub color: Color, // zakladni barva vlny, ale creepy budou mit nahodnou variaci
    pub is_mega: bool,
    pub is_sudden: bool,
    pub boss_hp: f32,
}

pub fn get_wave_def(wave: u32) -> WaveDef {
    match wave {
        1 => WaveDef { name:"Sheep", code:"n00E", hp:20.0, count:5, total_hp:100.0, req_dps:12.0, time_est:8.3, color:Color::srgba(0.75,0.4,0.75,0.6), is_mega:false, is_sudden:false, boss_hp:0.0 },
        2 => WaveDef { name:"Frost Wolf", code:"n00C", hp:35.0, count:6, total_hp:210.0, req_dps:12.0, time_est:17.5, color:Color::srgba(0.85,0.3,0.85,0.8), is_mega:false, is_sudden:false, boss_hp:0.0 },
        3 => WaveDef { name:"Scout", code:"u003", hp:50.0, count:6, total_hp:300.0, req_dps:18.0, time_est:16.6, color:Color::srgba(0.85,0.5,0.75,0.8), is_mega:false, is_sudden:false, boss_hp:0.0 },
        4 => WaveDef { name:"Warrior", code:"u000", hp:75.0, count:7, total_hp:525.0, req_dps:28.0, time_est:13.0, color:Color::srgba(0.95,0.25,0.6,0.9), is_mega:false, is_sudden:false, boss_hp:0.0 },
        5 => WaveDef { name:"Archer", code:"n00K", hp:110.0, count:8, total_hp:880.0, req_dps:32.0, time_est:27.5, color:Color::srgba(0.95,0.5,0.2,0.95), is_mega:false, is_sudden:false, boss_hp:0.0 },
        6 => WaveDef { name:"Gnoll", code:"n00Q", hp:160.0, count:8, total_hp:1280.0, req_dps:50.0, time_est:25.6, color:Color::srgba(0.9,0.8,0.15,0.95), is_mega:false, is_sudden:false, boss_hp:0.0 },
        7 => WaveDef { name:"Bandit", code:"n00R", hp:220.0, count:9, total_hp:1980.0, req_dps:70.0, time_est:28.3, color:Color::srgb(0.9,0.6,0.1), is_mega:false, is_sudden:false, boss_hp:0.0 },
        8 => WaveDef { name:"Necromancer SUDDEN DEATH", code:"n00S", hp:300.0, count:10, total_hp:6000.0, req_dps:120.0, time_est:50.0, color:Color::srgb(0.6,0.1,0.9), is_mega:false, is_sudden:true, boss_hp:0.0 },
        9 => WaveDef { name:"Dark Mage", code:"n00T", hp:420.0, count:10, total_hp:4200.0, req_dps:180.0, time_est:23.3, color:Color::srgb(0.3,0.2,0.8), is_mega:false, is_sudden:false, boss_hp:0.0 },
        10 => WaveDef { name:"Ogre Lord MEGA", code:"n00U", hp:600.0, count:9, total_hp:5400.0, req_dps:220.0, time_est:32.7, color:Color::srgb(0.8,0.15,0.15), is_mega:true, is_sudden:false, boss_hp:1800.0 },
        11 => WaveDef { name:"Demon", code:"n00V", hp:800.0, count:11, total_hp:8800.0, req_dps:280.0, time_est:31.4, color:Color::srgb(1.0,0.2,0.2), is_mega:false, is_sudden:false, boss_hp:0.0 },
        12 => WaveDef { name:"Dragon Whelp", code:"n00W", hp:1100.0, count:12, total_hp:13200.0, req_dps:350.0, time_est:37.7, color:Color::srgb(0.2,0.8,0.3), is_mega:false, is_sudden:false, boss_hp:0.0 },
        13 => WaveDef { name:"Infernal", code:"n00X", hp:1450.0, count:12, total_hp:17400.0, req_dps:450.0, time_est:38.6, color:Color::srgb(1.0,0.4,0.0), is_mega:false, is_sudden:false, boss_hp:0.0 },
        14 => WaveDef { name:"Pit Lord", code:"n00Y", hp:1900.0, count:13, total_hp:24700.0, req_dps:550.0, time_est:44.9, color:Color::srgb(0.5,0.0,0.0), is_mega:false, is_sudden:false, boss_hp:0.0 },
        15 => WaveDef { name:"Sheep King FINAL", code:"n00Z", hp:2500.0, count:15, total_hp:37500.0, req_dps:700.0, time_est:65.0, color:Color::srgb(1.0,0.9,0.0), is_mega:true, is_sudden:false, boss_hp:8000.0 },
        _ => {
            if wave <= 30 { return unit_wave(wave); } // vlny 16-30 podle posilanych jednotek
            // vlna 31+: +12 % HP za vlnu, +1 creep za 2 vlny, max 40 kusu (mobil). Cisla se daji ladit.
            let n = wave.saturating_sub(15) as f32; // saturating: vlna 0 nepadne na podteceni
            let mul = 1.12_f32.powf(n);
            let count = (15 + (n as usize) / 2).min(40);
            let hp = 2500.0 * mul;
            WaveDef { name:"Infinite", code:"n00Z", hp, count, total_hp: hp * count as f32, req_dps: 700.0 * mul, time_est:65.0, color:Color::srgb(1.0,1.0,1.0), is_mega:true, is_sudden:false, boss_hp: 8000.0 * mul }
        }
    }
}

// VLNY 16-30 podle posilanych jednotek (varianta C): vlna N = creep N z tabulky, HP se skaluje, aby vlna 16 nebyla slabsi nez 15
const WAVE_HP_SCALE: f32 = 2.5;    // HP vlny = HP jednotky x tolik (ODHAD, laditelne)
const WAVE_COUNT_LATE: usize = 15; // kolik creepu ma kazda vlna 16-30 (ODHAD)
const WAVE_BOSS_MUL: f32 = 3.0;    // HP mega bosse na konci vln 20, 25, 30 = HP creepu x tolik (ODHAD)

fn unit_wave(wave: u32) -> WaveDef {
    let u = get_unit_def(wave as u8);
    let hp = u.hp * WAVE_HP_SCALE;
    let boss = matches!(wave, 20 | 25 | 30);
    WaveDef { name: u.name, code: "n00Z", hp, count: WAVE_COUNT_LATE, total_hp: hp * WAVE_COUNT_LATE as f32, req_dps: 700.0, time_est: 65.0, color: u.color, is_mega: boss, is_sudden: false, boss_hp: if boss { hp * WAVE_BOSS_MUL } else { 0.0 } }
}

pub fn get_creep_color_and_hp(wave: u32) -> (Color, f32) {
    let wd = get_wave_def(wave);
    (wd.color, wd.hp)
}

// === JEDNOTKY K ODESLANI (panel S) ===
pub const UNIT_COUNT: u8 = 30; // kolik jednotek je v tabulce nize (panel S se podle toho postavi)

pub struct UnitDef {
    pub name: &'static str,
    pub cost: i32,                    // cena za odeslani
    pub hp: f32,
    pub income: i32,                  // zakladni +income kazdych 7s, kdyz ji posles (viz income_multiplier)
    pub speed_mul: f32,               // nasobek rychlosti chuze (1.0 = bezna)
    pub radius: f32,
    pub color: Color,                 // barva kolecka (jednotky bez obrazku)
    pub image: Option<&'static str>,  // obrazek
}

// Zvlastni schopnosti (v komentari) zatim NEJSOU naimplementovane, jen rychlost (speed_mul).
pub fn get_unit_def(kind: u8) -> UnitDef {
    match kind {
        1 => UnitDef { name:"DAVID", cost:5, hp:10.0, income:1, speed_mul:1.0, radius:16.0, color:Color::srgb(0.9,0.9,0.9), image:Some("creeps/creep1.png") },
        2 => UnitDef { name:"DAMIAN", cost:10, hp:17.0, income:2, speed_mul:1.0, radius:16.0, color:Color::srgb(0.9,0.9,0.9), image:Some("creeps/creep2.png") },
        3 => UnitDef { name:"OLGA", cost:15, hp:25.0, income:2, speed_mul:1.0, radius:12.0, color:Color::srgb(0.95,0.5,0.7), image:Some("creeps/creep3.png") },
        4 => UnitDef { name:"KAMIL", cost:25, hp:37.0, income:3, speed_mul:1.0, radius:13.0, color:Color::srgb(0.95,0.6,0.15), image:Some("creeps/creep4.png") },
        5 => UnitDef { name:"LUKAS", cost:40, hp:55.0, income:4, speed_mul:1.0, radius:14.0, color:Color::srgb(0.2,0.85,0.85), image:Some("creeps/creep5.png") },
        // 6 Rogue: rychly 120 %, tezko trefitelny
        6 => UnitDef { name:"ROGUE", cost:60, hp:80.0, income:5, speed_mul:1.2, radius:16.0, color:Color::WHITE, image:Some("creeps/creep6.png") },
        // 7 Berserker: enrage pod 30 % HP +30 % rychlost
        7 => UnitDef { name:"BERSERK", cost:85, hp:115.0, income:6, speed_mul:1.0, radius:17.0, color:Color::WHITE, image:Some("creeps/creep7.png") },
        // 8 Frost Archer: strili zpet - slow 5 % na veze 1 s
        8 => UnitDef { name:"ARCHER", cost:110, hp:150.0, income:7, speed_mul:1.0, radius:18.0, color:Color::WHITE, image:Some("creeps/creep8.png") },
        // 9 Wyvern: letajici - trefi ho jen Air veze (Sniper, Frost, Fire, Shock), Cannon ne
        9 => UnitDef { name:"WYVERN", cost:140, hp:190.0, income:8, speed_mul:1.0, radius:18.0, color:Color::WHITE, image:Some("creeps/air9.png") },
        // 10 Ogre: tezky - 50 % resist na push, 20 % resist na slow
        10 => UnitDef { name:"OGRE", cost:180, hp:260.0, income:9, speed_mul:1.0, radius:20.0, color:Color::WHITE, image:Some("creeps/creep10.png") },
        // 11 Gryphon: letajici rychly 130 %
        11 => UnitDef { name:"GRYPHON", cost:220, hp:320.0, income:10, speed_mul:1.3, radius:15.0, color:Color::srgb(0.9,0.75,0.3), image:Some("creeps/air11.png") },
        // 12 Priest: healer - leci 2 HP/s vsechny creepy v 200 radius
        12 => UnitDef { name:"PRIEST", cost:270, hp:400.0, income:11, speed_mul:1.0, radius:15.0, color:Color::srgb(0.95,0.9,0.95), image:Some("creeps/creep12.png") },
        // 13 Captain: aura +15 % max HP vsem creepum v 250 radius
        13 => UnitDef { name:"CAPTAIN", cost:330, hp:500.0, income:12, speed_mul:1.0, radius:16.0, color:Color::srgb(0.3,0.5,0.95), image:Some("creeps/creep13.png") },
        // 14 Sorceress: magic immune 50 % - jen 50 % dmg z Fire a Frost
        14 => UnitDef { name:"SORCER.", cost:400, hp:620.0, income:13, speed_mul:1.0, radius:17.0, color:Color::srgb(0.7,0.3,0.9), image:Some("creeps/creep14.png") },
        // 15 First Boss: immune stun, 50 % push resist, enrage pod 30 % HP +50 % rychlost + cerveny pulz
        15 => UnitDef { name:"BOSS", cost:500, hp:800.0, income:15, speed_mul:1.0, radius:24.0, color:Color::srgb(0.9,0.15,0.15), image:Some("creeps/boss1.png") },
        // 16-30 (late game, HP mobile = originalu / 2): zatim JEN staty, Air a rychlost; zvlastni schopnosti se doplni po jedne
        // 16 Enchantress: mana burn 5 veze v 300 radius
        16 => UnitDef { name:"ENCHANT", cost:600, hp:1000.0, income:16, speed_mul:1.0, radius:18.0, color:Color::srgb(0.85,0.4,0.85), image:None },
        // 17 Necromancer: na smrt 2x Skeleton 80 HP
        17 => UnitDef { name:"NECRO", cost:700, hp:1250.0, income:17, speed_mul:1.0, radius:18.0, color:Color::srgb(0.4,0.7,0.4), image:None },
        // 18 Frost Wyrm (Air): frost breath aura, slow na veze v 250 radius
        18 => UnitDef { name:"WYRM", cost:800, hp:1500.0, income:18, speed_mul:1.0, radius:20.0, color:Color::srgb(0.5,0.8,1.0), image:None },
        // 19 Infernal: fire aura 5 dmg/s vezim v 200 radius, 30 % resist Fire
        19 => UnitDef { name:"INFERNAL", cost:900, hp:1800.0, income:19, speed_mul:1.0, radius:20.0, color:Color::srgb(1.0,0.4,0.1), image:None },
        // 20 Second Boss: 50 % slow a push resist, immune stun, slam 10 % HP vezim na 0.5 s
        20 => UnitDef { name:"BOSS 2", cost:1000, hp:2200.0, income:20, speed_mul:1.0, radius:26.0, color:Color::srgb(0.7,0.1,0.1), image:None },
        // 21 Chimaera (Air, rychla 120 %): dual attack
        21 => UnitDef { name:"CHIMERA", cost:1200, hp:2700.0, income:22, speed_mul:1.2, radius:20.0, color:Color::srgb(0.9,0.6,0.2), image:None },
        // 22 Doom Guard: cleave 50 % dmg okolnim creepum v 100 radius
        22 => UnitDef { name:"DOOM", cost:1400, hp:3300.0, income:24, speed_mul:1.0, radius:22.0, color:Color::srgb(0.6,0.1,0.3), image:None },
        // 23 Lich: na smrt Frost Nova 150 radius
        23 => UnitDef { name:"LICH", cost:1600, hp:4000.0, income:26, speed_mul:1.0, radius:22.0, color:Color::srgb(0.5,0.7,0.9), image:None },
        // 24 Pit Lord: stomp kazdych 5 s, stun 0.5 s vezim v 300 radius, 100 % push resist
        24 => UnitDef { name:"PITLORD", cost:1800, hp:4800.0, income:28, speed_mul:1.0, radius:24.0, color:Color::srgb(0.5,0.2,0.1), image:None },
        // 25 Third Boss: 30 % resist (slow/push/burn/poison), enrage 40 % HP +50 % rychlost
        25 => UnitDef { name:"BOSS 3", cost:2000, hp:6000.0, income:30, speed_mul:1.0, radius:28.0, color:Color::srgb(0.9,0.3,0.1), image:None },
        // 26 Phoenix (Air): rebirth po smrti s 50 % HP, fire aura 8 dmg/s
        26 => UnitDef { name:"PHOENIX", cost:2300, hp:7200.0, income:32, speed_mul:1.0, radius:24.0, color:Color::srgb(1.0,0.55,0.1), image:None },
        // 27 Frostmourne Knight: lifesteal aura, +15 % HP aura jako Captain
        27 => UnitDef { name:"KNIGHT", cost:2600, hp:8500.0, income:34, speed_mul:1.0, radius:24.0, color:Color::srgb(0.3,0.4,0.8), image:None },
        // 28 Archlich: na smrt 3x Skeleton 150 HP + Frost Nova 200 radius
        28 => UnitDef { name:"ARCHLICH", cost:3000, hp:10000.0, income:36, speed_mul:1.0, radius:26.0, color:Color::srgb(0.3,0.5,0.7), image:None },
        // 29 Void Walker (Air): 20 % dodge, magic resist
        29 => UnitDef { name:"VOID", cost:3400, hp:12000.0, income:38, speed_mul:1.0, radius:26.0, color:Color::srgb(0.3,0.1,0.5), image:None },
        // 30 Final Boss: immune stun, 70 % resisty, enrage 40 % HP +50 % rychlost, na smrt exploze 500 dmg vezim v 300 radius
        _ => UnitDef { name:"FINAL", cost:4000, hp:15000.0, income:40, speed_mul:1.0, radius:32.0, color:Color::srgb(0.9,0.05,0.05), image:None },
    }
}

// letajici jednotky (Wyvern, Gryphon): lete rovne a trefi je jen Air veze
pub fn is_air(kind: u8) -> bool { matches!(kind, 9 | 11 | 18 | 21 | 26 | 29) }

// AURY NA VEZE: veze v radiu creepu dostanou upravy (nasobky; 1.0 = beze zmeny); random_target = sance na nahodny cil; stomp = kazdych stomp_every s omraceni stomp_stun s
pub struct TowerAuraDef { pub radius: f32, pub random_target: f32, pub dmg_mul: f32, pub range_mul: f32, pub rate_mul: f32, pub stomp_every: f32, pub stomp_stun: f32 }
pub fn tower_aura_for(kind: u8) -> Option<TowerAuraDef> {
    match kind {
        19 => Some(TowerAuraDef { radius: 200.0, random_target: 0.25, dmg_mul: 1.0, range_mul: 1.0, rate_mul: 1.0, stomp_every: 0.0, stomp_stun: 0.0 }), // Infernal: 25 % nahodny cil
        24 => Some(TowerAuraDef { radius: 300.0, random_target: 0.0, dmg_mul: 1.0, range_mul: 1.0, rate_mul: 1.0, stomp_every: 5.0, stomp_stun: 0.5 }), // Pit Lord: stomp
        26 => Some(TowerAuraDef { radius: 120.0, random_target: 0.0, dmg_mul: 0.75, range_mul: 1.0, rate_mul: 1.0, stomp_every: 0.0, stomp_stun: 0.0 }), // Phoenix: -25 % dmg, maly radius ODHAD
        27 => Some(TowerAuraDef { radius: 200.0, random_target: 0.0, dmg_mul: 1.0, range_mul: 0.75, rate_mul: 1.0, stomp_every: 0.0, stomp_stun: 0.0 }), // Knight: dosah -25 % ODHAD
        30 => Some(TowerAuraDef { radius: 300.0, random_target: 0.0, dmg_mul: 0.6, range_mul: 0.7, rate_mul: 0.7, stomp_every: 0.0, stomp_stun: 0.0 }), // Final Boss: mega aura, rychlost -30 %, dmg -40 %, dosah -30 % ODHAD
        _ => None,
    }
}

// SHIMMER (vizual odolnosti): pulzujici barevny kruh okolo tela; period = delka jednoho pulzu v sekundach
pub struct ShimmerDef { pub color: Color, pub period: f32 }
pub fn shimmer_for(kind: u8) -> Option<ShimmerDef> {
    match kind {
        14 => Some(ShimmerDef { color: Color::srgba(0.7, 0.3, 0.9, 0.25), period: 0.8 }), // Sorceress
        _ => None,
    }
}

// AURA (Captain): creepum v radiu zvysi max HP o hp_bonus (0.15 = +15 %), dokud je Captain nazivu a blizko
pub struct AuraDef { pub radius: f32, pub hp_bonus: f32 }
pub fn aura_for(kind: u8) -> Option<AuraDef> {
    match kind {
        13 => Some(AuraDef { radius: 250.0, hp_bonus: 0.15 }), // Captain
        _ => None,
    }
}

// ZPETNA STRELBA (Frost Archer): kazdych interval s zasahne nejblizsi vez v dosahu a zpomali jeji strelbu o slow na secs sekund
pub struct ShootBackDef { pub range: f32, pub interval: f32, pub slow: f32, pub secs: f32 }
pub fn shoot_back_for(kind: u8) -> Option<ShootBackDef> {
    match kind {
        8 => Some(ShootBackDef { range: 150.0, interval: 1.5, slow: 0.05, secs: 1.0 }), // Frost Archer (interval > secs, aby se ledovy efekt mezi zasahy stihl vypnout)
        _ => None,
    }
}

// LECENI (Priest): kazdou sekundu doplni hps HP vsem creepum v radiu (krome sebe)
pub struct HealDef { pub radius: f32, pub hps: f32 }
pub fn heal_for(kind: u8) -> Option<HealDef> {
    match kind {
        12 => Some(HealDef { radius: 200.0, hps: 2.0 }), // Priest
        _ => None,
    }
}

// ODOLNOSTI: stun_immune = nejde omracit, push = podil odhozeni, ktery jednotka ignoruje (0.5 = pushne ji jen o pulku), slow = podil zpomaleni, ktery ignoruje
pub struct ResistDef { pub stun_immune: bool, pub push: f32, pub slow: f32, pub dmg_mul: [f32; 8] } // dmg_mul: [-, Arrow, Cannon, Frost, Fire, Poison, Sniper, Shock]
pub fn resist_for(kind: u8) -> Option<ResistDef> {
    match kind {
        10 => Some(ResistDef { stun_immune: false, push: 0.5, slow: 0.2, dmg_mul: [1.0; 8] }), // Ogre: tank, 50 % push resist, 20 % slow resist
        14 => Some(ResistDef { stun_immune: false, push: 0.0, slow: 0.0, dmg_mul: [1.0, 1.0, 1.0, 0.5, 0.5, 1.0, 1.0, 1.0] }), // Sorceress: Frost a Fire jen 50 % damage
        15 => Some(ResistDef { stun_immune: true, push: 0.5, slow: 0.0, dmg_mul: [1.0; 8] }), // Boss
        20 => Some(ResistDef { stun_immune: true, push: 0.5, slow: 0.5, dmg_mul: [1.0; 8] }), // Second Boss
        25 => Some(ResistDef { stun_immune: false, push: 0.3, slow: 0.3, dmg_mul: [1.0, 1.0, 1.0, 1.0, 0.7, 0.7, 1.0, 1.0] }), // Third Boss: 30 % slow/push, Fire a Poison dmg -30 %
        30 => Some(ResistDef { stun_immune: true, push: 0.7, slow: 0.7, dmg_mul: [1.0, 1.0, 1.0, 1.0, 0.3, 0.3, 1.0, 1.0] }), // Final Boss: 70 % slow/push, Fire a Poison dmg -70 %
        19 => Some(ResistDef { stun_immune: false, push: 0.0, slow: 0.0, dmg_mul: [1.0, 1.0, 1.0, 1.0, 0.7, 1.0, 1.0, 1.0] }), // Infernal: Fire dmg -30 %
        24 => Some(ResistDef { stun_immune: false, push: 1.0, slow: 0.0, dmg_mul: [1.0; 8] }), // Pit Lord: 100 % push resist
        _ => None,
    }
}

// ENRAGE (zurivost): pod hp_frac max HP dostane jednotka +speed_bonus rychlosti a +size_bonus velikosti (0.30 = +30 %), flash = barva probliknuti
pub struct EnrageDef { pub hp_frac: f32, pub speed_bonus: f32, pub size_bonus: f32, pub flash: Color }

pub fn enrage_for(kind: u8) -> Option<EnrageDef> {
    match kind {
        7 => Some(EnrageDef { hp_frac: 0.30, speed_bonus: 0.30, size_bonus: 0.30, flash: Color::srgba(1.0, 0.1, 0.1, 0.65) }), // Berserker
        15 => Some(EnrageDef { hp_frac: 0.30, speed_bonus: 0.50, size_bonus: 0.0, flash: Color::srgba(1.0, 0.1, 0.1, 0.65) }), // Boss: jen rychlost, velikost se nemeni
        25 => Some(EnrageDef { hp_frac: 0.40, speed_bonus: 0.50, size_bonus: 0.0, flash: Color::srgba(1.0, 0.1, 0.1, 0.65) }), // Third Boss
        30 => Some(EnrageDef { hp_frac: 0.40, speed_bonus: 0.50, size_bonus: 0.0, flash: Color::srgba(1.0, 0.1, 0.1, 0.65) }), // Final Boss
        25 => Some(EnrageDef { hp_frac: 0.40, speed_bonus: 0.50, size_bonus: 0.0, flash: Color::srgba(1.0, 0.1, 0.1, 0.65) }), // Third Boss
        30 => Some(EnrageDef { hp_frac: 0.40, speed_bonus: 0.50, size_bonus: 0.0, flash: Color::srgba(1.0, 0.1, 0.1, 0.65) }), // Final Boss
        _ => None,
    }
}

// INCOME SCALING (proti nekonecnemu snowballu): kolikata odeslana jednotka celkem -> nasobek jejiho income
// (do kolika odeslani, nasobek); po poslednim pasmu uz zadny dalsi income
// (do kolikateho odeslani TETO jednotky, nasobek income): 1-50 = 100 %, 51-100 = 80 %, 101-200 = 60 %, 201-300 = 35 %, 301-500 = 15 %, dal 0 %
const INCOME_TIERS: [(u32, f32); 5] = [(50, 1.0), (100, 0.8), (200, 0.6), (300, 0.35), (500, 0.15)];

pub fn income_multiplier(sent_before: u32) -> f32 {
    for (limit, mul) in INCOME_TIERS {
        if sent_before < limit { return mul; }
    }
    0.0
}

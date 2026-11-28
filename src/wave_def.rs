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
        7 => WaveDef { name:"Bandit", code:"n00R", hp:170.0, count:9, total_hp:1530.0, req_dps:70.0, time_est:28.3, color:Color::srgb(0.9,0.6,0.1), is_mega:false, is_sudden:false, boss_hp:0.0 },
        8 => WaveDef { name:"Necromancer SUDDEN DEATH", code:"n00S", hp:190.0, count:10, total_hp:1900.0, req_dps:120.0, time_est:50.0, color:Color::srgb(0.6,0.1,0.9), is_mega:false, is_sudden:true, boss_hp:0.0 },
        9 => WaveDef { name:"Dark Mage", code:"n00T", hp:210.0, count:10, total_hp:2100.0, req_dps:180.0, time_est:23.3, color:Color::srgb(0.3,0.2,0.8), is_mega:false, is_sudden:false, boss_hp:0.0 },
        10 => WaveDef { name:"Ogre Lord MEGA", code:"n00U", hp:250.0, count:9, total_hp:2250.0, req_dps:220.0, time_est:32.7, color:Color::srgb(0.8,0.15,0.15), is_mega:true, is_sudden:false, boss_hp:1800.0 },
        11 => WaveDef { name:"Demon", code:"n00V", hp:280.0, count:11, total_hp:3080.0, req_dps:280.0, time_est:31.4, color:Color::srgb(1.0,0.2,0.2), is_mega:false, is_sudden:false, boss_hp:0.0 },
        12 => WaveDef { name:"Dragon Whelp", code:"n00W", hp:310.0, count:12, total_hp:3720.0, req_dps:350.0, time_est:37.7, color:Color::srgb(0.2,0.8,0.3), is_mega:false, is_sudden:false, boss_hp:0.0 },
        13 => WaveDef { name:"Infernal", code:"n00X", hp:350.0, count:12, total_hp:4200.0, req_dps:450.0, time_est:38.6, color:Color::srgb(1.0,0.4,0.0), is_mega:false, is_sudden:false, boss_hp:0.0 },
        14 => WaveDef { name:"Pit Lord", code:"n00Y", hp:390.0, count:13, total_hp:5070.0, req_dps:550.0, time_est:44.9, color:Color::srgb(0.5,0.0,0.0), is_mega:false, is_sudden:false, boss_hp:0.0 },
        15 => WaveDef { name:"Sheep King FINAL", code:"n00Z", hp:420.0, count:15, total_hp:6300.0, req_dps:700.0, time_est:65.0, color:Color::srgb(1.0,0.9,0.0), is_mega:true, is_sudden:false, boss_hp:5000.0 },
        _ => {
            if wave <= UNIT_COUNT as u32 { return unit_wave(wave); } // vlny 16-34 podle posilanych jednotek
            // vlna 35+: navazuje na vlnu 34, +12 % HP za vlnu (k tomu pojistka LATE_ROUND), +1 creep za 2 vlny, max 40 kusu (mobil)
            let n = wave.saturating_sub(UNIT_COUNT as u32) as f32; // saturating: vlna 0 nepadne na podteceni
            let mul = 1.12_f32.powf(n);
            let count = (WAVE_COUNT_LATE + (n as usize) / 2).min(40);
            let hp = unit_wave(UNIT_COUNT as u32).hp * mul;
            WaveDef { name:"Infinite", code:"n00Z", hp, count, total_hp: hp * count as f32, req_dps: 700.0 * mul, time_est:65.0, color:Color::srgb(1.0,1.0,1.0), is_mega:true, is_sudden:false, boss_hp: hp * WAVE_BOSS_MUL }
        }
    }
}

// VLNY 16-34 podle posilanych jednotek: vlna N = 15 creepu jednotky N (jmeno, barva, HP x WAVE_HP_SCALE).
// Ladeni (2026-11-28, model ekonomiky + bot vs bot): HP vlny ma byt zhruba 1x (vlna 20) az 2,2x (vlna 30)
// tolik, kolik ma hrac goldu ve vezich; od vlny 31 pojistka (LATE_ROUND) zapas rychle ukonci.
const WAVE_HP_SCALE: f32 = 0.25;   // HP creepu vlny = HP jednotky x tolik (spawn ho jeste nasobi skalou vlny, max 2,2)
const WAVE_COUNT_LATE: usize = 15; // kolik creepu ma kazda vlna 16-34
const WAVE_BOSS_MUL: f32 = 3.0;    // HP mega bosse na konci kazde 5. vlny = HP creepu x tolik

fn unit_wave(wave: u32) -> WaveDef {
    let u = get_unit_def(wave as u8);
    let hp = u.hp * WAVE_HP_SCALE;
    let boss = wave % 5 == 0;
    WaveDef { name: u.name, code: "n00Z", hp, count: WAVE_COUNT_LATE, total_hp: hp * WAVE_COUNT_LATE as f32, req_dps: 700.0, time_est: 65.0, color: u.color, is_mega: boss, is_sudden: false, boss_hp: if boss { hp * WAVE_BOSS_MUL } else { 0.0 } }
}

pub fn get_creep_color_and_hp(wave: u32) -> (Color, f32) {
    let wd = get_wave_def(wave);
    (wd.color, wd.hp)
}

// === JEDNOTKY K ODESLANI (panel S) ===
pub const UNIT_COUNT: u8 = 34; // kolik jednotek je v tabulce nize (panel S se podle toho postavi)

// druhy jednotek serazene podle ceny (poradi v panelu S); ID zustavaji stabilni, nove jednotky maji 31+
pub fn units_by_cost() -> Vec<u8> {
    let mut kinds: Vec<u8> = (1..=UNIT_COUNT).collect();
    kinds.sort_by_key(|&k| (get_unit_def(k).cost, k));
    kinds
}

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
        2 => UnitDef { name:"DAMIAN", cost:10, hp:20.0, income:2, speed_mul:1.0, radius:16.0, color:Color::srgb(0.9,0.9,0.9), image:Some("creeps/creep2.png") },
        3 => UnitDef { name:"OLGA", cost:15, hp:30.0, income:2, speed_mul:1.0, radius:12.0, color:Color::srgb(0.95,0.5,0.7), image:Some("creeps/creep3.png") },
        4 => UnitDef { name:"KAMIL", cost:25, hp:50.0, income:3, speed_mul:1.0, radius:13.0, color:Color::srgb(0.95,0.6,0.15), image:Some("creeps/creep4.png") },
        5 => UnitDef { name:"LUKAS", cost:40, hp:80.0, income:4, speed_mul:1.0, radius:14.0, color:Color::srgb(0.2,0.85,0.85), image:Some("creeps/creep5.png") },
        // 6 Rogue: rychly 120 %, tezko trefitelny
        6 => UnitDef { name:"ROGUE", cost:60, hp:120.0, income:5, speed_mul:1.2, radius:16.0, color:Color::WHITE, image:Some("creeps/creep6.png") },
        // 7 Berserker: enrage pod 30 % HP +30 % rychlost
        7 => UnitDef { name:"BERSERK", cost:85, hp:180.0, income:6, speed_mul:1.0, radius:17.0, color:Color::WHITE, image:Some("creeps/creep7.png") },
        // 8 Frost Archer: strili zpet - slow 5 % na veze 1 s
        8 => UnitDef { name:"ARCHER", cost:110, hp:250.0, income:7, speed_mul:1.0, radius:18.0, color:Color::WHITE, image:Some("creeps/creep8.png") },
        // 9 Wyvern: letajici - trefi ho jen Air veze (Sniper, Frost, Fire, Shock), Cannon ne
        9 => UnitDef { name:"WYVERN", cost:140, hp:330.0, income:8, speed_mul:1.0, radius:18.0, color:Color::WHITE, image:Some("creeps/air9.png") },
        // 10 Ogre: tezky - 50 % resist na push, 20 % resist na slow
        10 => UnitDef { name:"OGRE", cost:180, hp:440.0, income:9, speed_mul:1.0, radius:20.0, color:Color::WHITE, image:Some("creeps/creep10.png") },
        // 11 Gryphon: letajici rychly 130 %
        11 => UnitDef { name:"GRYPHON", cost:220, hp:550.0, income:10, speed_mul:1.3, radius:15.0, color:Color::srgb(0.9,0.75,0.3), image:Some("creeps/air11.png") },
        // 12 Priest: healer - leci 2 HP/s vsechny creepy v 200 radius
        12 => UnitDef { name:"PRIEST", cost:270, hp:690.0, income:11, speed_mul:1.0, radius:15.0, color:Color::srgb(0.95,0.9,0.95), image:Some("creeps/creep12.png") },
        // 13 Captain: aura +15 % max HP vsem creepum v 250 radius
        13 => UnitDef { name:"CAPTAIN", cost:330, hp:870.0, income:12, speed_mul:1.0, radius:16.0, color:Color::srgb(0.3,0.5,0.95), image:Some("creeps/creep13.png") },
        // 14 Sorceress: magic immune 50 % - jen 50 % dmg z Fire a Frost
        14 => UnitDef { name:"SORCER.", cost:400, hp:1100.0, income:13, speed_mul:1.0, radius:17.0, color:Color::srgb(0.7,0.3,0.9), image:Some("creeps/creep14.png") },
        // 15 First Boss: immune stun, 50 % push resist, enrage pod 30 % HP +50 % rychlost + cerveny pulz
        15 => UnitDef { name:"BOSS", cost:500, hp:1400.0, income:15, speed_mul:1.0, radius:24.0, color:Color::srgb(0.9,0.15,0.15), image:Some("creeps/boss1.png") },
        // 16-30 (late game): zatim JEN staty, Air a rychlost; zvlastni schopnosti se doplni po jedne
        // 16 Enchantress: mana burn 5 veze v 300 radius
        16 => UnitDef { name:"ENCHANT", cost:600, hp:1750.0, income:16, speed_mul:1.0, radius:18.0, color:Color::srgb(0.85,0.4,0.85), image:None },
        // 17 Necromancer: na smrt 2x Skeleton 80 HP
        17 => UnitDef { name:"NECRO", cost:700, hp:2050.0, income:17, speed_mul:1.0, radius:18.0, color:Color::srgb(0.4,0.7,0.4), image:None },
        // 18 Frost Wyrm (Air): frost breath aura, slow na veze v 250 radius
        18 => UnitDef { name:"WYRM", cost:800, hp:2400.0, income:18, speed_mul:1.0, radius:20.0, color:Color::srgb(0.5,0.8,1.0), image:None },
        // 19 Infernal: fire aura 5 dmg/s vezim v 200 radius, 30 % resist Fire
        19 => UnitDef { name:"INFERNAL", cost:900, hp:2750.0, income:19, speed_mul:1.0, radius:20.0, color:Color::srgb(1.0,0.4,0.1), image:None },
        // 20 Second Boss: 50 % slow a push resist, immune stun, slam 10 % HP vezim na 0.5 s
        20 => UnitDef { name:"BOSS 2", cost:1000, hp:3100.0, income:20, speed_mul:1.0, radius:26.0, color:Color::srgb(0.7,0.1,0.1), image:None },
        // 21 Chimaera (Air, rychla 120 %): dual attack
        21 => UnitDef { name:"CHIMERA", cost:1200, hp:3750.0, income:22, speed_mul:1.2, radius:20.0, color:Color::srgb(0.9,0.6,0.2), image:None },
        // 22 Doom Guard: cleave 50 % dmg okolnim creepum v 100 radius
        22 => UnitDef { name:"DOOM", cost:1400, hp:4500.0, income:24, speed_mul:1.0, radius:22.0, color:Color::srgb(0.6,0.1,0.3), image:None },
        // 23 Lich: na smrt Frost Nova 150 radius
        23 => UnitDef { name:"LICH", cost:1600, hp:5200.0, income:26, speed_mul:1.0, radius:22.0, color:Color::srgb(0.5,0.7,0.9), image:None },
        // 24 Pit Lord: stomp kazdych 5 s, stun 0.5 s vezim v 300 radius, 100 % push resist
        24 => UnitDef { name:"PITLORD", cost:1800, hp:5900.0, income:28, speed_mul:1.0, radius:24.0, color:Color::srgb(0.5,0.2,0.1), image:None },
        // 25 Third Boss: 30 % resist (slow/push/burn/poison), enrage 40 % HP +50 % rychlost
        25 => UnitDef { name:"BOSS 3", cost:2000, hp:6700.0, income:30, speed_mul:1.0, radius:28.0, color:Color::srgb(0.9,0.3,0.1), image:None },
        // 26 Phoenix (Air): rebirth po smrti s 50 % HP, fire aura 8 dmg/s
        26 => UnitDef { name:"PHOENIX", cost:2300, hp:7800.0, income:32, speed_mul:1.0, radius:24.0, color:Color::srgb(1.0,0.55,0.1), image:None },
        // 27 Frostmourne Knight: lifesteal aura, +15 % HP aura jako Captain
        27 => UnitDef { name:"KNIGHT", cost:2600, hp:8900.0, income:34, speed_mul:1.0, radius:24.0, color:Color::srgb(0.3,0.4,0.8), image:None },
        // 28 Archlich: na smrt 3x Skeleton 150 HP + Frost Nova 200 radius
        28 => UnitDef { name:"ARCHLICH", cost:3000, hp:10500.0, income:36, speed_mul:1.0, radius:26.0, color:Color::srgb(0.3,0.5,0.7), image:None },
        // 29 Void Walker (Air): 20 % dodge, magic resist
        29 => UnitDef { name:"VOID", cost:3400, hp:12000.0, income:38, speed_mul:1.0, radius:26.0, color:Color::srgb(0.3,0.1,0.5), image:None },
        // 30 Final Boss: immune stun, 70 % resisty, enrage 40 % HP +50 % rychlost, na smrt exploze 500 dmg vezim v 300 radius
        30 => UnitDef { name:"FINAL", cost:4000, hp:14400.0, income:40, speed_mul:1.0, radius:32.0, color:Color::srgb(0.9,0.05,0.05), image:None },
        // === 31-34: pozdni jednotky pro vlny 30+ (vlny 31-34 jsou podle nich) ===
        // 31 Titan: obri tank - pomaly 85 %, immune stun, 80 % resist push, 50 % resist slow
        31 => UnitDef { name:"TITAN", cost:4600, hp:16800.0, income:42, speed_mul:0.85, radius:30.0, color:Color::srgb(0.55,0.55,0.6), image:None },
        // 32 Banshee (Air, rychla 125 %): Frost a Fire jen 50 % dmg, svetly shimmer
        32 => UnitDef { name:"BANSHEE", cost:5300, hp:19600.0, income:44, speed_mul:1.25, radius:22.0, color:Color::srgb(0.75,0.85,1.0), image:None },
        // 33 Hierophant: velky lecitel - leci 60 HP/s vsechny creepy v 250 radius
        33 => UnitDef { name:"HIEROPH.", cost:6100, hp:22900.0, income:46, speed_mul:1.0, radius:24.0, color:Color::srgb(1.0,0.95,0.6), image:None },
        // 34 Overlord: aura +15 % max HP creepum v 250 radius, enrage pod 40 % HP +50 % rychlost
        34 => UnitDef { name:"OVERLORD", cost:7000, hp:26700.0, income:48, speed_mul:1.0, radius:32.0, color:Color::srgb(0.35,0.05,0.25), image:None },
        _ => get_unit_def(30),
    }
}

// letajici jednotky (Wyvern, Gryphon): lete rovne a trefi je jen Air veze
pub fn is_air(kind: u8) -> bool { matches!(kind, 9 | 11 | 18 | 21 | 26 | 29 | 32) }

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
        32 => Some(ShimmerDef { color: Color::srgba(0.75, 0.85, 1.0, 0.25), period: 0.6 }), // Banshee
        _ => None,
    }
}

// AURA (Captain): creepum v radiu zvysi max HP o hp_bonus (0.15 = +15 %), dokud je Captain nazivu a blizko
pub struct AuraDef { pub radius: f32, pub hp_bonus: f32 }
pub fn aura_for(kind: u8) -> Option<AuraDef> {
    match kind {
        13 => Some(AuraDef { radius: 250.0, hp_bonus: 0.15 }), // Captain
        34 => Some(AuraDef { radius: 250.0, hp_bonus: 0.15 }), // Overlord
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
        12 => Some(HealDef { radius: 200.0, hps: 8.0 }), // Priest (asi 1 % jeho HP za sekundu)
        33 => Some(HealDef { radius: 250.0, hps: 60.0 }), // Hierophant
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
        31 => Some(ResistDef { stun_immune: true, push: 0.8, slow: 0.5, dmg_mul: [1.0; 8] }), // Titan
        32 => Some(ResistDef { stun_immune: false, push: 0.0, slow: 0.0, dmg_mul: [1.0, 1.0, 1.0, 0.5, 0.5, 1.0, 1.0, 1.0] }), // Banshee: Frost a Fire jen 50 %
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
        34 => Some(EnrageDef { hp_frac: 0.40, speed_bonus: 0.50, size_bonus: 0.0, flash: Color::srgba(1.0, 0.1, 0.1, 0.65) }), // Overlord
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

#[cfg(test)]
mod tests {
    use super::*;

    // HP normalniho creepa tak, jak ho spawne kolo (skala vlny, sudden death, pojistka)
    fn creep_hp(wave: u32) -> f32 {
        let w = get_wave_def(wave);
        let scale = (1.0 + wave as f32 * crate::constants::CREEP_GROWTH_PER_WAVE).min(crate::constants::CREEP_GROWTH_MAX_SCALE);
        let late = if wave <= crate::constants::LATE_ROUND { 1.0 } else { crate::constants::LATE_ROUND_HP_GROWTH.powi((wave - crate::constants::LATE_ROUND) as i32) };
        w.hp * scale * late * if w.is_sudden { 2.0 } else { 1.0 }
    }

    #[test]
    fn waves_get_stronger_every_round_up_to_forty() {
        for wave in 2..=40 {
            if wave == 9 { continue; } // vlna 8 je sudden death (dvojnasobne HP), vlna 9 je zase normalni
            assert!(creep_hp(wave) > creep_hp(wave - 1), "vlna {} neni silnejsi nez {}", wave, wave - 1);
        }
    }

    #[test]
    fn wave_thirty_stays_in_a_playable_range() {
        // ladeno na ~1x (vlna 20) az ~2x (vlna 30) goldu ve vezich; driv mela vlna 30 creepy po 334 000 HP
        assert!(creep_hp(20) < 3_000.0);
        assert!(creep_hp(30) < 15_000.0);
        assert!(creep_hp(36) > 3.0 * creep_hp(30)); // pojistka po 30. vlne
    }

    #[test]
    fn pricier_units_never_give_less_hp_per_gold() {
        let kinds = units_by_cost();
        assert_eq!(kinds.len(), UNIT_COUNT as usize);
        for pair in kinds.windows(2) {
            let (a, b) = (get_unit_def(pair[0]), get_unit_def(pair[1]));
            assert!(b.hp / b.cost as f32 >= a.hp / a.cost as f32 - 0.01, "{} ma mene HP za gold nez {}", b.name, a.name);
        }
    }

    #[test]
    fn every_unit_has_its_own_definition() {
        let mut names: Vec<&str> = (1..=UNIT_COUNT).map(|k| get_unit_def(k).name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), UNIT_COUNT as usize);
    }
}

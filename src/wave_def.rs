// DATA PODLE FINALNIHO GDD (Meme TD 1v1, verze 1.0): typy creepu, bossove, vlny 1-30, posilane jednotky,
// damage typy a pravidla zivotu / diamantu. Zadne kresleni ani Bevy systemy - jen tabulky a ciste funkce,
// aby slo jadro pozdeji presunout do Rust core / serveru (GDD 14).
use bevy::prelude::Color;

// === 9. CREEPOVE (10 typu + Danovy Optimalizator z tabulky vln) ===
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CreepType {
    Wojak,         // zaklad, zadna schopnost
    Zoomer,        // Zoomer Doomer: rychly 1,55x, slaby na led (Mrazak)
    Zombi,         // Tlusty Zombi z Lidlu: tank 0,6x, slaby na ohen (Toustovac)
    Karen,         // Karen Manazerka: obrnena (20 % fyzicka odolnost -> fyzicke 75 %)
    WifiUpir,      // Wi-Fi Upir: regeneruje 1,5 % HP/s, kdyz 2 s nedostane zasah
    Influencer,    // Cringe Influencer: aura +10 % rychlost okolnim creepum
    AdBlock,       // AdBlock Rytir: zacina se stitem 30 % HP
    KopirBot,      // Kopirovaci Bot: po smrti 2 kopie s 20 % HP
    Brambora,      // Lagujici Brambora: kazde 3 s nahodna rychlost 0,8-1,3x
    Chad,          // Chad Kuryr Elite: 1,15x, pod 50 % HP +25 % rychlost, obrneny
    Optimalizator, // Danovy Optimalizator: je ve vlnach 10, 18, 24, v seznamu creepu chybi -> obrneny + magicka odolnost
}

pub struct CreepTypeDef {
    pub name: &'static str,
    pub speed: f32,          // nasobek zakladni rychlosti
    pub armored: bool,       // fyzicke 75 %, pruzarne a magicke 125 %
    pub fast: bool,          // fyzicke 125 %
    pub magic_resist: bool,  // magicke 75 %
    pub kill_gold: i32,      // GDD 4.1: 2 az 4 podle typu
    pub image: &'static str,
}

pub fn creep_def(t: CreepType) -> CreepTypeDef {
    use CreepType::*;
    let d = |name, speed, armored, fast, magic_resist, kill_gold, _color: Color, image| CreepTypeDef { name, speed, armored, fast, magic_resist, kill_gold, image };
    match t {
        Wojak => d("Wojak", 1.0, false, false, false, 2, Color::srgb(0.85, 0.8, 0.7), "creeps/creep1.png"),
        Zoomer => d("Zoomer Doomer", 1.55, false, true, false, 2, Color::srgb(0.3, 0.9, 0.9), "creeps/creep6.png"),
        Zombi => d("Tlusty Zombi", 0.6, false, false, false, 4, Color::srgb(0.4, 0.7, 0.3), "creeps/creep10.png"),
        Karen => d("Karen", 0.85, true, false, false, 3, Color::srgb(0.95, 0.6, 0.75), "creeps/creep3.png"),
        WifiUpir => d("Wi-Fi Upir", 0.95, false, false, false, 3, Color::srgb(0.55, 0.3, 0.8), "creeps/creep14.png"),
        Influencer => d("Cringe Influencer", 1.0, false, false, false, 3, Color::srgb(1.0, 0.5, 0.8), "creeps/creep13.png"),
        AdBlock => d("AdBlock Rytir", 0.9, false, false, false, 3, Color::srgb(0.6, 0.65, 0.75), "creeps/creep8.png"),
        KopirBot => d("Kopirovaci Bot", 1.0, false, false, false, 2, Color::srgb(0.7, 0.7, 0.7), "creeps/creep5.png"),
        Brambora => d("Lagujici Brambora", 1.0, false, false, false, 2, Color::srgb(0.8, 0.65, 0.3), "creeps/creep4.png"),
        Chad => d("Chad Kuryr", 1.15, true, false, false, 4, Color::srgb(0.95, 0.75, 0.2), "creeps/creep7.png"),
        Optimalizator => d("Danovy Optimalizator", 0.9, true, false, true, 3, Color::srgb(0.4, 0.45, 0.8), "creeps/creep2.png"),
    }
}

pub const ADBLOCK_SHIELD: f32 = 0.30;     // stit = 30 % max HP
pub const WIFI_REGEN: f32 = 0.015;        // 1,5 % max HP za sekundu...
pub const WIFI_REGEN_IDLE: f32 = 2.0;     // ...kdyz 2 s nedostal zasah
pub const KOPIR_COPIES: u32 = 2;          // po smrti 2 kopie...
pub const KOPIR_COPY_HP: f32 = 0.20;      // ...s 20 % max HP
pub const INFLUENCER_AURA: f32 = 0.10;    // +10 % rychlost okolnim creepum
pub const INFLUENCER_RADIUS: f32 = 2.0;   // v policcich
pub const BRAMBORA_EVERY: f32 = 3.0;      // kazde 3 s nova rychlost
pub const BRAMBORA_SPEED: (f32, f32) = (0.8, 1.3);
pub const CHAD_ENRAGE: (f32, f32) = (0.5, 0.25); // pod 50 % HP +25 % rychlost

// === 11. BOSSOVE ===
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossKind { Ucitel, Sef, Tchyne, DanovyUrad, Babicka, Mama }

pub struct BossDef { pub name: &'static str, pub speed: f32, pub armored: bool, pub image: &'static str, pub quote: &'static str }

pub fn boss_def(b: BossKind) -> BossDef {
    use BossKind::*;
    match b {
        Ucitel => BossDef { name: "Pan Ucitel", speed: 0.8, armored: false, image: "creeps/boss1.png", quote: "Kdo nebyl pripraven, propada." },
        Sef => BossDef { name: "Sef z Prace", speed: 0.8, armored: false, image: "creeps/boss2.png", quote: "Muzes zustat prescas?" },
        Tchyne => BossDef { name: "Tchyne", speed: 0.75, armored: false, image: "creeps/boss1.png", quote: "Uz zase hrajes?" },
        DanovyUrad => BossDef { name: "Danovy Urad", speed: 0.7, armored: true, image: "creeps/boss2.png", quote: "Mate to v trojim vyhotoveni?" },
        Babicka => BossDef { name: "Babicka s Vareckou", speed: 1.3, armored: false, image: "creeps/boss1.png", quote: "Pojd domu!" },
        Mama => BossDef { name: "Tvoje Mama Vola", speed: 0.75, armored: false, image: "creeps/boss2.png", quote: "Okamzite vypni ten pocitac!" },
    }
}

// schopnosti bossu (GDD 11), cisla na jednom miste
pub const UCITEL_GUARD: (f32, f32, f32) = (8.0, 2.0, 0.35);   // kazdych 8 s na 2 s -35 % fyzickeho dmg
pub const UCITEL_TOWER_SLOW: f32 = 0.85;                       // tabule: veze v okoli strili o 15 % pomaleji
pub const SEF_ENRAGE: (f32, f32) = (0.5, 0.30);                // pod 50 % HP +30 % rychlost
pub const SEF_SPEED_AURA: f32 = 0.20;                          // +20 % rychlost creepum
pub const TCHYNE_TOWER_DMG: f32 = 0.85;                        // -15 % dmg vezi v okoli
pub const URAD_DISABLE: (f32, f32) = (7.0, 2.0);               // kazdych 7 s vypne 1 nahodnou vez na 2 s
pub const URAD_LEAK_GOLD: i32 = 100;                           // kdyz projde, sebere 100 goldu
pub const BABICKA_STUN: (f32, f32) = (5.0, 1.0);               // varecka: kazdych 5 s omraci veze v okoli na 1 s
pub const MAMA_PHASES: (f32, f32) = (0.66, 0.33);              // hranice fazi (podil HP)
pub const MAMA_SUMMON_EVERY: f32 = 6.0;                        // faze 1: privola Kopirovaciho Bota
pub const MAMA_SHIELD: (f32, f32) = (8.0, 0.15);               // faze 2: kazdych 8 s stit 15 % max HP
pub const MAMA_RAGE: (f32, f32) = (0.40, 0.20);                // faze 3: +40 % rychlost, +20 % cisteho dmg
pub const MAMA_CALL: (f32, f32) = (10.0, 3.0);                 // kazdych 10 s vypne nahodnou vez na 3 s
pub const BOSS_AURA_RADIUS: f32 = 3.0;                         // v policcich: dosah aur a debuffu bossu na veze

// === 10. VLNY 1-30 ===
pub struct WaveDef {
    pub small: (CreepType, u32, f32),        // (typ, pocet, HP)
    pub big: Option<(CreepType, u32, f32)>,  // velci creepove (leak 2)
    pub boss: Option<(BossKind, f32)>,       // boss (leak 3)
    pub reward: i32,                         // gold za prezitou vlnu (boss vlna: + diamant)
    pub mixed: bool,                         // vlny 29+: "smisene" typy, strida se vsech 10
}

pub const LAST_WAVE: u32 = 30;
pub const ENDLESS_HP_GROWTH: f32 = 1.30; // po vlne 30: +30 % HP kazdou dalsi vlnu (sudden death)

pub fn wave_def(wave: u32) -> WaveDef {
    use CreepType::*;
    use BossKind::*;
    let w = |small, big, boss, reward| WaveDef { small, big, boss, reward, mixed: false };
    match wave {
        0 | 1 => w((Wojak, 18, 75.0), Some((Karen, 3, 220.0)), None, 45),
        2 => w((Wojak, 22, 105.0), Some((Karen, 3, 290.0)), None, 48),
        3 => w((Zoomer, 20, 95.0), Some((Chad, 3, 310.0)), None, 52),
        4 => w((Wojak, 24, 155.0), Some((WifiUpir, 4, 400.0)), None, 56),
        5 => w((AdBlock, 16, 190.0), None, Some((Ucitel, 2800.0)), 100),
        6 => w((Wojak, 28, 210.0), Some((Influencer, 3, 600.0)), None, 66),
        7 => w((Zoomer, 24, 190.0), Some((Chad, 4, 680.0)), None, 70),
        8 => w((WifiUpir, 26, 265.0), Some((Karen, 4, 760.0)), None, 74),
        9 => w((KopirBot, 30, 230.0), Some((AdBlock, 3, 920.0)), None, 78),
        10 => w((Optimalizator, 20, 340.0), None, Some((Sef, 6500.0)), 140),
        11 => w((Wojak, 34, 410.0), Some((Influencer, 4, 1100.0)), None, 90),
        12 => w((Zoomer, 30, 365.0), Some((Karen, 5, 1250.0)), None, 95),
        13 => w((Brambora, 32, 470.0), Some((WifiUpir, 4, 1450.0)), None, 100),
        14 => w((AdBlock, 36, 510.0), Some((Chad, 5, 1650.0)), None, 105),
        15 => w((KopirBot, 24, 620.0), None, Some((Tchyne, 13500.0)), 190),
        16 => w((Wojak, 40, 760.0), Some((Influencer, 5, 2000.0)), None, 120),
        17 => w((Zoomer, 36, 690.0), Some((Chad, 6, 2250.0)), None, 126),
        18 => w((WifiUpir, 38, 850.0), Some((Optimalizator, 5, 2600.0)), None, 132),
        19 => w((KopirBot, 42, 790.0), Some((AdBlock, 6, 2950.0)), None, 138),
        20 => w((Brambora, 28, 1050.0), None, Some((DanovyUrad, 27000.0)), 250),
        21 => w((Wojak, 46, 1250.0), Some((Karen, 6, 3500.0)), None, 155),
        22 => w((Zoomer, 42, 1120.0), Some((Chad, 7, 3900.0)), None, 162),
        23 => w((AdBlock, 44, 1390.0), Some((Influencer, 6, 4500.0)), None, 169),
        24 => w((WifiUpir, 48, 1520.0), Some((Optimalizator, 7, 5100.0)), None, 176),
        25 => w((KopirBot, 32, 1750.0), None, Some((Babicka, 52000.0)), 330),
        26 => w((Wojak, 52, 2050.0), Some((Karen, 7, 5900.0)), None, 195),
        27 => w((Zoomer, 48, 1850.0), Some((Chad, 8, 6700.0)), None, 205),
        28 => w((AdBlock, 50, 2300.0), Some((WifiUpir, 8, 7600.0)), None, 215),
        29 => WaveDef { small: (Wojak, 54, 2600.0), big: Some((Chad, 9, 8700.0)), boss: None, reward: 225, mixed: true },
        30 => WaveDef { small: (Chad, 40, 3200.0), big: None, boss: Some((Mama, 100000.0)), reward: 0, mixed: true },
        // po vlne 30: sudden death, vlna 30 bez bosse a s +30 % HP za kazdou dalsi vlnu
        _ => {
            let mul = ENDLESS_HP_GROWTH.powi((wave - LAST_WAVE) as i32);
            WaveDef { small: (Chad, 40, 3200.0 * mul), big: Some((Zombi, 6, 9000.0 * mul)), boss: None, reward: 0, mixed: true }
        }
    }
}

pub fn is_boss_wave(wave: u32) -> bool { wave_def(wave).boss.is_some() }

// smisene vlny (29+): typ creepa podle poradi, strida se vsech 10 typu
pub fn mixed_type(i: u32) -> CreepType {
    use CreepType::*;
    [Wojak, Zoomer, Zombi, Karen, WifiUpir, Influencer, AdBlock, KopirBot, Brambora, Chad][(i % 10) as usize]
}

// === 4.4 POSILANE JEDNOTKY ===
pub struct SendDef {
    pub name: &'static str,
    pub cost: i32,
    pub income: i32,       // +gold za tick (napoprve az pri dalsim ticku)
    pub hp_pct: f32,       // HP = tolik x HP maleho creepa aktualni vlny
    pub leak: u32,         // kolik zivotu stoji, kdyz projde
    pub creep: CreepType,  // jak vypada a co umi
    pub mini_boss: bool,   // Mini-Boss Ratio: max 2 za vlnu, vetsi
}

pub const SEND_COUNT: u8 = 10;
pub const MAX_SENDS_PER_WAVE: u32 = 12;
pub const MAX_MINI_BOSSES_PER_WAVE: u32 = 2;

pub fn send_def(kind: u8) -> SendDef {
    use CreepType::*;
    let s = |name, cost, income, hp_pct, leak, creep| SendDef { name, cost, income, hp_pct, leak, creep, mini_boss: false };
    match kind {
        1 => s("Minion z Chatu", 25, 1, 0.70, 1, Wojak),
        2 => s("Zoomer Express", 45, 1, 0.65, 1, Zoomer),
        3 => s("Karen Balicek", 80, 2, 1.30, 1, Karen),
        4 => s("Wi-Fi Upir", 120, 3, 1.15, 1, WifiUpir),
        5 => s("AdBlock Rytir", 165, 4, 1.20, 2, AdBlock),
        6 => s("Kopirovaci Bot", 220, 5, 1.10, 2, KopirBot),
        7 => s("Influencer Squad", 300, 6, 1.20, 2, Influencer),
        8 => s("Chad Kuryr", 400, 8, 1.50, 2, Chad),
        9 => s("Troll Tank", 550, 10, 2.00, 2, Zombi),
        _ => SendDef { name: "Mini-Boss Ratio", cost: 800, income: 13, hp_pct: 3.00, leak: 3, creep: Chad, mini_boss: true },
    }
}

// HP poslane jednotky: podil HP maleho creepa vlny, ktera prave bezi (pred 1. vlnou podle 1. vlny)
pub fn send_hp(kind: u8, wave: u32) -> f32 {
    send_def(kind).hp_pct * wave_def(wave.max(1)).small.2
}

// === 4.1 EKONOMIKA ===
pub const INCOME_TICK_SECS: f32 = 7.0;
pub const BASE_INCOME: i32 = 3;
pub const INCOME_CAP: i32 = 40;      // max gold za tick (base + sendy)
pub const BOSS_KILL_GOLD: i32 = 150;
pub const SELL_BEFORE_WAVE: f32 = 0.70;
pub const SELL_DURING_WAVE: f32 = 0.50;
pub const DIAMOND_GOLD: i32 = 400;   // boss, kdyz uz diamant nepotrebujes, da misto nej 400 goldu

pub fn tick_income(income: i32) -> i32 { income.min(INCOME_CAP) }

// === 2. ZIVOTY ===
pub const LEAK_SMALL: u32 = 1;
pub const LEAK_BIG: u32 = 2;
pub const LEAK_BOSS: u32 = 3;
pub const HARSH_LEAK_WAVE: u32 = 26; // od teto vlny projity creep bere aspon 2 zivoty, ale souperi prida o 1 mene

// (kolik zivotu ztrati ten, komu creep prosel, kolik ziska souper)
pub fn leak_lives(leak_cost: u32, wave: u32) -> (u32, u32) {
    if wave >= HARSH_LEAK_WAVE { let lost = leak_cost.max(2); (lost, lost - 1) } else { (leak_cost, leak_cost) }
}

// === 5. DAMAGE TYPY ===
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DmgType { Physical, Pierce, Magic, Explosive, True }

pub fn tower_dmg_type(kind: u8) -> DmgType {
    use DmgType::*;
    match kind {
        1 | 6 | 11 => Explosive,                                 // Bramboromet, Toustovac, Katapult
        5 | 10 => Pierce,                                        // Kaktus, Sniper
        4 | 7 | 8 | 9 | 15 => Magic,                             // Router, Muchomurka, Mrazak, Tesla, Danovy Urad
        16 | 25 => True,                                         // Chuck Norris, Admin Banhammer (cisty dmg jen u specialek)
        18 | 22 | 23 | 24 => Magic,                              // Shrek, Haxor, Karen, Cerna Dira
        _ => Physical,                                           // Nerfka, Babicka, DJ, Pepa, Kasicka, Tchyne, Gigachad, Doge, Trollface
    }
}

pub const DMG_STRONG: f32 = 1.25;
pub const DMG_WEAK: f32 = 0.75;
pub const SLOW_CAP: f32 = 0.50;     // spolecny limit zpomaleni
pub const BOSS_CC_MUL: f32 = 0.5;   // bossove: o 50 % kratsi CC

// co zasahovany cil ma (sbira se z komponent creepa)
#[derive(Clone, Copy, Default)]
pub struct Target {
    pub armored: bool,
    pub fast: bool,
    pub regen: bool,
    pub shielded: bool,     // ma jeste stit
    pub magic_resist: bool,
    pub boss: bool,
    pub weak_ice: bool,     // Zoomer
    pub weak_fire: bool,    // Tlusty Zombi
    pub phys_guard: f32,    // Pan Ucitel: docasna redukce fyzickeho dmg (0.35 = -35 %)
    pub true_bonus: f32,    // Mama ve 3. fazi: +20 % cisteho dmg
}

impl Target {
    pub fn of(t: CreepType) -> Self {
        let d = creep_def(t);
        Target {
            armored: d.armored, fast: d.fast, magic_resist: d.magic_resist,
            regen: t == CreepType::WifiUpir, weak_ice: t == CreepType::Zoomer, weak_fire: t == CreepType::Zombi,
            ..Default::default()
        }
    }
}

// nasobek damage: silny zasah 125 %, slaby 75 %, kdyz plati oboji, zustane 100 %
pub fn damage_mul(tower_kind: u8, t: &Target) -> f32 {
    use DmgType::*;
    let dt = tower_dmg_type(tower_kind);
    let (strong, weak) = match dt {
        Physical => (t.fast, t.armored),
        Pierce => (t.armored || t.boss, t.shielded),
        Magic => (t.armored || t.regen, t.magic_resist),
        Explosive => (false, t.boss),
        True => (false, false),
    };
    let mut m = match (strong, weak) { (true, false) => DMG_STRONG, (false, true) => DMG_WEAK, _ => 1.0 };
    if (t.weak_ice && tower_kind == crate::special::MRAZAK) || (t.weak_fire && tower_kind == crate::special::TOUSTOVAC) { m *= DMG_STRONG; }
    if dt == Physical { m *= 1.0 - t.phys_guard; }
    if dt == True { m *= 1.0 + t.true_bonus; }
    m
}

// === 4.2 DIAMANTY (domluveno 2026-11-28) ===
// Diamant dava zabity boss (kdyz ho propustis, dostane ho souper). Postavit Special stoji gold + 1 diamant,
// upgrade Special na R2 a na R3 kazdy dalsi gold + 1 diamant. Kdo uz diamanty nepotrebuje, dostane misto nej 400 goldu.
pub const SPECIAL_DIAMOND_COST: i32 = 1;

// kolik diamantu jeste strana potrebuje, aby mela obe Special veze z balicku na R3
// (ranks = nejvyssi rank postavene Special veze kazdeho druhu z balicku, 0 = nepostavena)
pub fn diamonds_needed(ranks: &[u32]) -> i32 {
    ranks.iter().map(|&r| 3 - r.min(3) as i32).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waves_follow_the_gdd_table() {
        assert_eq!(wave_def(1).small, (CreepType::Wojak, 18, 75.0));
        assert_eq!(wave_def(5).boss, Some((BossKind::Ucitel, 2800.0)));
        assert_eq!(wave_def(30).boss, Some((BossKind::Mama, 100000.0)));
        assert_eq!((1..=30).filter(|&w| is_boss_wave(w)).collect::<Vec<_>>(), vec![5, 10, 15, 20, 25, 30]);
        assert!((wave_def(32).small.2 - 3200.0 * 1.3 * 1.3).abs() < 0.5); // sudden death +30 % za vlnu
    }

    #[test]
    fn small_creeps_get_stronger_over_four_waves() {
        // vlny se stridaji (rychli Zoomeri maji mene HP), ale kazda vlna musi byt silnejsi nez 4 vlny pred ni
        for w in 5..=30 { assert!(wave_def(w).small.2 > wave_def(w - 4).small.2, "vlna {}", w); }
    }

    #[test]
    fn send_units_match_gdd_4_4() {
        assert_eq!((send_def(1).cost, send_def(1).income, send_def(1).leak), (25, 1, 1));
        assert_eq!((send_def(10).cost, send_def(10).income, send_def(10).leak), (800, 13, 3));
        assert!(send_def(10).mini_boss);
        assert!((send_hp(1, 1) - 52.5).abs() < 0.01); // 70 % z 75 HP
    }

    #[test]
    fn income_is_capped_at_forty_per_tick() {
        assert_eq!(tick_income(3), 3);
        assert_eq!(tick_income(55), 40);
    }

    #[test]
    fn from_wave_26_a_leak_costs_two_and_gives_one() {
        assert_eq!(leak_lives(LEAK_SMALL, 25), (1, 1));
        assert_eq!(leak_lives(LEAK_SMALL, 26), (2, 1));
        assert_eq!(leak_lives(LEAK_BOSS, 30), (3, 2));
    }

    #[test]
    fn damage_types_are_strong_and_weak_per_gdd_5() {
        let karen = Target::of(CreepType::Karen);
        assert_eq!(damage_mul(crate::special::BABICKA, &karen), DMG_WEAK);   // fyzicke na brneni
        assert_eq!(damage_mul(crate::special::SNIPER, &karen), DMG_STRONG);  // pruzarne na brneni
        let zoomer = Target::of(CreepType::Zoomer);
        assert_eq!(damage_mul(crate::special::NERFKA, &zoomer), DMG_STRONG); // fyzicke na rychle
        let boss = Target { boss: true, ..Default::default() };
        assert_eq!(damage_mul(crate::special::KATAPULT, &boss), DMG_WEAK);   // vybusne na bosse
        assert_eq!(damage_mul(crate::special::CHUCK_NORRIS, &karen), 1.0);   // cisty dmg vsem stejne
        let shield = Target { shielded: true, ..Default::default() };
        assert_eq!(damage_mul(crate::special::SNIPER, &shield), DMG_WEAK);   // pruzarne na stit
    }

    #[test]
    fn diamonds_are_needed_until_both_specials_are_rank_three() {
        assert_eq!(diamonds_needed(&[0, 0]), 6);
        assert_eq!(diamonds_needed(&[3, 1]), 2);
        assert_eq!(diamonds_needed(&[3, 3]), 0);
    }
}

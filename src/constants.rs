pub const COLS: usize = 7;
pub const ROWS: usize = 14;
pub const CELL: f32 = 52.0;
pub const LANE_W: f32 = COLS as f32 * CELL;
pub const LANE_H: f32 = ROWS as f32 * CELL;
pub const LANE_X: f32 = 0.0;
pub const LANE_Y: f32 = 0.0;
pub const FINISH_EXTRA_ROWS: usize = 2;
pub const SPAWN_ABOVE: f32 = 120.0; // o kolik nad horni okraj lajny se rodi creepy, odeslane jednotky a hlidac

// bezpecna oblast (systemove listy telefonu): nahore stav/cas/baterie, dole tlacitka zpet/home; na PC 0. Doladit podle telefonu.
pub const SAFE_TOP: f32 = 0.0;    // okraje systemovych list se ted ctou z Androidu (safe_area_system), tady zustava jen ruzna rezerva
pub const SAFE_BOTTOM: f32 = 0.0;
pub const BASE_INCOME: f32 = 20.0; // zakladni income (gold za tik 7 s) uz od zacatku hry
// gold za zabiti creepu automaticke vlny (poslane jednotky gold nedavaji)
pub const KILL_GOLD_NORMAL: i32 = 15;
pub const KILL_GOLD_ELITE: i32 = 40;
pub const KILL_GOLD_BOSS: i32 = 80;
pub const KILL_GOLD_MEGA: i32 = 150;
// zaloha pro Android, kdyz se okraje systemovych list nepodari precist (logicke px; z logu: stavova lista 128 px, navigacni 168 px pri hustote 3.75)
pub const FALLBACK_INSET_TOP: f32 = 34.0;
pub const FALLBACK_INSET_BOTTOM: f32 = 46.0;
// vychozi posun kamery, aby lajna sedela mezi horni listou hry (36 px) a spodni bezpecnou oblasti
pub const CAM_BASE_Y: f32 = (SAFE_TOP + 36.0 - SAFE_BOTTOM) / 2.0;

// 1v1: lajna bota lezi ve svete o tolik vpravo od hracovy. Je to dal nez nejvetsi dosah veze,
// takze strely, aury a leceni jedne strany nikdy nezasahnou na druhou.
pub const SIDE_OFFSET_X: f32 = 3000.0;
pub const START_GOLD: i32 = 250;  // staci na ctyri nejlevnejsi Basic veze (40+50+70+80 g)
pub const START_LIVES: u32 = 15;  // oba zacinaji s 15, dohromady 30 (pretahovana)
pub const TOTAL_LIVES: u32 = 30;

// === KOLA ===
pub const BUILD_PHASE_SECS: f32 = 15.0;  // prvni pauza na zacatku zapasu (stavba)
pub const ROUND_PAUSE_SECS: f32 = 7.0;   // pauza mezi koly
pub const ROUND_MAX_SECS: f32 = 60.0;    // kolo skonci nejpozdeji po tolika sekundach
pub const BOSS_EVERY: u32 = 5;           // kazde 5. kolo prijde i boss
pub const ELITE_HP_MUL: f32 = 3.0;       // vetsi creep ma tolikrat vic HP nez normalni
pub const BOSS_HP_MUL: f32 = 8.0;        // boss (kdyz vlna nema vlastni HP bosse) = HP normalniho x tolik
pub const LATE_ROUND: u32 = 30;          // po tomto kole HP roste navic rychleji (pojistka: zapas skonci kratce po 30. vlne)
pub const LATE_ROUND_HP_GROWTH: f32 = 1.15; // ...o 15 % za kazde dalsi kolo
// kolik zivotu stoji pruchod (souper je ziska)
pub const LEAK_NORMAL: u32 = 1;
pub const LEAK_ELITE: u32 = 2;
pub const LEAK_BOSS: u32 = 4;

pub const CREEP_BASE_SPEED: f32 = 85.0;
pub const MAX_TOWER_LEVEL: u32 = 7;   // max rank Basic veze (upgrade za gold)
pub const BASIC_TOWER_COUNT: usize = 15;
pub const DECK_BASIC_SIZE: usize = 4;
pub const DECK_SPECIAL_SIZE: usize = 2;
pub const DECK_SIZE: usize = DECK_BASIC_SIZE + DECK_SPECIAL_SIZE;
// vsechny veze, ze kterych se sklada balicek (druh, nazev)
pub const ALL_TOWERS: [(u8, &str); 25] = [
    (1, "BRAMBOROMET"),
    (2, "NERFKA"),
    (3, "BABICKA"),
    (4, "ROUTER"),
    (5, "KAKTUS"),
    (6, "TOUSTOVAC"),
    (7, "MUCHOMURKA"),
    (8, "MRAZAK"),
    (9, "TESLA"),
    (10, "SNIPER"),
    (11, "KATAPULT"),
    (12, "DJ"),
    (13, "PEPA"),
    (14, "KASICKA"),
    (15, "DANOVY URAD"),
    (16, "CHUCK NORRIS"),
    (17, "TCHYNE"),
    (18, "SHREK BAZINA"),
    (19, "GIGACHAD GYM"),
    (20, "DOGE MINER"),
    (21, "TROLLFACE"),
    (22, "HAXOR"),
    (23, "KAREN MANAZERKA"),
    (24, "CERNA DIRA DO BRNA"),
    (25, "ADMIN BANHAMMER"),
];

pub fn is_special_tower(kind: u8) -> bool {
    kind > BASIC_TOWER_COUNT as u8 && kind <= ALL_TOWERS.len() as u8
}

pub fn tower_rank_cap(kind: u8) -> u32 {
    if is_special_tower(kind) { 3 } else { MAX_TOWER_LEVEL }
}
// kratka zkratka na vez bez ikony (aby se barevne ctverecky daly rozeznat)
pub fn tower_tag(kind: u8) -> &'static str {
    ALL_TOWERS.iter().find(|t| t.0 == kind).map_or("?", |t| &t.1[..t.1.len().min(3)]) // "DJ" ma jen 2 znaky
}
// bot
pub const BOT_THINK_SECS: f32 = 0.8;  // jak casto bot premysli (mensi = rychlejsi = tezsi)
pub const SELL_SECONDS: f32 = 3.0;   // jak dlouho trva prodej veze (behem toho jeste strili)
pub const SELL_REFUND: f32 = 0.5;    // kolik z celkove investice (vsech staveb ve vezi) se vrati
pub const LEAK_GOLD_RATIO: f32 = 0.2; // kdyz tvoje jednotka projde: tolik z ceny se vrati jako gold
pub const TOWER_RANGE_SCALE: f32 = 0.5; // dosah vsech veze se nasobi timto cislem (1.0 = puvodni dosahy z tabulky)

pub const TOWER_POISON: u8 = 7;

// HP creepu roste s vlnou (velikost uz ne)
pub const CREEP_GROWTH_PER_WAVE: f32 = 0.07;
pub const CREEP_GROWTH_MAX_SCALE: f32 = 2.2;

pub const FROST_OUTLINE_MAX_EXTRA: f32 = 36.0;

// None = veze zatim nema ikonu (poison, sniper) -> kresli se barevny ctverecek
pub fn tower_icon_path(kind: u8) -> Option<&'static str> {
    let _ = kind;
    None
}

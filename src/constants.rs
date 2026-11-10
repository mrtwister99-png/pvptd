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
pub const BASE_INCOME: f32 = 10.0; // zakladni income (gold za tik 7 s) uz od zacatku hry
// zaloha pro Android, kdyz se okraje systemovych list nepodari precist (logicke px; z logu: stavova lista 128 px, navigacni 168 px pri hustote 3.75)
pub const FALLBACK_INSET_TOP: f32 = 34.0;
pub const FALLBACK_INSET_BOTTOM: f32 = 46.0;
// vychozi posun kamery, aby lajna sedela mezi horni listou hry (36 px) a spodni bezpecnou oblasti
pub const CAM_BASE_Y: f32 = (SAFE_TOP + 36.0 - SAFE_BOTTOM) / 2.0;

pub const CREEP_BASE_SPEED: f32 = 85.0;
pub const MAX_TOWER_LEVEL: u32 = 5;
pub const MAX_WAVES: u32 = 30;       // celkem vln, potom uz zadne nechodi
pub const SELL_SECONDS: f32 = 3.0;   // jak dlouho trva prodej veze (behem toho jeste strili)
pub const SELL_REFUND: f32 = 0.75;   // kolik z celkove investice se vrati
pub const LEAK_GOLD_RATIO: f32 = 0.2; // kdyz tvoje jednotka projde: tolik z ceny se vrati jako gold
pub const LEAK_LIFE_GAIN: u32 = 1;    // ...a tolik zivotu pribude
pub const TOWER_RANGE_SCALE: f32 = 0.5; // dosah vsech veze se nasobi timto cislem (1.0 = puvodni dosahy z tabulky)

pub const TOWER_POISON: u8 = 5;

// HP creepu roste s vlnou (velikost uz ne)
pub const CREEP_GROWTH_PER_WAVE: f32 = 0.07;
pub const CREEP_GROWTH_MAX_SCALE: f32 = 2.2;

pub const FROST_OUTLINE_MAX_EXTRA: f32 = 36.0;

// None = veze zatim nema ikonu (poison, sniper) -> kresli se barevny ctverecek
pub fn tower_icon_path(kind: u8) -> Option<&'static str> {
    match kind {
        1 => Some("icons/towers/arrow.png"),
        2 => Some("icons/towers/cannon.png"),
        3 => Some("icons/towers/frost.png"),
        4 => Some("icons/towers/fire.png"),
        _ => None,
    }
}

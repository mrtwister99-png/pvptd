// SPECIALNI VEZE (druhy 8-18): parametry jejich schopnosti podle ranku 1..7.
// Zakladni cisla (poskozeni, dosah, interval) jsou v tower_def.rs, tady jen to, co je navic.

pub const HARPOON: u8 = 8;     // strili jen na letajici a srazi je na zem
pub const GEYSER: u8 = 9;      // pulz: odhodi pozemni creepy v okoli zpet
pub const EXECUTIONER: u8 = 10; // % chybejiciho HP + poprava pod hranici (ne boss)
pub const VOODOO: u8 = 11;     // oznaceny creep sdili cast poskozeni s okolim
pub const CURSE: u8 = 12;      // oznaceny creep dostava vic poskozeni, vypne mu leceni a auru
pub const ILLUSION: u8 = 13;   // zmateny creep jde nahodnym smerem
pub const TWIN: u8 = 14;       // pri spojeni zustane na puvodnim miste nova vez R1
pub const MINT: u8 = 15;       // po kazdem kole gold
pub const BELL: u8 = 16;       // okolni veze strili rychleji a silneji
pub const SPIKES: u8 = 17;     // past na blizkeho pozemniho creepa
pub const BARRACKS: u8 = 18;   // poslane jednotky maji vic HP a rychlost

fn r(rank: u32) -> f32 { (rank.clamp(1, 7) - 1) as f32 } // 0..6

// veze, ktere vubec nestrili (jejich schopnost resi vlastni system)
pub fn shoots(kind: u8) -> bool { !matches!(kind, GEYSER | MINT | BELL | SPIKES | BARRACKS) }

// Executioner: kolik % chybejiciho HP prida k rane a pod kolik % HP creepa popravi (bosse nikdy)
pub fn executioner_missing_pct(rank: u32) -> f32 { 0.10 + 0.03 * r(rank) }
pub fn executioner_threshold(rank: u32) -> f32 { [0.0, 0.0, 0.10, 0.12, 0.15, 0.18, 0.22][(rank.clamp(1, 7) - 1) as usize] }

// Voodoo: kolik % poskozeni oznaceneho creepa dostanou creepi kolem a v jakem okruhu
pub fn voodoo_share(rank: u32) -> f32 { 0.30 + 0.05 * r(rank) }
pub const VOODOO_RADIUS: f32 = 90.0;
pub const VOODOO_SECS: f32 = 4.0;

// Curse: o kolik vic poskozeni creep dostava (0.15 = +15 %)
pub fn curse_mul(rank: u32) -> f32 { 0.15 + 0.04 * r(rank) }
pub const CURSE_SECS: f32 = 4.0;

// Illusion: max vzdalenost (svetove jednotky), kterou zmateny creep ujde nahodnym smerem
pub fn illusion_dist(rank: u32) -> f32 { 40.0 + 25.0 * r(rank) }

// Mint: gold na konci kazdeho kola
pub fn mint_gold(rank: u32) -> i32 { [5, 8, 12, 17, 23, 30, 40][(rank.clamp(1, 7) - 1) as usize] }

// Bell: bonus k rychlosti strelby a k poskozeni vezi v okruhu (vic Bell se nescita, plati nejsilnejsi)
pub fn bell_rate(rank: u32) -> f32 { 0.10 + 0.04 * r(rank) }
pub fn bell_dmg(rank: u32) -> f32 { 0.04 * r(rank) }

// Spikes: od ranku 5 past zpomali, na ranku 7 i otravi
pub fn spikes_slow(rank: u32) -> f32 { if rank >= 5 { 0.3 } else { 0.0 } }
pub fn spikes_poison(rank: u32) -> f32 { if rank >= 7 { 0.25 } else { 0.0 } } // jed = podil poskozeni pasti za sekundu

// Barracks: poslane jednotky (plati nejvyssi Barracks na poli)
pub fn barracks_hp(rank: u32) -> f32 { 0.10 * rank.clamp(1, 7) as f32 }
pub fn barracks_speed(rank: u32) -> f32 { 0.04 * rank.clamp(1, 7) as f32 }

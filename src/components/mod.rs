
use bevy::prelude::*;

// === STRANA ZAPASU (1v1) ===
// Kazda vez, creep a hlidac patri jedne strane. Creep stoji vzdy na lajne strany, ktere skodi.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side { Player, Bot }

impl Side {
    pub const ALL: [Side; 2] = [Side::Player, Side::Bot];
    pub fn idx(self) -> usize { match self { Side::Player => 0, Side::Bot => 1 } }
    pub fn other(self) -> Side { match self { Side::Player => Side::Bot, Side::Bot => Side::Player } }
    // stred lajny teto strany ve svete (osa X)
    pub fn lane_x(self) -> f32 { crate::constants::LANE_X + self.idx() as f32 * crate::constants::SIDE_OFFSET_X }
}

// === TOWER ===
#[derive(Component)]
pub struct Tower {
    pub side: Side,
    pub damage_min: f32,
    pub damage_max: f32,
    pub range: f32,
    pub fire_rate: f32,
    pub cooldown: f32,
    pub kind: u8,
    pub col: usize,
    pub row: usize,
    pub level: u32,
    pub splash: f32,
    pub slow_per_hit: f32,
    pub slow_max: f32,
    pub push_chance: f32,
    pub push_dist: f32,
    pub dot_dps: f32,
    pub dot_duration: f32,
    pub chain_targets: u32,
    pub hit_count: u32,
    pub last_ability_wave: u32,
    pub dj_drop_timer: f32,
    pub invested: i32, // kolik goldu stala stavba + vsechny upgrady (pro prodej)
}

// === CREEP - vzdy kolecko, jina barva, rust per wave + HP bar ===
#[derive(Component)]
pub struct Creep {
    pub is_boss: bool,
    pub is_mega: bool,
    pub max_hp_reductions: u8,
    pub base_radius: f32,
    pub speed_mul: f32, // nasobek rychlosti chuze (1.0 = bezna)
    pub leak_cost: u32, // kolik zivotu stoji, kdyz projde (GDD: maly 1, velky 2, boss 3; poslane jednotky podle tabulky)
    pub kill_gold: i32, // gold za zabiti (GDD 4.1: 2-4 podle typu, boss 150, poslane jednotky 0)
}

// === CREEPOVE PODLE GDD (data v wave_def.rs) ===
// co creep ma proti damage typum (silny / slaby zasah); stit a schopnosti bossu ho meni za behu
#[derive(Component, Clone, Copy)] pub struct DmgTarget(pub crate::wave_def::Target);
// stit (AdBlock Rytir, Mama ve 2. fazi): pohlti damage, dokud nedojde; prev = HP v minulem snimku
#[derive(Component)] pub struct Shield { pub hp: f32, pub prev: f32 }
// regenerace (Wi-Fi Upir): kdyz `idle` sekund nedostal zasah, leci se
#[derive(Component)] pub struct Regen { pub idle: f32, pub prev: f32 }
// po smrti se rozpadne na kopie (Kopirovaci Bot)
#[derive(Component)] pub struct CopyOnDeath;
// aura rychlosti (Cringe Influencer, Sef z Prace): okolni creepove jdou rychleji
#[derive(Component)] pub struct SpeedAura { pub bonus: f32, pub radius: f32 }
// bonus rychlosti, ktery creep prave ma od aury (pocita speed_aura_system)
#[derive(Component, Default)] pub struct AuraSpeed(pub f32);
// nahodna rychlost (Lagujici Brambora): kazde 3 s nova
#[derive(Component)] pub struct RandomSpeed { pub timer: Timer, pub mul: f32 }
// boss podle GDD 11: casovace jeho schopnosti
#[derive(Component)] pub struct Boss { pub kind: crate::wave_def::BossKind, pub a: Timer, pub b: Timer, pub phase: u8, pub guard_left: f32 }

#[derive(Component)] pub struct CreepMainCircle;
#[derive(Component)] pub struct Health(pub f32);
#[derive(Component)] pub struct MaxHealth(pub f32);

#[derive(Component, Default)]
pub struct Bullet {
    pub active: bool, // false = volny slot v poolu strel
    pub damage: f32,
    pub shot_count: u32,
    pub owner_side: Option<Side>,
    pub owner_row: u8,
    pub owner_col: u8,
    pub wave: u32,
    pub vel: Vec2,
    pub life: Timer,
    pub aoe: f32,
    pub kind: u8,
    pub level: u32,
    pub slow: f32,
    pub slow_max: f32,
    pub push_chance: f32,
    pub push_dist: f32,
    pub chain_left: u32,
    pub dot_dps: f32,
    pub dot_dur: f32,
    pub hit_radius: f32,         // jak blizko creepa musi strela byt, aby zasahla
    pub hits_air: bool,          // trefi strela i letajici jednotky
    pub chain_radius: f32,       // jak daleko muze chain preskocit na dalsiho creepa
    pub chain_falloff: f32,      // nasobek damage za kazdy preskok (0.75 = o 25 % mene)
    pub push_speed: f32,         // rychlost odhozeni (jednotek/s)
}

#[derive(Component)] pub struct SlowDebuff { pub timer: Timer, pub factor: f32 }
#[derive(Component)] pub struct BurnDebuff { pub timer: Timer, pub tick: Timer, pub dps: f32, pub level: u32, pub kind: u8 } // level a druh veze (efekty, damage typ)
#[derive(Component)] pub struct StunDebuff { pub timer: Timer }
#[derive(Component)] pub struct PoisonDebuff { pub timer: Timer, pub tick: Timer, pub dps: f32, pub level: u32, pub stacks: u8 }
#[derive(Component)] pub struct DeathExplosion { pub damage: f32, pub radius: f32 }
#[derive(Component)] pub struct DamageNumber { pub timer: Timer, pub vel: Vec2, pub active: bool }

// UI
#[derive(Component)] pub struct GoldText;
#[derive(Component)] pub struct DiamondText;
#[derive(Component)] pub struct WaveText;
#[derive(Component)] pub struct WaveTimerText;
#[derive(Component)] pub struct PauseButtonUI;
#[derive(Component)] pub struct PauseMenuRoot;
#[derive(Component)] pub struct PauseOptionsRoot;
#[derive(Component)] pub struct PauseConfirmRoot;
#[derive(Component)] pub struct PauseConfirmText;
#[derive(Component)] pub struct PauseMenuButton(pub crate::menu::PauseAction);
#[derive(Component)] pub struct PauseResumeButton;
#[derive(Component)] pub struct PauseConfirmYesButton;
#[derive(Component)] pub struct PauseConfirmNoButton;
#[derive(Component)] pub struct HoverHighlight;
#[derive(Component)] pub struct GridLine;
#[derive(Component)] pub struct CreepHpText;
#[derive(Component)] pub struct CreepStatusText;
#[derive(Component)] pub struct SlowOutline;
#[derive(Component)] pub struct TowerLevelText;

// HP bar
#[derive(Component)] pub struct HpBarBg;
#[derive(Component)] pub struct HpBarFill;
#[derive(Component)] pub struct HpBarText;

// S panel a tlacitka POSTAVIT (UpgradeButton) / PRODAT
#[derive(Component)] pub struct SendToggleButton;
#[derive(Component)] pub struct SendPanel;
#[derive(Component)] pub struct TowerActionPanel;
#[derive(Component)] pub struct UpgradeButton;
#[derive(Component)] pub struct SellButton;

// odesilani jednotek (panel S) + income
#[derive(Component)] pub struct SendPanelItem(pub u8);
#[derive(Component)] pub struct SentUnit;
#[derive(Component)] pub struct IncomeText;
#[derive(Component)] pub struct BuildTowerPanel;
#[derive(Component)] pub struct BuildTowerButton(pub u8);

// UPGRADE / PRODAT: popisky tlacitek a probihajici prodej veze
#[derive(Component)] pub struct UpgradeLabel;
#[derive(Component)] pub struct SellLabel;
#[derive(Component)] pub struct Selling { pub timer: Timer }

// kruhy dosahu veze (aktualni a po upgradu)
#[derive(Component)] pub struct RangeRing;
#[derive(Component)] pub struct RangeRingNext;

// odhozeni creepa dozadu (smerem na start) o `remaining` jednotek, rychlosti `speed` (jednotek/s)
#[derive(Component)] pub struct Knockback { pub remaining: f32, pub speed: f32 }

// letajici jednotka: leti rovne pres bludiste a trefi ji jen veze, ktere umi strilet do vzduchu
#[derive(Component)] pub struct Air;

// enrage: po poklesu HP pod hp_frac * max HP se jednou zvysi rychlost a velikost (Chad Kuryr, Sef z Prace)
#[derive(Component)] pub struct Enrage { pub hp_frac: f32, pub speed_bonus: f32, pub size_bonus: f32, pub flash: Color }

// cervene probliknuti pri spusteni enrage (child kolecko, po case se smaze)
#[derive(Component)] pub struct EnrageFlash { pub timer: Timer }

// odolnosti (bossove): stun_immune = nejde omracit, push = podil odhozeni, ktery se ignoruje (0.5 = 50 %)
#[derive(Component, Clone, Copy)] pub struct Resist { pub stun_immune: bool, pub push: f32, pub slow: f32 } // slow = podil sily zpomaleni, ktery se ignoruje (0.2 = 20 %)


// tlacitko vpravo nahore: prepnout kameru na lajnu soupere a zpet
#[derive(Component)] pub struct SwitchViewButton;
#[derive(Component)] pub struct SwitchViewLabel;
#[derive(Component)] pub struct LivesText;
// panel se staty vybrane veze (nad tlacitky dole uprostred)
#[derive(Component)] pub struct TowerInfoPanel;
#[derive(Component)] pub struct TowerInfoText;
// okno potvrzeni prodeje (ANO = Enter, NE = Esc)
#[derive(Component)] pub struct SellConfirmRoot;
#[derive(Component)] pub struct SellConfirmText;
#[derive(Component)] pub struct SellYesButton;
#[derive(Component)] pub struct SellNoButton;

// === znacky od specialnich vezi ===
// Curse: creep dostava o `mul` vic poskozeni a nefunguje mu leceni ani aura
#[derive(Component)] pub struct Cursed { pub mul: f32, pub timer: Timer }
// Illusion: creep jde smerem `dir`, dokud neujde `remaining` (nebo nenarazi)
#[derive(Component)] pub struct Confused { pub dir: Vec2, pub remaining: f32 }

// obrazovka konce zapasu (VYHRA / PROHRA) a tlacitko zpet do menu
#[derive(Component)] pub struct GameOverRoot;
#[derive(Component)] pub struct GameOverMenuButton;

// lecici jednotka (Priest): kazdych 0.5 s doplni HP creepum v radiu (data z wave_def::heal_for)
#[derive(Component)] pub struct Heal { pub radius: f32, pub hps: f32, pub tick: Timer }

// aura Captaina: creepum v radiu zvysi max HP (data z wave_def::aura_for); AuraBuff = jednotka bonus prave ma (delta = o kolik zvysil max HP)
#[derive(Component)] pub struct Aura { pub radius: f32, pub hp_bonus: f32 }
#[derive(Component)] pub struct AuraBuff { pub delta: f32 }

// zpetna strelba (Frost Archer): v dosahu zpomali nejblizsi vez (data z wave_def::shoot_back_for)
#[derive(Component)] pub struct ShootBack { pub range: f32, pub slow: f32, pub secs: f32, pub tick: Timer }

// vez zpomalena strelou jednotky: fire_rate se nasobi factor, dokud bezi timer
#[derive(Component)] pub struct TowerSlow { pub timer: Timer, pub factor: f32 }

// znazorneni zmrazeni veze (ledovy ctverec + kruh); child veze, maze se, kdyz TowerSlow vyprsi
#[derive(Component)] pub struct FrostOverlay;

// pulzujici kruh okolo tela (vizual odolnosti, data z wave_def::shimmer_for); scale se meni podle base_r a period
#[derive(Component)] pub struct Shimmer { pub base_r: f32, pub period: f32 }

// hlaska Hlidace Pepy / bosse dole na obrazovce (Visibility ji zobrazuje/skryva, timer = jak dlouho zustane videt)
#[derive(Component)] pub struct GuardianMsg { pub timer: Timer }
#[derive(Component)] pub struct GuardianMsgText;

// zvyraznene vybrane policko (zluty ramecek), jede za SelectedCell
#[derive(Component)] pub struct SelectionMarker;

// koren UI: vsechny hlavni uzly jsou jeho deti, jeho okraje se posouvaji podle systemovych list
#[derive(Component)] pub struct SafeRoot;

// testovaci tlacitko vlevo nahore pod income: kazde stisknuti prida +50 do income
#[derive(Component)] pub struct CheatButton;
#[derive(Component)] pub struct TestWaveButton; // TEST rezim: poslat dalsi vlnu na vlastni lajnu

// aura creepu na VEZE v radiu (data z wave_def::tower_aura_for): upravuje strelbu vezi, stomp je periodicke omraceni
#[derive(Component)]
pub struct TowerAura { pub radius: f32, pub random_target: f32, pub dmg_mul: f32, pub range_mul: f32, pub rate_mul: f32, pub stomp_every: f32, pub stomp_stun: f32, pub stomp_single: bool, pub timer: Timer } // stomp_single = omraci jen 1 nahodnou vez (Danovy Urad, Mama)

// upravy veze od aur creepu (pocita tower_aura_system kazdy snimek, cte tower_shoot)
#[derive(Component)]
pub struct TowerMods { pub dmg_mul: f32, pub range_mul: f32, pub rate_mul: f32, pub random_target: f32, pub stun_left: f32 }
impl Default for TowerMods {
    fn default() -> Self { Self { dmg_mul: 1.0, range_mul: 1.0, rate_mul: 1.0, random_target: 0.0, stun_left: 0.0 } }
}
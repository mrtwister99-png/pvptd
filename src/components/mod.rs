
use bevy::prelude::*;

// === TOWER ===
#[derive(Component)]
pub struct Tower {
    pub damage: f32,
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
}

// === CREEP - vzdy kolecko, jina barva, rust per wave + HP bar ===
#[derive(Component)]
pub struct Creep {
    pub is_boss: bool,
    pub is_mega: bool,
    pub base_radius: f32,
    pub speed_mul: f32, // nasobek rychlosti chuze (1.0 = bezna)
}

#[derive(Component)] pub struct CreepMainCircle;
#[derive(Component)] pub struct Health(pub f32);
#[derive(Component)] pub struct MaxHealth(pub f32);

#[derive(Component, Default)]
pub struct Bullet {
    pub active: bool, // false = volny slot v poolu strel
    pub damage: f32,
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
#[derive(Component)] pub struct BurnDebuff { pub timer: Timer, pub tick: Timer, pub dps: f32, pub level: u32 } // level veze (pro efekty)
#[derive(Component)] pub struct StunDebuff { pub timer: Timer }
#[derive(Component)] pub struct PoisonDebuff { pub timer: Timer, pub tick: Timer, pub dps: f32, pub level: u32 }
#[derive(Component)] pub struct DamageNumber { pub timer: Timer, pub vel: Vec2, pub active: bool }

// UI
#[derive(Component)] pub struct GoldText;
#[derive(Component)] pub struct DiamondText;
#[derive(Component)] pub struct EscapedText;
#[derive(Component)] pub struct WaveText;
#[derive(Component)] pub struct WaveTimerText;
#[derive(Component)] pub struct PauseButtonUI;
#[derive(Component)] pub struct BuildPreview;
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

// B / S panely a upgrade / prodat
#[derive(Component)] pub struct BuildToggleButton;
#[derive(Component)] pub struct BuildPanel;
#[derive(Component)] pub struct BuildPanelItem(pub u8);
#[derive(Component)] pub struct SendToggleButton;
#[derive(Component)] pub struct SendPanel;
#[derive(Component)] pub struct TowerActionPanel;
#[derive(Component)] pub struct UpgradeButton;
#[derive(Component)] pub struct SellButton;

// odesilani jednotek (panel S) + income
#[derive(Component)] pub struct SendPanelItem(pub u8);
#[derive(Component)] pub struct SentUnit { pub cost: i32 } // cena jednotky (pro bonus za projiti)
#[derive(Component)] pub struct IncomeText;

// UPGRADE / PRODAT: popisky tlacitek a probihajici prodej veze
#[derive(Component)] pub struct UpgradeLabel;
#[derive(Component)] pub struct SellLabel;
#[derive(Component)] pub struct Selling { pub timer: Timer }

// kruhy dosahu veze (aktualni a po upgradu) a rezervovane tlacitko vpravo nahore
#[derive(Component)] pub struct RangeRing;
#[derive(Component)] pub struct RangeRingNext;
#[derive(Component)] pub struct ReservedButton;

// odhozeni creepa dozadu (smerem na start) o `remaining` jednotek, rychlosti `speed` (jednotek/s)
#[derive(Component)] pub struct Knockback { pub remaining: f32, pub speed: f32 }

// letajici jednotka: leti rovne pres bludiste a trefi ji jen veze, ktere umi strilet do vzduchu
#[derive(Component)] pub struct Air;

// enrage: po poklesu HP pod hp_frac * max HP se jednou zvysi rychlost a velikost (data z wave_def::enrage_for)
#[derive(Component)] pub struct Enrage { pub hp_frac: f32, pub speed_bonus: f32, pub size_bonus: f32, pub flash: Color }

// cervene probliknuti pri spusteni enrage (child kolecko, po case se smaze)
#[derive(Component)] pub struct EnrageFlash { pub timer: Timer }

// odolnosti jednotky (data z wave_def::resist_for): stun_immune = nejde omracit, push = podil odhozeni, ktery se ignoruje (0.5 = 50 %)
#[derive(Component)] pub struct Resist { pub stun_immune: bool, pub push: f32, pub slow: f32, pub dmg_mul: [f32; 8] } // slow = podil sily zpomaleni, ktery se ignoruje (0.2 = 20 %); dmg_mul[druh veze] = nasobek damage (1.0 = beze zmeny)

// hlidac: nesmrtelna jednotka (bez Creep = veze ani strely ji nevidi), pred vlnou probehne lajnu a niCi veze, ktere blokuji cestu
#[derive(Component)] pub struct Guardian;

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

// hlaska hlidace dole na obrazovce (Visibility ji zobrazuje/skryva, timer = jak dlouho zustane videt)
#[derive(Component)] pub struct GuardianMsg { pub timer: Timer }

// zvyraznene vybrane policko (zluty ramecek), jede za SelectedCell
#[derive(Component)] pub struct SelectionMarker;

// koren UI: vsechny hlavni uzly jsou jeho deti, jeho okraje se posouvaji podle systemovych list
#[derive(Component)] pub struct SafeRoot;

// testovaci tlacitko vlevo nahore pod income: kazde stisknuti prida +50 do income
#[derive(Component)] pub struct CheatButton;

// aura creepu na VEZE v radiu (data z wave_def::tower_aura_for): upravuje strelbu vezi, stomp je periodicke omraceni
#[derive(Component)]
pub struct TowerAura { pub radius: f32, pub random_target: f32, pub dmg_mul: f32, pub range_mul: f32, pub rate_mul: f32, pub stomp_every: f32, pub stomp_stun: f32, pub timer: Timer }

// upravy veze od aur creepu (pocita tower_aura_system kazdy snimek, cte tower_shoot)
#[derive(Component)]
pub struct TowerMods { pub dmg_mul: f32, pub range_mul: f32, pub rate_mul: f32, pub random_target: f32, pub stun_left: f32 }
impl Default for TowerMods {
    fn default() -> Self { Self { dmg_mul: 1.0, range_mul: 1.0, rate_mul: 1.0, random_target: 0.0, stun_left: 0.0 } }
}
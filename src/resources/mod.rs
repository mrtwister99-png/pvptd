use bevy::prelude::*;
use crate::constants::{COLS, ROWS};

// faze zapasu: pauza mezi koly, nebo prave bezi kolo
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoundPhase { Pause, Running }

#[derive(Resource)]
pub struct RoundState {
    pub phase: RoundPhase,
    pub timer: Timer,        // odpocet pauzy
    pub elapsed: f32,        // jak dlouho uz bezi kolo
}

impl Default for RoundState {
    fn default() -> Self {
        Self { phase: RoundPhase::Pause, timer: Timer::from_seconds(crate::constants::BUILD_PHASE_SECS, TimerMode::Once), elapsed: 0.0 }
    }
}

// rezim zapasu vybrany v menu po HRAT: proti botovi, nebo TEST (jen vlastni lajna, neomezene penize,
// vsechny veze, posilani creepu kdykoli na sebe, zivoty se nepocitaji)
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameMode { #[default] VsBot, Test }

impl GameMode {
    pub fn is_test(self) -> bool { self == GameMode::Test }
}

// run podminky pro systemy, ktere bezi jen v jednom rezimu
pub fn in_test_mode(mode: Res<GameMode>) -> bool { mode.is_test() }
pub fn not_test_mode(mode: Res<GameMode>) -> bool { !mode.is_test() }

// v testu se gold i diamanty pořád doplňují na tuhle hodnotu
pub const TEST_GOLD: i32 = 1_000_000;
pub const TEST_DIAMONDS: i32 = 999;

// statistiky zapasu podle GDD: limity posilani v teto vlne, pocty leaku (tie-break po vlne 30),
// cas zabiti posledniho bosse a sudden death po vlne 30 (dalsi leak = konec)
#[derive(Resource, Default, Clone)]
pub struct MatchStats {
    pub sends: [u32; 2],          // kolik jednotek strana v teto vlne poslala (max 12)
    pub mini_bosses: [u32; 2],    // kolik Mini-Bossu (max 2)
    pub leaks: [u32; 2],          // kolik creepu proslo na lajne strany za cely zapas
    pub last_boss_kill: [Option<f32>; 2], // kdy strana zabila bosse 30. vlny (driv = lepsi)
    pub sudden_death: bool,
}

// konec zapasu: Some(vitez), jakmile nekdo prijde o vsechny zivoty
#[derive(Resource, Default)]
pub struct MatchOver(pub Option<crate::components::Side>);
#[derive(Resource)]
pub struct WavePaused(pub bool);
#[derive(Resource)]
pub struct WaveCount(pub u32);
// income: kazdych 7s pripise `amount` gold (zvysuje se odesilanim jednotek)
pub struct Income { pub amount: f32, pub timer: Timer } // amount = base + sendy (vyplaci se max 40 za tick, GDD 4.1)

// stav jedne strany (hrac / bot): vlastni gold, income, zivoty a bludiste
pub struct SideState {
    pub gold: i32,
    pub diamonds: i32,
    pub gold_boost_timer: f32,
    pub doge_moon_used: bool,
    pub income: Income,
    pub lives: u32,
    pub grid: Grid,
    pub path: PathMap,
    pub builds: u32,   // kolik vezi uz postavil (cena dalsi stavby roste)
    pub deck: Vec<u8>, // balicek vezi (druhy), ze kterych stavba nahodne vybira
}

impl SideState {
    pub fn new() -> Self {
        Self {
            gold: crate::constants::START_GOLD,
            diamonds: 0,
            gold_boost_timer: 0.0,
            doge_moon_used: false,
            income: Income { amount: crate::wave_def::BASE_INCOME as f32, timer: Timer::from_seconds(crate::wave_def::INCOME_TICK_SECS, TimerMode::Repeating) },
            lives: crate::constants::START_LIVES,
            grid: Grid { blocked: [[false; COLS]; ROWS] },
            path: PathMap { dist: [[None; COLS]; ROWS] },
            builds: 0,
            deck: vec![1, 2, 3, 4, 16, 17],
        }
    }
}

// obe strany zapasu; index = Side::idx() (0 = hrac, 1 = bot)
#[derive(Resource)]
pub struct Sides(pub [SideState; 2]);

impl Default for Sides {
    fn default() -> Self { Self([SideState::new(), SideState::new()]) }
}

impl Sides {
    pub fn get(&self, side: crate::components::Side) -> &SideState { &self.0[side.idx()] }
    pub fn get_mut(&mut self, side: crate::components::Side) -> &mut SideState { &mut self.0[side.idx()] }
}

// balicek hrace vybrany v menu (BALICEK); pri startu zapasu se zkopiruje do SideState
#[derive(Resource)]
pub struct Deck(pub Vec<u8>);
impl Default for Deck {
    fn default() -> Self { Self(vec![1, 2, 3, 4, 16, 17]) }
}

// ovladani klavesnici: Enter na prazdnem policku postavi, Enter na vezi ji vylepsi, S na vezi = prodat (okno ANO/NE)
#[derive(Resource, Default)]
pub struct KeyControl { pub sell_confirm: Option<(usize, usize)> } // sell_confirm = otevrene okno ANO/NE pro prodej

// na kterou lajnu se prave diva kamera
#[derive(Resource)]
pub struct ViewSide(pub crate::components::Side);
#[derive(Resource)]
pub struct HoverCell(pub Option<(usize, usize)>);
#[derive(Resource)]
pub struct SelectedCell(pub Option<(usize, usize)>);

// budova drzena na kurzoru (None = nic nedrzim)
// jeden ukazatel pro mys i dotyk: pozice (logicke pixely okna) a "prave stisknuto"
#[derive(Resource, Default)]
pub struct InputPointer { pub pos: Option<Vec2>, pub pressed: bool }

#[derive(Resource, Default)]
pub struct HeldBuilding(pub Option<u8>);

// ktery vysouvaci panel je otevreny (B = budovy, S = jednotky)
#[derive(Resource, Default)]
pub struct PanelsOpen { pub build: bool, pub send: bool, pub build_target: Option<(usize, usize)> } // build_target = policko, na ktere se stavi z nabidky B (None = budova se drzi na kurzoru)

#[derive(Clone)]
pub struct Grid { pub blocked: [[bool; COLS]; ROWS] }

#[derive(Clone)]
pub struct PathMap { pub dist: [[Option<i32>; COLS]; ROWS] }


#[derive(Resource, Default)]
pub struct DamageNumberPool { pub free: Vec<Entity> }

#[derive(Resource, Default)]
pub struct BulletPool { pub free: Vec<Entity> }

// Sdilene assety creepu - mesh/material se nevytvari pro kazdeho creepa znovu
#[derive(Resource)]
pub struct CreepAssets {
    pub unit_circle: Handle<Mesh>,                                 // kruh o polomeru 1 - velikost se dela pres scale
    pub slow_mat: Handle<ColorMaterial>,
    pub shadow_mesh: Handle<Mesh>,                                 // vyplneny kruh o polomeru 1 (stin letajicich jednotek, velikost pres scale)
}

// okraje systemovych list (logicke px): nahore stav/cas/baterie, dole zpet/home; plni safe_area_system
#[derive(Resource, Default, Clone, Copy, PartialEq)]
pub struct SafeInsets { pub left: f32, pub top: f32, pub right: f32, pub bottom: f32 }

pub const GRID_CELL: f32 = 64.0; // velikost bunky prostorove mrizky (svetove jednotky)

// Prostorova mrizka creepu - prebuduje se jednou za snimek, bullet_hit pak hleda jen v okoli strely
#[derive(Resource, Default)]
pub struct CreepGrid { cells: std::collections::HashMap<(i32, i32), Vec<(Entity, Vec2)>> }

impl CreepGrid {
    fn key(p: Vec2) -> (i32, i32) { ((p.x / GRID_CELL).floor() as i32, (p.y / GRID_CELL).floor() as i32) }
    pub fn clear(&mut self) { for v in self.cells.values_mut() { v.clear(); } } // klice zustavaji, vektory se znovupouziji
    pub fn insert(&mut self, e: Entity, p: Vec2) { self.cells.entry(Self::key(p)).or_default().push((e, p)); }
    // vsichni creepi ve vzdalenosti < r od pos (zapise do out)
    pub fn query(&self, pos: Vec2, r: f32, out: &mut Vec<(Entity, Vec2)>) {
        let (x0, y0) = Self::key(pos - Vec2::splat(r));
        let (x1, y1) = Self::key(pos + Vec2::splat(r));
        for cx in x0..=x1 {
            for cy in y0..=y1 {
                if let Some(v) = self.cells.get(&(cx, cy)) {
                    for &(e, p) in v { if p.distance(pos) < r { out.push((e, p)); } }
                }
            }
        }
    }
}
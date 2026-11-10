use bevy::prelude::*;
use crate::constants::{COLS, ROWS};

#[derive(Resource)]
pub struct CreepTimer(pub Timer);
#[derive(Resource)]
pub struct WavePaused(pub bool);
#[derive(Resource)]
pub struct EscapedCount(pub u32);
#[derive(Resource)]
pub struct Gold(pub i32);
#[derive(Resource)]
pub struct Diamonds(pub i32);
#[derive(Resource)]
pub struct WaveCount(pub u32);
// income: kazdych 7s pripise `amount` gold (zvysuje se odesilanim jednotek)
#[derive(Resource)]
pub struct Income { pub amount: f32, pub carry: f32, pub sent: [u32; 32], pub timer: Timer } // amount = gold za tik, carry = zbytek za desetinnou carkou, sent = kolik jednotek uz odeslano
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

#[derive(Resource, Clone)]
pub struct Grid { pub blocked: [[bool; COLS]; ROWS] }

#[derive(Resource, Clone)]
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
    pub shadow_mat: Handle<ColorMaterial>,                         // pruhledna cerna
    pub main_meshes: std::collections::HashMap<u32, Handle<Mesh>>, // cache kruhu hlavniho tela podle polomeru (x10)
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
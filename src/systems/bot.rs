// BOT (souper v 1v1): hraje se stejnymi pravidly jako hrac.
// Kazdych BOT_THINK_SECS: 1) spoji dvojici stejnych vezi, 2) postavi vez do sveho bludiste, 3) posle jednotku.
use bevy::prelude::*;
use rand::Rng;
use crate::components::{Side, Tower, Selling};
use crate::constants::{BOT_THINK_SECS, COLS, ROWS};
use crate::resources::{CreepAssets, MatchOver, RoundPhase, RoundState, Sides, WavePaused};
use crate::systems::creep::spawn_sent_unit;
use crate::systems::tower_build::{build_cost, try_build_random, try_merge, TowerRef};
use crate::utils::compute_path;
use crate::wave_def::{get_unit_def, income_multiplier, UNIT_COUNT};

// Plan bludiste bota: "had" ze zdi pres celou sirku s jednou mezerou, mezery se strida vlevo/vpravo.
// Radky 0-1 dole jsou cil, 13 nahore vstup. Poradi = poradi stavby.
fn bot_plan() -> Vec<(usize, usize)> {
    let mut cells = Vec::new();
    // prvni vrstva zdi (radek, sloupec mezery)
    let walls = [(11usize, COLS - 1), (8, 0), (5, COLS - 1), (2, 0)];
    for &(row, gap) in walls.iter() {
        for c in 0..COLS { if c != gap { cells.push((row, c)); } }
    }
    // druha vrstva hned pod prvni (stejna mezera -> cesta zustava otevrena), vic vezi kolem cesty
    for &(row, gap) in walls.iter().take(3) {
        for c in 0..COLS { if c != gap { cells.push((row - 1, c)); } }
    }
    cells
}

// zustane cesta z horni rady do cile otevrena, kdyz se policko zastavi? (bot si bludiste nikdy nezazdi)
fn keeps_path_open(blocked: &[[bool; COLS]; ROWS], row: usize, col: usize) -> bool {
    let mut b = *blocked;
    b[row][col] = true;
    let dist = compute_path(&b);
    (0..COLS).any(|c| !b[ROWS - 1][c] && dist[ROWS - 1][c].is_some())
}

pub fn bot_system(
    mut commands: Commands,
    time: Res<Time>,
    paused: Res<WavePaused>,
    over: Res<MatchOver>,
    round: Res<RoundState>,
    mut sides: ResMut<Sides>,
    asset_server: Res<AssetServer>,
    assets: Option<Res<CreepAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut think: Local<f32>,
    towers: Query<(Entity, &Tower), Without<Selling>>,
) {
    if paused.0 || over.0.is_some() { return; }
    *think += time.delta_secs();
    if *think < BOT_THINK_SECS { return; }
    *think = 0.0;
    let me = Side::Bot;

    // 1) spojeni: dvojice stejneho druhu a ranku (nejnizsi rank prvni). Uvolnene policko bot pozdeji zastavi znovu.
    let mine: Vec<TowerRef> = towers.iter().filter(|(_, t)| t.side == me).map(|(e, t)| TowerRef::of(e, t)).collect();
    let mut pair: Option<(TowerRef, TowerRef)> = None;
    for (i, a) in mine.iter().enumerate() {
        for b in mine.iter().skip(i + 1) {
            if a.kind == b.kind && a.rank == b.rank && a.rank < crate::constants::MAX_TOWER_LEVEL && pair.map_or(true, |(p, _)| a.rank < p.rank) {
                // nova vez zustane na policku vys v bludisti (blize vstupu = dele strili), uvolni se to nizsi
                pair = Some(if a.row >= b.row { (*b, *a) } else { (*a, *b) });
            }
        }
    }
    if let Some((from, to)) = pair {
        if try_merge(&mut commands, &asset_server, &mut sides, me, from, to) { return; }
    }

    // 2) stavba: dalsi volne policko z planu, ktere nezazdi cestu
    // (behem kola obcas stavbu vynecha a radsi posle jednotku = investice do income)
    let invest = round.phase == RoundPhase::Running && rand::thread_rng().gen_bool(0.3);
    let st = sides.get(me);
    let cost = build_cost(st.builds);
    if st.gold >= cost && !invest {
        let occupied: Vec<(usize, usize)> = mine.iter().map(|t| (t.row, t.col)).collect();
        let next = bot_plan().into_iter().find(|&(r, c)| !st.grid.blocked[r][c] && !occupied.contains(&(r, c)) && keeps_path_open(&st.grid.blocked, r, c));
        if let Some((r, c)) = next {
            if try_build_random(me, r, c, &mut commands, &mut sides, &asset_server) { return; }
        }
    }

    // 3) posilani: jen behem kola a jen z goldu navic (aby mel na dalsi stavbu); posle nejdrazsi jednotku do tretiny goldu
    if round.phase != RoundPhase::Running { return; }
    let Some(assets) = assets else { return; };
    let st = sides.get_mut(me);
    // posila z goldu, ktery mu zbyl po stavbe, nebo kdyz se rozhodl investovat (pak az do poloviny goldu)
    let budget = if invest { st.gold / 2 } else { st.gold - build_cost(st.builds) };
    if budget <= 0 { return; }
    let pick = (1..=UNIT_COUNT).filter(|&k| get_unit_def(k).cost <= budget).max_by_key(|&k| get_unit_def(k).cost);
    let Some(kind) = pick else { return; };
    let def = get_unit_def(kind);
    st.gold -= def.cost;
    let idx = (kind as usize).min(31);
    st.income.amount += def.income as f32 * income_multiplier(st.income.sent[idx]);
    st.income.sent[idx] += 1;
    spawn_sent_unit(&mut commands, &assets, &mut meshes, &mut materials, &asset_server, me.other(), kind);
    println!("Bot poslal {} za {}G (income {:.1})", def.name, def.cost, st.income.amount);
}

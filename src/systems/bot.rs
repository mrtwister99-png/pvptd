// BOT (souper v 1v1): hraje se stejnymi pravidly jako hrac.
// Kazdych BOT_THINK_SECS: 1) spoji dvojici stejnych vezi, 2) postavi vez do sveho bludiste, 3) posle jednotku.
use bevy::prelude::*;
use rand::Rng;
use crate::components::{Side, Tower, Selling};
use crate::constants::{BOT_THINK_SECS, COLS, ROWS};
use crate::resources::{CreepAssets, MatchOver, RoundPhase, RoundState, Sides, WavePaused};
use crate::systems::creep::{spawn_sent_unit, barracks_buff};
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
    mut status: Local<u32>, // pocitadlo pro vypis stavu do logu
    wave: Res<crate::resources::WaveCount>,
    towers: Query<(Entity, &Tower), Without<Selling>>,
    all_towers: Query<&Tower>,
) {
    if paused.0 || over.0.is_some() { return; }
    *think += time.delta_secs();
    if *think < BOT_THINK_SECS { return; }
    *think = 0.0;
    // za koho bot hraje: vzdy za Side::Bot; s AUTOPLAY_BOTH=1 i za hrace (test bot proti botovi)
    let ai_sides: &[Side] = if std::env::var("AUTOPLAY_BOTH").is_ok() { &[Side::Bot, Side::Player] } else { &[Side::Bot] };
    for &me in ai_sides {
        let mine: Vec<TowerRef> = towers.iter().filter(|(_, t)| t.side == me).map(|(e, t)| TowerRef::of(e, t)).collect();
        // kolik vezi chce bot mit: na zacatku 4, kazde kolo o 2 vic (max cele bludiste)
        let want = (4 + 2 * wave.0 as usize).min(40);
        if me == Side::Bot { *status += 1; }
        if me == Side::Bot && *status % 12 == 0 {
            let st = sides.get(me);
            println!("BOT: gold {} income {:.0} vezi {}/{} (staveb {})", st.gold, st.income.amount, mine.len(), want, st.builds);
        }

        // 1) spojeni: dvojice stejneho druhu a ranku (nejnizsi rank prvni). Az kdyz ma bot aspon 6 vezi,
        //    aby si na zacatku nerozbijel bludiste. Uvolnene policko pozdeji zastavi znovu.
        if mine.len() >= 6 {
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
                if try_merge(&mut commands, &asset_server, &mut sides, me, from, to) { continue; }
            }
        }

        // 2) stavba: dalsi volne policko z planu; kdyz je plan plny, kterekoli volne policko. Nikdy nezazdi cestu.
        //    Kdyz uz ma dost vezi, obcas stavbu vynecha a posle jednotku (investice do income).
        let enough = mine.len() >= want;
        let invest = enough && round.phase == RoundPhase::Running && rand::thread_rng().gen_bool(0.3);
        let st = sides.get(me);
        let cost = build_cost(st.builds);
        if st.gold >= cost && !invest {
            let free = |&(r, c): &(usize, usize)| !st.grid.blocked[r][c] && keeps_path_open(&st.grid.blocked, r, c);
            let next = bot_plan().into_iter().find(free)
                .or_else(|| (2..ROWS - 1).rev().flat_map(|r| (0..COLS).map(move |c| (r, c))).find(free));
            if let Some((r, c)) = next {
                if try_build_random(me, r, c, &mut commands, &mut sides, &asset_server) { continue; }
            }
        }

        // 3) posilani: jen behem kola a jen z goldu navic (aby mel na dalsi stavbu); posle nejdrazsi jednotku do tretiny goldu
        if round.phase != RoundPhase::Running { continue; }
        let Some(assets) = assets.as_ref() else { continue; };
        let st = sides.get_mut(me);
        // dokud nema dost vezi, posila jen z velkeho prebytku (2x cena stavby); pak z goldu nad cenou dalsi stavby,
        // pri investici az do poloviny goldu
        let cost = build_cost(st.builds);
        let budget = if invest { st.gold / 2 } else if enough { st.gold - cost } else { st.gold - 2 * cost };
        if budget <= 0 { continue; }
        let pick = (1..=UNIT_COUNT).filter(|&k| get_unit_def(k).cost <= budget).max_by_key(|&k| get_unit_def(k).cost);
        let Some(kind) = pick else { continue; };
        let def = get_unit_def(kind);
        st.gold -= def.cost;
        let idx = (kind as usize).min(31);
        st.income.amount += def.income as f32 * income_multiplier(st.income.sent[idx]);
        st.income.sent[idx] += 1;
        let (hp_mul, speed_mul) = barracks_buff(me, &all_towers);
        spawn_sent_unit(&mut commands, &assets, &mut meshes, &mut materials, &asset_server, me.other(), kind, hp_mul, speed_mul);
        println!("Bot poslal {} za {}G (income {:.1})", def.name, def.cost, st.income.amount);
    }
}

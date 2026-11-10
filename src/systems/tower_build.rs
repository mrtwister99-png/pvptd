
use bevy::prelude::*;
use crate::components::{Tower, Selling, UpgradeLabel, SellLabel, RangeRing, RangeRingNext, HoverHighlight, TowerLevelText, PauseButtonUI, BuildPreview, BuildToggleButton, BuildPanel, BuildPanelItem, SendToggleButton, SendPanel, TowerActionPanel, UpgradeButton, SellButton, SelectionMarker, SafeRoot};
use crate::resources::{HoverCell, SelectedCell, Gold, Grid, PathMap, WavePaused, HeldBuilding, PanelsOpen, InputPointer, SafeInsets};
use crate::constants::{CELL, LANE_H, LANE_W, CAM_BASE_Y, tower_icon_path, MAX_TOWER_LEVEL, SELL_SECONDS, SELL_REFUND};
use crate::tower_def::{get_tower_def, get_total_spent_gold};
use crate::utils::{grid_to_world, compute_path};

fn try_build_tower(
    kind: u8,
    row: usize,
    col: usize,
    commands: &mut Commands,
    gold: &mut ResMut<Gold>,
    grid: &mut ResMut<Grid>,
    path: &mut ResMut<PathMap>,
    asset_server: &Res<AssetServer>,
) -> bool {
    if row >= crate::constants::ROWS || col >= crate::constants::COLS { return false; }
    if grid.blocked[row][col] { println!("Blokovano [{},{}]", col, row); return false; }
    let def = get_tower_def(kind, 1);
    if gold.0 < def.gold { println!("Malo gold: potreba {} mas {}", def.gold, gold.0); return false; }
    let mut test_blocked = grid.blocked.clone();
    test_blocked[row][col] = true;
    let test_dist = compute_path(&test_blocked);
    // stavba muze zablokovat cestu - creepy pak stoji pred zdi, dokud hlidac (guardian_system) nezboura vez v ceste

    gold.0 -= def.gold;
    grid.blocked[row][col] = true;
    path.dist = test_dist;
    let pos = grid_to_world(row, col);
    let fire_rate = 1.0 / def.interval.max(0.1);
    let avg = (def.dmg_min + def.dmg_max)*0.5;
    let sprite = match tower_icon_path(kind) {
        Some(path) => Sprite { image: asset_server.load(path), custom_size: Some(Vec2::new(CELL - 6.0, CELL - 6.0)), ..default() },
        None => Sprite { color: def.color, custom_size: Some(Vec2::new(CELL - 12.0, CELL - 12.0)), ..default() }, // docasne ctverecek
    };

    commands.spawn((
        sprite,
        Transform::from_xyz(pos.x, pos.y, 5.0),
        Tower { damage: avg, damage_min: def.dmg_min, damage_max: def.dmg_max, range: def.range, fire_rate, cooldown:0.0, kind, col, row, level:1, splash:def.splash, slow_per_hit:def.slow_per, slow_max:def.slow_max, push_chance:def.push_chance, push_dist:def.push_dist, dot_dps:def.dot_dps, dot_duration:def.dot_dur, chain_targets:def.chain, hit_count:0 },
    )).with_children(|p| { p.spawn((Text2d::new("Lv1"), TextFont { font_size: 10.0, ..default() }, TextColor(Color::WHITE), Transform::from_xyz(0.0, -18.0, 1.0), TowerLevelText)); });

    println!("Postavena vez {:?} na [{},{}]", kind, col, row);
    true
}

// kolik gold vrati prodej veze
fn sell_price(kind: u8, level: u32) -> i32 {
    (get_total_spent_gold(kind, level) as f32 * SELL_REFUND) as i32
}

// klik na PRODAT: spusti odpocet prodeje (behem nej dal strili); klik na ZRUSIT (tlacitko behem prodeje) prodej zrusi
pub fn sell_tower_input(
    mut commands: Commands,
    selected_cell: Res<SelectedCell>,
    q_towers: Query<(Entity, &Tower, &Children, Has<Selling>)>,
    mut q_level_text: Query<&mut Text2d, With<TowerLevelText>>,
    q_btn: Query<&Interaction, (Changed<Interaction>, With<SellButton>)>,
) {
    if !q_btn.iter().any(|i| *i == Interaction::Pressed) { return; } // jen klik na tlacitko PRODAT / ZRUSIT
    let Some((row, col)) = selected_cell.0 else { return; };
    let Some((ent, tower, children, selling)) = q_towers.iter().find(|(_, t, _, _)| t.row == row && t.col == col) else { return; };
    if selling {
        // zrusit prodej: vratit napis levelu
        commands.entity(ent).remove::<Selling>();
        for &child in children.iter() {
            if let Ok(mut txt) = q_level_text.get_mut(child) { txt.0 = format!("Lv{}", tower.level); }
        }
    } else {
        commands.entity(ent).insert(Selling { timer: Timer::from_seconds(SELL_SECONDS, TimerMode::Once) });
    }
}

// odpocet prodeje: nad vezi bezi "PRODEJ 3..1", po dokonceni zmizi a vrati se gold
pub fn selling_system(
    mut commands: Commands,
    time: Res<Time>,
    mut gold: ResMut<Gold>,
    mut grid: ResMut<Grid>,
    mut path: ResMut<PathMap>,
    mut selected_cell: ResMut<SelectedCell>,
    mut q_selling: Query<(Entity, &Tower, &mut Selling, &Children)>,
    mut q_level_text: Query<&mut Text2d, With<TowerLevelText>>,
) {
    for (ent, tower, mut selling, children) in q_selling.iter_mut() {
        selling.timer.tick(time.delta());
        if selling.timer.finished() {
            let price = sell_price(tower.kind, tower.level);
            grid.blocked[tower.row][tower.col] = false;
            path.dist = compute_path(&grid.blocked);
            commands.entity(ent).despawn_recursive();
            gold.0 += price;
            if selected_cell.0 == Some((tower.row, tower.col)) { selected_cell.0 = None; }
            println!("Prodana vez {:?} Lv{} za {}G", tower.kind, tower.level, price);
            continue;
        }
        let label = format!("PRODEJ {:.0}", selling.timer.remaining_secs().ceil());
        for &child in children.iter() {
            if let Ok(mut txt) = q_level_text.get_mut(child) { if txt.0 != label { txt.0 = label.clone(); } }
        }
    }
}

pub fn upgrade_tower_input(
    mut gold: ResMut<Gold>,
    selected_cell: Res<SelectedCell>,
    mut q_towers: Query<(&mut Tower, &Children), Without<Selling>>,
    mut q_level_text: Query<&mut Text2d, With<TowerLevelText>>,
    q_btn: Query<&Interaction, (Changed<Interaction>, With<UpgradeButton>)>,
) {
    if !q_btn.iter().any(|i| *i == Interaction::Pressed) { return; } // jen klik na zelene UPGRADE
    let Some((row, col)) = selected_cell.0 else { return; };
    for (mut tower, children) in q_towers.iter_mut() {
        if tower.row == row && tower.col == col {
            if tower.level >= MAX_TOWER_LEVEL { println!("MAX LEVEL 5/5!"); return; }
            let next_def = get_tower_def(tower.kind, tower.level+1);
            if gold.0 < next_def.gold { println!("Potrebujes {}G, mas {}G", next_def.gold, gold.0); return; }
            gold.0 -= next_def.gold;
            tower.level += 1;
            tower.damage_min = next_def.dmg_min;
            tower.damage_max = next_def.dmg_max;
            tower.damage = (next_def.dmg_min + next_def.dmg_max)*0.5;
            tower.range = next_def.range;
            tower.fire_rate = 1.0 / next_def.interval.max(0.1);
            tower.splash = next_def.splash;
            tower.slow_per_hit = next_def.slow_per;
            tower.slow_max = next_def.slow_max;
            tower.push_chance = next_def.push_chance;
            tower.push_dist = next_def.push_dist;
            tower.dot_dps = next_def.dot_dps;
            tower.dot_duration = next_def.dot_dur;
            tower.chain_targets = next_def.chain;
            for &child in children.iter() {
                if let Ok(mut txt) = q_level_text.get_mut(child) { txt.0 = format!("Lv{}", tower.level); }
            }
            println!("Upgraded na Lv{} {}", tower.level, next_def.name);
            return;
        }
    }
}

// jeden ukazatel pro mys i dotyk (dotyk ma prednost); "pressed" = prave stisknuto / klepnuto
// Dotyk: klepnuti se vyhodnoti az PO PUSTENI prstu, a jen kdyz se prst netahl (posun kamery), nebyly dva prsty (zoom)
// a dotyk nezacal na UI tlacitku. Mys: klik hned.
pub fn pointer_system(
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    q_ui: Query<&Interaction, With<Button>>,
    mut pointer: ResMut<InputPointer>,
    mut tap: Local<(f32, bool, bool)>, // (kolik se prst pohnul, byly dva prsty, dotyk zacal na UI)
) {
    pointer.pressed = false;
    if touches.iter_just_pressed().next().is_some() && touches.iter().count() == 1 { *tap = (0.0, false, false); } // novy dotyk
    if touches.iter().count() >= 2 { tap.1 = true; }
    if q_ui.iter().any(|i| *i != Interaction::None) { tap.2 = true; }
    for t in touches.iter() { tap.0 += t.delta().length(); }
    if let Some(t) = touches.iter_just_released().next() {
        pointer.pos = Some(t.position());
        pointer.pressed = tap.0 <= 8.0 && !tap.1 && !tap.2;
    } else if let Some(t) = touches.iter().next() {
        pointer.pos = Some(t.position()); // prst drzi na displeji
    } else if let Ok(window) = windows.get_single() {
        pointer.pos = window.cursor_position();
        pointer.pressed = mouse.just_pressed(MouseButton::Left);
    }
}

pub fn mouse_hover_system(
    pointer: Res<InputPointer>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut hover: ResMut<HoverCell>,
    mut q_highlight: Query<(&mut Transform, &mut Visibility), With<HoverHighlight>>,
) {
    let Some(cursor_pos) = pointer.pos else { hover.0 = None; for (_, mut vis) in q_highlight.iter_mut() { *vis = Visibility::Hidden; } return; };
    let Ok((camera, cam_tf)) = camera_q.get_single() else { return; };
    let Ok(world_pos) = camera.viewport_to_world_2d(cam_tf, cursor_pos) else { return; };
    if let Some((r,c)) = crate::utils::world_to_grid(world_pos.x, world_pos.y) {
        hover.0 = Some((r,c));
        let wp = crate::utils::grid_to_world(r,c);
        for (mut tf, mut vis) in q_highlight.iter_mut() { tf.translation.x = wp.x; tf.translation.y = wp.y; *vis = Visibility::Visible; }
    } else {
        hover.0 = None;
        for (_, mut vis) in q_highlight.iter_mut() { *vis = Visibility::Hidden; }
    }
}

// klik / klepnuti: drzim budovu = postavit; prazdne policko = otevre nabidku B; vez = vyber (UPGRADE / PRODAT)
pub fn mouse_click_select_system(
    pointer: Res<InputPointer>,
    hover: Res<HoverCell>,
    mut selected: ResMut<SelectedCell>,
    mut commands: Commands,
    mut gold: ResMut<Gold>,
    mut grid: ResMut<Grid>,
    mut path: ResMut<PathMap>,
    asset_server: Res<AssetServer>,
    mut held: ResMut<HeldBuilding>,
    mut panels: ResMut<PanelsOpen>,
    q_ui: Query<&Interaction, With<Button>>,
    q_towers: Query<&Tower>,
) {
    // klik na UI tlacitko se nesmi propsat do mrizky pod nim
    if q_ui.iter().any(|i| *i != Interaction::None) { return; }
    if !pointer.pressed { return; }
    panels.build = false; // klik mimo menu = zavrit B / S
    panels.send = false;
    panels.build_target = None;
    match (hover.0, held.0) {
        // drzim budovu -> klik na bunku = postavit (pri uspechu pustim)
        (Some(cell), Some(kind)) => {
            if try_build_tower(kind, cell.0, cell.1, &mut commands, &mut gold, &mut grid, &mut path, &asset_server) {
                held.0 = None;
                selected.0 = None;
            }
        }
        (Some(cell), None) => {
            selected.0 = Some(cell);
            // prazdne policko = otevrit nabidku B (stavi se primo na toto policko); vez = jen vyber
            if !q_towers.iter().any(|t| t.row == cell.0 && t.col == cell.1) {
                panels.build = true;
                panels.build_target = Some(cell);
            }
        }
        // klik mimo mrizku = zrusit vyber
        (None, _) => selected.0 = None,
    }
}

pub fn keyboard_pause_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<WavePaused>,
) {
    if keys.just_pressed(KeyCode::Space) {
        paused.0 = !paused.0;
    }
}

pub fn pause_button_system_top_left(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PauseButtonUI>)>,
    mut paused: ResMut<WavePaused>,
) {
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            paused.0 = !paused.0;
            println!("{}", if paused.0 { "PAUSED" } else { "RESUMED" });
        }
    }
}

// B / S: otevirani panelu (klik nebo klavesa); vyber budovy: kdyz je cilove policko, postavi se tam, jinak se drzi na kurzoru
pub fn panels_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    q_b: Query<&Interaction, (Changed<Interaction>, With<BuildToggleButton>)>,
    q_s: Query<&Interaction, (Changed<Interaction>, With<SendToggleButton>)>,
    q_items: Query<(&Interaction, &BuildPanelItem), Changed<Interaction>>,
    mut commands: Commands,
    mut gold: ResMut<Gold>,
    mut grid: ResMut<Grid>,
    mut path: ResMut<PathMap>,
    asset_server: Res<AssetServer>,
    mut selected: ResMut<SelectedCell>,
    mut panels: ResMut<PanelsOpen>,
    mut held: ResMut<HeldBuilding>,
) {
    let b_clicked = q_b.iter().any(|i| *i == Interaction::Pressed);
    let s_clicked = q_s.iter().any(|i| *i == Interaction::Pressed);
    if keys.just_pressed(KeyCode::KeyB) || b_clicked {
        if held.0.is_some() {
            held.0 = None; // B behem drzeni budovy = zrusit
        } else {
            panels.build = !panels.build;
            panels.build_target = None;
            panels.send = false;
        }
    }
    if keys.just_pressed(KeyCode::KeyS) || s_clicked {
        held.0 = None;
        panels.send = !panels.send;
        panels.build = false;
        panels.build_target = None;
    }
    for (interaction, item) in q_items.iter() {
        if *interaction != Interaction::Pressed { continue; }
        if gold.0 < get_tower_def(item.0, 1).gold { continue; }
        match panels.build_target {
            // nabidka otevrena klepnutim na policko: postavit rovnou tam
            Some((r, c)) => {
                if try_build_tower(item.0, r, c, &mut commands, &mut gold, &mut grid, &mut path, &asset_server) {
                    panels.build = false;
                    panels.build_target = None;
                    selected.0 = None;
                }
            }
            // nabidka z tlacitka B: budova na kurzor, panel se zavre
            None => {
                held.0 = Some(item.0);
                panels.build = false;
            }
        }
    }
}

// zobrazeni panelu (Display), ztmaveni budov a UPGRADE bez penez, UPGRADE/PRODAT (s cenami) jen pro vybranou vez
pub fn panels_display_system(
    panels: Res<PanelsOpen>,
    held: Res<HeldBuilding>,
    gold: Res<Gold>,
    selected_cell: Res<SelectedCell>,
    q_towers: Query<(&Tower, Has<Selling>)>,
    mut q_build: Query<&mut Node, (With<BuildPanel>, Without<SendPanel>, Without<TowerActionPanel>)>,
    mut q_send: Query<&mut Node, (With<SendPanel>, Without<BuildPanel>, Without<TowerActionPanel>)>,
    mut q_action: Query<&mut Node, (With<TowerActionPanel>, Without<BuildPanel>, Without<SendPanel>)>,
    mut q_items: Query<(&BuildPanelItem, &mut BackgroundColor)>,
    mut q_up_label: Query<&mut Text, (With<UpgradeLabel>, Without<SellLabel>)>,
    mut q_sell_label: Query<&mut Text, (With<SellLabel>, Without<UpgradeLabel>)>,
    mut q_up_btn: Query<&mut BackgroundColor, (With<UpgradeButton>, Without<SellButton>, Without<BuildPanelItem>)>,
    mut q_sell_btn: Query<&mut BackgroundColor, (With<SellButton>, Without<UpgradeButton>, Without<BuildPanelItem>)>,
) {
    // vybrana postavena vez (kind, level, prave se prodava)
    let selected = selected_cell.0.and_then(|(r, c)| q_towers.iter().find(|(t, _)| t.row == r && t.col == c).map(|(t, s)| (t.kind, t.level, s)));
    let show = |open: bool| if open { Display::Flex } else { Display::None };
    for mut n in q_build.iter_mut() { n.display = show(panels.build); }
    for mut n in q_send.iter_mut() { n.display = show(panels.send); }
    for mut n in q_action.iter_mut() { n.display = show(selected.is_some() && held.0.is_none() && !panels.build && !panels.send); }
    for (item, mut bg) in q_items.iter_mut() {
        let def = get_tower_def(item.0, 1);
        let c = def.color.to_srgba();
        bg.0 = if gold.0 >= def.gold { def.color } else { Color::srgb(c.red * 0.25, c.green * 0.25, c.blue * 0.25) };
    }
    if let Some((kind, level, selling)) = selected {
        // UPGRADE: cena dalsiho levelu; ztmaveny, kdyz nejsou penize / max level / probiha prodej
        let next_cost = if level >= MAX_TOWER_LEVEL { None } else { Some(get_tower_def(kind, level + 1).gold) };
        let can_upgrade = !selling && next_cost.map_or(false, |c| gold.0 >= c);
        let up = if selling { "UPGRADE\n-".to_string() } else { match next_cost { Some(c) => format!("UPGRADE\n{}G", c), None => "UPGRADE\nMAX".to_string() } };
        // PRODAT -> pri prodeji se zmeni na ZRUSIT
        let sell = if selling { "ZRUSIT\nPRODEJ".to_string() } else { format!("PRODAT\n+{}G\n({:.0}s)", sell_price(kind, level), SELL_SECONDS) };
        for mut t in q_up_label.iter_mut() { if t.0 != up { t.0 = up.clone(); } }
        for mut t in q_sell_label.iter_mut() { if t.0 != sell { t.0 = sell.clone(); } }
        let up_col = if can_upgrade { Color::srgb(0.15, 0.6, 0.2) } else { Color::srgb(0.07, 0.15, 0.08) };
        let sell_col = if selling { Color::srgb(0.8, 0.55, 0.1) } else { Color::srgb(0.75, 0.2, 0.15) };
        for mut bg in q_up_btn.iter_mut() { if bg.0 != up_col { bg.0 = up_col; } }
        for mut bg in q_sell_btn.iter_mut() { if bg.0 != sell_col { bg.0 = sell_col; } }
    }
}

// kruh dosahu: vybrane veze (bily) / budovy na kurzoru; pri najeti na UPGRADE vetsi zeleny kruh = dosah po upgradu
pub fn range_rings_system(
    held: Res<HeldBuilding>,
    hover: Res<HoverCell>,
    selected_cell: Res<SelectedCell>,
    q_up_btn: Query<&Interaction, With<UpgradeButton>>,
    q_towers: Query<(&Tower, &Transform)>,
    mut q_ring: Query<(&mut Transform, &mut Visibility), (With<RangeRing>, Without<RangeRingNext>, Without<Tower>)>,
    mut q_next: Query<(&mut Transform, &mut Visibility), (With<RangeRingNext>, Without<RangeRing>, Without<Tower>)>,
) {
    let mut ring: Option<(Vec2, f32)> = None; // (stred, polomer)
    let mut next: Option<f32> = None;          // polomer po upgradu
    if let Some(kind) = held.0 {
        if let Some((r, c)) = hover.0 { ring = Some((grid_to_world(r, c), get_tower_def(kind, 1).range)); }
    } else if let Some((r, c)) = selected_cell.0 {
        if let Some((tower, tf)) = q_towers.iter().find(|(t, _)| t.row == r && t.col == c) {
            ring = Some((tf.translation.truncate(), tower.range));
            let hovering_up = q_up_btn.iter().any(|i| *i != Interaction::None);
            if hovering_up && tower.level < MAX_TOWER_LEVEL { next = Some(get_tower_def(tower.kind, tower.level + 1).range); }
        }
    }
    for (mut tf, mut vis) in q_ring.iter_mut() {
        match ring {
            Some((c, r)) => { tf.translation.x = c.x; tf.translation.y = c.y; tf.scale = Vec3::new(r, r, 1.0); *vis = Visibility::Visible; }
            None => { if *vis != Visibility::Hidden { *vis = Visibility::Hidden; } }
        }
    }
    for (mut tf, mut vis) in q_next.iter_mut() {
        match (ring, next) {
            (Some((c, _)), Some(r)) => { tf.translation.x = c.x; tf.translation.y = c.y; tf.scale = Vec3::new(r, r, 1.0); *vis = Visibility::Visible; }
            _ => { if *vis != Visibility::Hidden { *vis = Visibility::Hidden; } }
        }
    }
}

// duch drzene budovy - sleduje bunku pod kurzorem
pub fn held_ghost_system(
    held: Res<HeldBuilding>,
    hover: Res<HoverCell>,
    asset_server: Res<AssetServer>,
    mut q_ghost: Query<(&mut Transform, &mut Visibility, &mut Sprite), With<BuildPreview>>,
) {
    for (mut tf, mut vis, mut sprite) in q_ghost.iter_mut() {
        match (held.0, hover.0) {
            (Some(kind), Some((r, c))) => {
                let wp = grid_to_world(r, c);
                tf.translation.x = wp.x;
                tf.translation.y = wp.y;
                match tower_icon_path(kind) {
                    Some(path) => {
                        sprite.image = asset_server.load(path);
                        sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.6);
                    }
                    None => {
                        // bez ikony: pruhledny barevny ctverecek
                        let c = get_tower_def(kind, 1).color.to_srgba();
                        sprite.image = Handle::default();
                        sprite.color = Color::srgba(c.red, c.green, c.blue, 0.6);
                    }
                }
                sprite.custom_size = Some(Vec2::new(CELL - 6.0, CELL - 6.0));
                *vis = Visibility::Visible;
            }
            _ => *vis = Visibility::Hidden,
        }
    }
}

// posouvani panelu jednotek (S): kolecko mysi nebo tah prstem; panel ma 2 sloupce a viditelnych 5 radku
const SEND_ITEM: f32 = 64.0;   // velikost ctverce
const SEND_GAP: f32 = 6.0;     // mezera mezi ctverci
const SEND_VIEW_H: f32 = 5.0 * SEND_ITEM + 4.0 * SEND_GAP; // viditelna vyska panelu (5 radku)

pub fn send_panel_scroll_system(
    mut wheel: EventReader<bevy::input::mouse::MouseWheel>,
    touches: Res<Touches>,
    panels: Res<PanelsOpen>,
    mut q: Query<&mut ScrollPosition, With<SendPanel>>,
) {
    if !panels.send { wheel.clear(); return; }
    let mut dy = 0.0;
    for ev in wheel.read() {
        dy -= match ev.unit { bevy::input::mouse::MouseScrollUnit::Line => ev.y * 30.0, bevy::input::mouse::MouseScrollUnit::Pixel => ev.y };
    }
    for t in touches.iter() { dy -= t.delta().y; } // tah prstem
    if dy == 0.0 { return; }
    let rows = (crate::wave_def::UNIT_COUNT as f32 / 2.0).ceil();
    let content_h = rows * SEND_ITEM + (rows - 1.0) * SEND_GAP;
    let max = (content_h - SEND_VIEW_H).max(0.0);
    for mut s in q.iter_mut() { s.offset_y = (s.offset_y + dy).clamp(0.0, max); }
}

// === KAMERA: zoom stipnutim dvou prstu, posun nahoru/dolu tahem jednoho prstu ===
// PC test: Ctrl + kolecko = zoom, Shift + kolecko = posun nahoru/dolu
const CAM_ZOOM_MIN: f32 = 0.5;    // scale kamery: mensi = vic priblizeno
const CAM_ZOOM_MAX: f32 = 1.6;    // vetsi = oddaleno (vidi se i pas nad lajnou)
const CAM_PAN_SLACK: f32 = 100.0; // o kolik se da vyjet za okraj lajny
const CAM_DRAG_MIN: f32 = 8.0;    // vic pixelu pohybu prstu = posun kamery (mene = klepnuti)

pub fn camera_gesture_system(
    touches: Res<Touches>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: EventReader<bevy::input::mouse::MouseWheel>,
    panels: Res<PanelsOpen>,
    windows: Query<&Window>,
    q_ui: Query<&Interaction, With<Button>>,
    mut cam: Query<&mut Transform, With<Camera2d>>,
    mut last_dist: Local<Option<f32>>, // vzdalenost prstu v minulem snimku (zoom)
    mut drag: Local<f32>,              // kolik se prst pohnul od dotyku
    insets: Res<SafeInsets>,           // okraje systemovych list
    mut last_extra: Local<f32>,        // posledni posun kamery podle okraju
) {
    let Ok(mut tf) = cam.get_single_mut() else { return; };
    // lajna se vycentruje do bezpecne oblasti: kdyz se zmeni okraje, kamera se posune o rozdil
    let extra = (insets.top - insets.bottom) / 2.0;
    if extra != *last_extra { tf.translation.y += extra - *last_extra; *last_extra = extra; }
    let mut scale = tf.scale.x;
    let mut dy = 0.0_f32; // posun kamery nahoru(+)/dolu(-) ve svetovych jednotkach
    let mut dx = 0.0_f32; // posun kamery vpravo(+)/vlevo(-)
    let mut anchor = Vec2::ZERO; // zoom ke kurzoru (PC): odsazeni kurzoru od stredu obrazovky v pixelech (y nahoru)
    let cursor_off = windows.get_single().ok().and_then(|w| w.cursor_position().map(|c| Vec2::new(c.x - w.width() / 2.0, w.height() / 2.0 - c.y))).unwrap_or(Vec2::ZERO);
    if touches.iter_just_pressed().next().is_some() { *drag = 0.0; }
    let mut it = touches.iter();
    match (it.next(), it.next()) {
        // dva prsty: prsty od sebe = priblizit, k sobe = oddalit
        (Some(a), Some(b)) => {
            let dist = a.position().distance(b.position());
            if let Some(prev) = *last_dist { if prev > 1.0 && dist > 1.0 { scale *= prev / dist; } }
            *last_dist = Some(dist);
            *drag = f32::MIN; // po zoomu zbyvajici prst uz neposouva, dokud se nezacne novy dotyk
        }
        // jeden prst: tah = posun (jen mimo tlacitka a kdyz neni otevreny panel jednotek, ten se taky posouva tahem)
        (Some(a), None) => {
            *last_dist = None;
            *drag += a.delta().length();
            if *drag > CAM_DRAG_MIN && !panels.send && !q_ui.iter().any(|i| *i != Interaction::None) {
                dy += a.delta().y * scale; // obsah jede s prstem: prst dolu = kamera nahoru
                dx -= a.delta().x * scale; // prst doprava = kamera doleva
            }
        }
        _ => { *last_dist = None; }
    }
    // PC: kolecko s modifikatorem
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    for ev in wheel.read() {
        let step = match ev.unit { bevy::input::mouse::MouseScrollUnit::Line => ev.y, bevy::input::mouse::MouseScrollUnit::Pixel => ev.y / 40.0 };
        if ctrl { scale *= 1.0 - step * 0.1; anchor = cursor_off; } else if shift { dy += step * 40.0 * scale; }
    }
    scale = scale.clamp(CAM_ZOOM_MIN, CAM_ZOOM_MAX);
    let half_h = windows.get_single().map(|w| w.height() / 2.0).unwrap_or(400.0);
    let half_w = windows.get_single().map(|w| w.width() / 2.0).unwrap_or(640.0);
    let limit_y = (LANE_H / 2.0 + CAM_PAN_SLACK - half_h * scale).max(CAM_PAN_SLACK); // jak daleko od stredu smi kamera
    let limit_x = (LANE_W / 2.0 + CAM_PAN_SLACK - half_w * scale).max(0.0);            // na siroke obrazovce se bocne nehybe
    let shift_zoom = anchor * (tf.scale.x - scale); // bod pod kurzorem zustane pri zoomu na miste
    let x = (tf.translation.x + dx + shift_zoom.x).clamp(-limit_x, limit_x);
    let y = (tf.translation.y + dy + shift_zoom.y).clamp(CAM_BASE_Y + extra - limit_y, CAM_BASE_Y + extra + limit_y); // okolo zakladniho posunu kamery
    if tf.scale.x != scale { tf.scale = Vec3::new(scale, scale, 1.0); }
    if tf.translation.x != x { tf.translation.x = x; }
    if tf.translation.y != y { tf.translation.y = y; }
}

// zluty ramecek na vybrane bunce (klepnuti/klik); prepisuje se jen kdyz se vyber zmenil
pub fn selection_marker_system(selected: Res<SelectedCell>, mut q: Query<(&mut Transform, &mut Visibility), With<SelectionMarker>>) {
    if !selected.is_changed() { return; }
    for (mut tf, mut vis) in q.iter_mut() {
        match selected.0 {
            Some((r, c)) => {
                let p = grid_to_world(r, c);
                tf.translation.x = p.x;
                tf.translation.y = p.y;
                if *vis != Visibility::Visible { *vis = Visibility::Visible; }
            }
            None => { if *vis != Visibility::Hidden { *vis = Visibility::Hidden; } }
        }
    }
}

// po startu prehodi vsechny hlavni UI uzly (bez rodice) pod jeden koren, jehoz okraje posouva safe_area_system
pub fn adopt_ui_system(mut commands: Commands, q: Query<Entity, (With<Node>, Without<Parent>)>) {
    let root = commands.spawn((
        Node { position_type: PositionType::Absolute, left: Val::Px(0.0), right: Val::Px(0.0), top: Val::Px(0.0), bottom: Val::Px(0.0), ..default() },
        SafeRoot,
    )).id();
    // koren (SafeRoot) musi mit rodice: korenove uzly Bevy ignoruji vlastni posun (top/left), potomci ho respektuji
    let outer = commands.spawn(Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }).id();
    commands.entity(outer).add_child(root);
    for e in q.iter() { commands.entity(root).add_child(e); }
}

// Android: 2x za sekundu precte okraje systemovych list a posune koren UI; na PC nic
pub fn safe_area_system(
    time: Res<Time>,
    windows: Query<&Window>,
    mut wait: Local<f32>,
    mut misses: Local<u32>, // kolikrat po sobe se okraje nepodarilo precist
    mut insets: ResMut<SafeInsets>,
    mut q: Query<&mut Node, With<SafeRoot>>,
) {
    *wait -= time.delta_secs();
    if *wait > 0.0 { return; }
    *wait = 0.5;
    let jni = crate::utils::android_insets_px().filter(|v| v.1 > 0 || v.3 > 0); // samé nuly = nespolehlive
    let scale = windows.get_single().map(|w| w.scale_factor()).unwrap_or(1.0).max(0.1);
    let new = match jni {
        Some((l, t, r, b)) => { *misses = 0; SafeInsets { left: l as f32 / scale, top: t as f32 / scale, right: r as f32 / scale, bottom: b as f32 / scale } }
        None => {
            if !cfg!(target_os = "android") { return; }
            *misses = (*misses + 1).min(100);
            if *misses < 4 { return; } // chvili pockat, okno jeste nemusi mit okraje
            if *misses == 4 { println!("Okraje z Androidu se nepodarilo precist, pouzivam pevne hodnoty"); }
            SafeInsets { left: 0.0, top: crate::constants::FALLBACK_INSET_TOP, right: 0.0, bottom: crate::constants::FALLBACK_INSET_BOTTOM }
        }
    };
    if *insets == new { return; }
    *insets = new;
    for mut n in q.iter_mut() {
        n.left = Val::Px(new.left);
        n.top = Val::Px(new.top);
        n.right = Val::Px(new.right);
        n.bottom = Val::Px(new.bottom);
    }
    println!("Bezpecne okraje (logicke px): nahore {:.0}, dole {:.0}, vlevo {:.0}, vpravo {:.0}", new.top, new.bottom, new.left, new.right);
}
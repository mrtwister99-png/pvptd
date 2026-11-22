
use bevy::prelude::*;
use crate::components::{Side, SwitchViewButton, Tower, Selling, UpgradeLabel, SellLabel, RangeRing, RangeRingNext, HoverHighlight, TowerLevelText, PauseButtonUI, BuildPreview, SendToggleButton, SendPanel, TowerActionPanel, UpgradeButton, SellButton, SelectionMarker, SafeRoot};
use crate::resources::{HoverCell, SelectedCell, Sides, ViewSide, MatchOver, TowerDrag, KeyControl, WavePaused, HeldBuilding, PanelsOpen, InputPointer, SafeInsets};
use crate::constants::{CELL, LANE_H, LANE_W, CAM_BASE_Y, tower_icon_path, MAX_TOWER_LEVEL, SELL_SECONDS, SELL_REFUND, BUILD_COST_STEP, ROWS, COLS};
use rand::Rng;
use crate::tower_def::get_tower_def;
use crate::utils::{grid_to_world, world_to_grid, compute_path};

// cena dalsi stavby: 10, 20, 30... az do stropu (podle toho, kolik vezi uz strana postavila)
pub fn build_cost(builds: u32) -> i32 { (BUILD_COST_STEP * (builds as i32 + 1)).min(crate::constants::BUILD_COST_MAX) }

// postavi vez daneho druhu a ranku na policko (gold ani bludiste neresi, to dela volajici)
pub fn spawn_tower(commands: &mut Commands, asset_server: &AssetServer, side: Side, kind: u8, rank: u32, row: usize, col: usize, invested: i32) {
    let def = get_tower_def(kind, rank);
    let pos = grid_to_world(side, row, col);
    let fire_rate = 1.0 / def.interval.max(0.1);
    let sprite = match tower_icon_path(kind) {
        Some(path) => Sprite { image: asset_server.load(path), custom_size: Some(Vec2::new(CELL - 6.0, CELL - 6.0)), ..default() },
        None => Sprite { color: def.color, custom_size: Some(Vec2::new(CELL - 12.0, CELL - 12.0)), ..default() }, // docasne ctverecek
    };
    commands.spawn((
        sprite,
        Transform::from_xyz(pos.x, pos.y, 5.0),
        Tower { side, damage_min: def.dmg_min, damage_max: def.dmg_max, range: def.range, fire_rate, cooldown: 0.0, kind, col, row, level: rank, splash: def.splash, slow_per_hit: def.slow_per, slow_max: def.slow_max, push_chance: def.push_chance, push_dist: def.push_dist, dot_dps: def.dot_dps, dot_duration: def.dot_dur, chain_targets: def.chain, hit_count: 0, invested },
    )).with_children(|p| {
        p.spawn((Text2d::new(format!("R{}", rank)), TextFont { font_size: 11.0, ..default() }, TextColor(Color::WHITE), Transform::from_xyz(0.0, -18.0, 1.0), TowerLevelText));
        // vez bez ikony: zkratka nazvu doprostred ctverecku, at se daji rozeznat
        if tower_icon_path(kind).is_none() {
            p.spawn((Text2d::new(crate::constants::tower_tag(kind)), TextFont { font_size: 12.0, ..default() }, TextColor(Color::BLACK), Transform::from_xyz(0.0, 3.0, 1.0)));
        }
    });
}

// stavba: nahodna vez ranku 1 z balicku strany na volne policko; zaplati se cena dalsi stavby
// (stavba muze zablokovat cestu - creepy pak stoji pred zdi, dokud hlidac nezboura vez v ceste)
pub fn try_build_random(side: Side, row: usize, col: usize, commands: &mut Commands, sides: &mut Sides, asset_server: &AssetServer) -> bool {
    let st = sides.get_mut(side);
    if row >= ROWS || col >= COLS || st.grid.blocked[row][col] || st.deck.is_empty() { return false; }
    let cost = build_cost(st.builds);
    if st.gold < cost { return false; }
    let kind = st.deck[rand::thread_rng().gen_range(0..st.deck.len())];
    st.gold -= cost;
    st.builds += 1;
    st.grid.blocked[row][col] = true;
    st.path.dist = compute_path(&st.grid.blocked);
    spawn_tower(commands, asset_server, side, kind, 1, row, col, cost);
    println!("{:?} postavil {} R1 na [{},{}] za {}G", side, kind, col, row, cost);
    true
}

// kopie toho, co spojovani potrebuje vedet o vezi
#[derive(Clone, Copy)]
pub struct TowerRef { pub e: Entity, pub kind: u8, pub rank: u32, pub row: usize, pub col: usize, pub invested: i32 }
impl TowerRef {
    pub fn of(e: Entity, t: &Tower) -> Self { Self { e, kind: t.kind, rank: t.level, row: t.row, col: t.col, invested: t.invested } }
}

// spojeni: dve stejne veze stejneho ranku (pod max) -> na miste `to` vznikne NAHODNA vez z balicku o rank vyssi,
// policko `from` se uvolni (zmeni se bludiste). Jine veze se nespoji a nic se nestane.
pub fn try_merge(commands: &mut Commands, asset_server: &AssetServer, sides: &mut Sides, side: Side, from: TowerRef, to: TowerRef) -> bool {
    if from.e == to.e || from.kind != to.kind || from.rank != to.rank || from.rank >= MAX_TOWER_LEVEL { return false; }
    let st = sides.get_mut(side);
    if st.deck.is_empty() { return false; }
    let kind = st.deck[rand::thread_rng().gen_range(0..st.deck.len())];
    commands.entity(from.e).despawn_recursive();
    commands.entity(to.e).despawn_recursive();
    spawn_tower(commands, asset_server, side, kind, to.rank + 1, to.row, to.col, from.invested + to.invested);
    if from.kind == crate::special::TWIN {
        // TWIN: na puvodnim miste zustane nova nahodna vez R1 z balicku (2x R1 -> R2 + R1), bludiste se nemeni
        let left = st.deck[rand::thread_rng().gen_range(0..st.deck.len())];
        spawn_tower(commands, asset_server, side, left, 1, from.row, from.col, 0);
    } else {
        st.grid.blocked[from.row][from.col] = false;
        st.path.dist = compute_path(&st.grid.blocked);
    }
    println!("{:?} spojil 2x R{} -> {} R{} na [{},{}]", side, to.rank, kind, to.rank + 1, to.col, to.row);
    true
}

// kolik gold vrati prodej veze
fn sell_price(invested: i32) -> i32 {
    (invested as f32 * SELL_REFUND) as i32
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
    let Some((ent, tower, children, selling)) = q_towers.iter().find(|(_, t, _, _)| t.side == Side::Player && t.row == row && t.col == col) else { return; };
    if selling {
        // zrusit prodej: vratit napis levelu
        commands.entity(ent).remove::<Selling>();
        for &child in children.iter() {
            if let Ok(mut txt) = q_level_text.get_mut(child) { txt.0 = format!("R{}", tower.level); }
        }
    } else {
        commands.entity(ent).try_insert(Selling { timer: Timer::from_seconds(SELL_SECONDS, TimerMode::Once) });
    }
}

// odpocet prodeje: nad vezi bezi "PRODEJ 3..1", po dokonceni zmizi a vrati se gold
pub fn selling_system(
    mut commands: Commands,
    time: Res<Time>,
    mut sides: ResMut<Sides>,
    mut selected_cell: ResMut<SelectedCell>,
    mut q_selling: Query<(Entity, &Tower, &mut Selling, &Children)>,
    mut q_level_text: Query<&mut Text2d, With<TowerLevelText>>,
) {
    for (ent, tower, mut selling, children) in q_selling.iter_mut() {
        selling.timer.tick(time.delta());
        if selling.timer.finished() {
            let price = sell_price(tower.invested);
            let st = sides.get_mut(tower.side);
            st.grid.blocked[tower.row][tower.col] = false;
            st.path.dist = compute_path(&st.grid.blocked);
            commands.entity(ent).despawn_recursive();
            st.gold += price;
            if tower.side == Side::Player && selected_cell.0 == Some((tower.row, tower.col)) { selected_cell.0 = None; }
            println!("Prodana vez {:?} R{} za {}G", tower.kind, tower.level, price);
            continue;
        }
        let label = format!("PRODEJ {:.0}", selling.timer.remaining_secs().ceil());
        for &child in children.iter() {
            if let Ok(mut txt) = q_level_text.get_mut(child) { if txt.0 != label { txt.0 = label.clone(); } }
        }
    }
}

// tlacitko POSTAVIT (dole uprostred): na vybranem prazdnem policku postavi nahodnou vez z balicku
pub fn build_button_input(
    mut commands: Commands,
    mut sides: ResMut<Sides>,
    selected_cell: Res<SelectedCell>,
    asset_server: Res<AssetServer>,
    over: Res<MatchOver>,
    q_towers: Query<&Tower>,
    q_btn: Query<&Interaction, (Changed<Interaction>, With<UpgradeButton>)>,
) {
    if !q_btn.iter().any(|i| *i == Interaction::Pressed) || over.0.is_some() { return; }
    let Some((row, col)) = selected_cell.0 else { return; };
    if q_towers.iter().any(|t| t.side == Side::Player && t.row == row && t.col == col) { return; }
    try_build_random(Side::Player, row, col, &mut commands, &mut sides, &asset_server);
    // policko zustava vybrane: tlacitko se prepne na PRODAT
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
    // stavet a vybirat se da jen na vlastni lajne (na lajne soupere je hover None)
    if let Some((r,c)) = crate::utils::world_to_grid(Side::Player, world_pos.x, world_pos.y) {
        hover.0 = Some((r,c));
        let wp = crate::utils::grid_to_world(Side::Player, r,c);
        for (mut tf, mut vis) in q_highlight.iter_mut() { tf.translation.x = wp.x; tf.translation.y = wp.y; *vis = Visibility::Visible; }
    } else {
        hover.0 = None;
        for (_, mut vis) in q_highlight.iter_mut() { *vis = Visibility::Hidden; }
    }
}

// klik / klepnuti na policko = vyber (prazdne -> tlacitko POSTAVIT, vez -> PRODAT); mimo mrizku = zrusit vyber
pub fn mouse_click_select_system(
    pointer: Res<InputPointer>,
    hover: Res<HoverCell>,
    mut selected: ResMut<SelectedCell>,
    mut panels: ResMut<PanelsOpen>,
    q_ui: Query<&Interaction, With<Button>>,
) {
    // klik na UI tlacitko se nesmi propsat do mrizky pod nim
    if q_ui.iter().any(|i| *i != Interaction::None) { return; }
    if !pointer.pressed { return; }
    panels.send = false; // klik mimo menu = zavrit S
    selected.0 = hover.0;
}

pub fn keyboard_pause_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut paused: ResMut<WavePaused>,
    over: Res<MatchOver>,
) {
    if over.0.is_some() { return; } // po konci zapasu uz pauza nejde zrusit
    if keys.just_pressed(KeyCode::Space) {
        paused.0 = !paused.0;
    }
}

pub fn pause_button_system_top_left(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<PauseButtonUI>)>,
    mut paused: ResMut<WavePaused>,
    over: Res<MatchOver>,
) {
    if over.0.is_some() { return; }
    for interaction in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            paused.0 = !paused.0;
            println!("{}", if paused.0 { "PAUSED" } else { "RESUMED" });
        }
    }
}

// S: otevreni / zavreni panelu jednotek (klik nebo klavesa S)
pub fn panels_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    q_s: Query<&Interaction, (Changed<Interaction>, With<SendToggleButton>)>,
    mut panels: ResMut<PanelsOpen>,
) {
    let s_clicked = q_s.iter().any(|i| *i == Interaction::Pressed);
    if keys.just_pressed(KeyCode::KeyS) || s_clicked { panels.send = !panels.send; }
}

// zobrazeni panelu S a tlacitek dole uprostred: prazdne vybrane policko = POSTAVIT (s cenou), vez = PRODAT
pub fn panels_display_system(
    panels: Res<PanelsOpen>,
    sides: Res<Sides>,
    selected_cell: Res<SelectedCell>,
    kc: Res<KeyControl>,
    q_towers: Query<(&Tower, Has<Selling>)>,
    mut q_send: Query<&mut Node, (With<SendPanel>, Without<TowerActionPanel>, Without<UpgradeButton>, Without<SellButton>)>,
    mut q_action: Query<&mut Node, (With<TowerActionPanel>, Without<SendPanel>, Without<UpgradeButton>, Without<SellButton>)>,
    mut q_up_label: Query<&mut Text, (With<UpgradeLabel>, Without<SellLabel>)>,
    mut q_sell_label: Query<&mut Text, (With<SellLabel>, Without<UpgradeLabel>)>,
    mut q_up_btn: Query<(&mut BackgroundColor, &mut Node), (With<UpgradeButton>, Without<SellButton>, Without<SendPanel>, Without<TowerActionPanel>)>,
    mut q_sell_btn: Query<(&mut BackgroundColor, &mut Node), (With<SellButton>, Without<UpgradeButton>, Without<SendPanel>, Without<TowerActionPanel>)>,
) {
    let st = sides.get(Side::Player);
    let show = |open: bool| if open { Display::Flex } else { Display::None };
    for mut n in q_send.iter_mut() { n.display = show(panels.send); }
    for mut n in q_action.iter_mut() { n.display = show(selected_cell.0.is_some() && !panels.send); }
    let Some((r, c)) = selected_cell.0 else { return; };
    let tower = q_towers.iter().find(|(t, _)| t.side == Side::Player && t.row == r && t.col == c).map(|(t, s)| (t.invested, s));
    for (_, mut n) in q_up_btn.iter_mut() { n.display = show(tower.is_none()); }
    for (_, mut n) in q_sell_btn.iter_mut() { n.display = show(tower.is_some()); }
    match tower {
        None => {
            // POSTAVIT: cena dalsi stavby, ztmavene bez penez
            let cost = build_cost(st.builds);
            let armed = kc.build_armed == Some((r, c));
            let up = if armed { format!("ENTER =\nPOSTAVIT\n{}G", cost) } else { format!("POSTAVIT\n{}G", cost) };
            for mut t in q_up_label.iter_mut() { if t.0 != up { t.0 = up.clone(); } }
            let col = if armed { Color::srgb(0.85, 0.7, 0.1) } else if st.gold >= cost { Color::srgb(0.15, 0.6, 0.2) } else { Color::srgb(0.07, 0.15, 0.08) };
            for (mut bg, _) in q_up_btn.iter_mut() { if bg.0 != col { bg.0 = col; } }
        }
        Some((invested, selling)) => {
            // PRODAT -> pri prodeji se zmeni na ZRUSIT
            let sell = if selling { "ZRUSIT\nPRODEJ".to_string() } else { format!("PRODAT\n+{}G\n({:.0}s)", sell_price(invested), SELL_SECONDS) };
            for mut t in q_sell_label.iter_mut() { if t.0 != sell { t.0 = sell.clone(); } }
            let col = if selling { Color::srgb(0.8, 0.55, 0.1) } else { Color::srgb(0.75, 0.2, 0.15) };
            for (mut bg, _) in q_sell_btn.iter_mut() { if bg.0 != col { bg.0 = col; } }
        }
    }
}

// kruh dosahu: vybrane veze (bily) / budovy na kurzoru; pri najeti na UPGRADE vetsi zeleny kruh = dosah po upgradu
pub fn range_rings_system(
    held: Res<HeldBuilding>,
    hover: Res<HoverCell>,
    selected_cell: Res<SelectedCell>,
    q_up_btn: Query<&Interaction, With<UpgradeButton>>,
    q_towers: Query<(&Tower, &Transform)>,
    mut q_ring: Query<(&mut Transform, &mut Visibility, &MeshMaterial2d<crate::fx::RangeMaterial>), (With<RangeRing>, Without<RangeRingNext>, Without<Tower>)>,
    mut range_mats: ResMut<Assets<crate::fx::RangeMaterial>>,
    mut last: Local<Option<(u8, u32)>>, // pro jakou vez (druh, polomer) je material nastaveny
    mut q_next: Query<(&mut Transform, &mut Visibility), (With<RangeRingNext>, Without<RangeRing>, Without<Tower>)>,
) {
    let mut ring: Option<(Vec2, f32)> = None; // (stred, polomer)
    let mut ring_kind: u8 = 0;
    let mut next: Option<f32> = None;          // polomer po upgradu
    if let Some(kind) = held.0 {
        if let Some((r, c)) = hover.0 { ring = Some((grid_to_world(Side::Player, r, c), get_tower_def(kind, 1).range)); }
    } else if let Some((r, c)) = selected_cell.0 {
        if let Some((tower, tf)) = q_towers.iter().find(|(t, _)| t.side == Side::Player && t.row == r && t.col == c) {
            ring = Some((tf.translation.truncate(), tower.range));
            ring_kind = tower.kind;
            let hovering_up = q_up_btn.iter().any(|i| *i != Interaction::None);
            if hovering_up && tower.level < MAX_TOWER_LEVEL { next = Some(get_tower_def(tower.kind, tower.level + 1).range); }
        }
    }
    for (mut tf, mut vis, mat) in q_ring.iter_mut() {
        match ring {
            Some((c, r)) => {
                tf.translation.x = c.x; tf.translation.y = c.y; tf.scale = Vec3::new(r * 2.0, r * 2.0, 1.0);
                if *vis != Visibility::Visible { *vis = Visibility::Visible; }
                // barva podle veze (svetlejsi), material se prepise jen kdyz se zmeni vez nebo polomer
                let key = (ring_kind, r as u32);
                if *last != Some(key) {
                    *last = Some(key);
                    let col = get_tower_def(ring_kind.max(1), 1).color.to_srgba();
                    let light = Color::srgba(0.5 + col.red * 0.5, 0.5 + col.green * 0.5, 0.5 + col.blue * 0.5, 0.95);
                    if let Some(m) = range_mats.get_mut(&mat.0) { m.params = crate::fx::range_params(light, r); }
                }
            }
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
    view: Res<ViewSide>,               // na kterou lajnu se divame
    tower_drag: Res<TowerDrag>,        // tahne se vez -> kamera se nehybe
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
            if *drag > CAM_DRAG_MIN && !panels.send && tower_drag.from.is_none() && !q_ui.iter().any(|i| *i != Interaction::None) {
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
    let cx = view.0.lane_x(); // stred prave sledovane lajny
    let x = (tf.translation.x + dx + shift_zoom.x).clamp(cx - limit_x, cx + limit_x);
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
                let p = grid_to_world(Side::Player, r, c);
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
// prepnuti kamery mezi moji lajnou a lajnou soupere (tlacitko vpravo nahore nebo Tab)
pub fn switch_view_system(
    keys: Res<ButtonInput<KeyCode>>,
    q_btn: Query<&Interaction, (Changed<Interaction>, With<SwitchViewButton>)>,
    mut view: ResMut<ViewSide>,
    mut cam: Query<&mut Transform, With<Camera2d>>,
    mut q_btn_bg: Query<&mut BackgroundColor, With<SwitchViewButton>>,
    mut held: ResMut<HeldBuilding>,
    mut panels: ResMut<PanelsOpen>,
    mut selected: ResMut<SelectedCell>,
    mut drag: ResMut<TowerDrag>,
    mut kc: ResMut<KeyControl>,
) {
    let clicked = q_btn.iter().any(|i| *i == Interaction::Pressed);
    if !clicked && !keys.just_pressed(KeyCode::Tab) { return; }
    let from = view.0;
    view.0 = from.other();
    // kamera skoci na stejne misto druhe lajny (zoom i vyska zustavaji)
    if let Ok(mut tf) = cam.get_single_mut() { tf.translation.x += view.0.lane_x() - from.lane_x(); }
    // na lajne soupere se nestavi: zrusit drzenou budovu, vyber a stavebni nabidku
    drag.from = None;
    kc.carry = None;
    kc.build_armed = None;
    if view.0 != Side::Player {
        held.0 = None;
        selected.0 = None;
        panels.build = false;
        panels.build_target = None;
    }
    // popisek tlacitka (vcetne zivotu soupere) prepisuje opponent_hp_system
    let col = if view.0 == Side::Player { Color::srgb(0.55, 0.2, 0.2) } else { Color::srgb(0.2, 0.45, 0.25) };
    for mut bg in q_btn_bg.iter_mut() { bg.0 = col; }
}

// PRETAHOVANI VEZE (spojovani): stisk na vlastni vezi a tah na jinou vez stejneho druhu a ranku = spojeni.
// Behem tahu jede pod prstem pruhledny duch veze. Pusteni jinde / na jinou vez = nic se nestane.
pub fn tower_drag_system(
    mut commands: Commands,
    windows: Query<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    q_ui: Query<&Interaction, With<Button>>,
    mut drag: ResMut<TowerDrag>,
    mut sides: ResMut<Sides>,
    asset_server: Res<AssetServer>,
    over: Res<MatchOver>,
    view: Res<ViewSide>,
    q_towers: Query<(Entity, &Tower, &Sprite), (Without<Selling>, Without<BuildPreview>)>,
    mut q_ghost: Query<(&mut Transform, &mut Visibility, &mut Sprite), (With<BuildPreview>, Without<Tower>)>,
) {
    let Ok((camera, cam_tf)) = camera_q.get_single() else { return; };
    let to_world = |p: Vec2| camera.viewport_to_world_2d(cam_tf, p).ok();
    // ukazatel: dotyk ma prednost pred mysi
    let (pressed_at, held_at, released_at) = if touches.iter().next().is_some() || touches.iter_just_released().next().is_some() {
        (touches.iter_just_pressed().next().map(|t| t.position()), touches.iter().next().map(|t| t.position()), touches.iter_just_released().next().map(|t| t.position()))
    } else {
        let cur = windows.get_single().ok().and_then(|w| w.cursor_position());
        (if mouse.just_pressed(MouseButton::Left) { cur } else { None }, if mouse.pressed(MouseButton::Left) { cur } else { None }, if mouse.just_released(MouseButton::Left) { cur } else { None })
    };

    // zacatek tahu: stisk na vlastni vezi (ne na tlacitku, ne pri dvou prstech, ne po konci zapasu)
    if let Some(p) = pressed_at {
        drag.from = None;
        drag.moved = false;
        let on_ui = q_ui.iter().any(|i| *i != Interaction::None);
        if !on_ui && over.0.is_none() && view.0 == Side::Player && touches.iter().count() <= 1 {
            if let Some(w) = to_world(p) {
                if let Some(cell) = world_to_grid(Side::Player, w.x, w.y) {
                    if q_towers.iter().any(|(_, t, _)| t.side == Side::Player && t.row == cell.0 && t.col == cell.1) {
                        drag.from = Some(cell);
                        drag.start = p;
                    }
                }
            }
        }
    }
    let Some(from) = drag.from else { return; };
    if touches.iter().count() >= 2 { drag.from = None; for (_, mut vis, _) in q_ghost.iter_mut() { *vis = Visibility::Hidden; } return; } // zoom = zrusit tah

    // behem tahu: duch veze pod prstem
    if let Some(p) = held_at {
        if p.distance(drag.start) > 10.0 { drag.moved = true; }
        if drag.moved {
            if let (Some(w), Some((_, _, src))) = (to_world(p), q_towers.iter().find(|(_, t, _)| t.side == Side::Player && t.row == from.0 && t.col == from.1)) {
                for (mut tf, mut vis, mut sprite) in q_ghost.iter_mut() {
                    tf.translation.x = w.x;
                    tf.translation.y = w.y;
                    sprite.image = src.image.clone();
                    let c = src.color.to_srgba();
                    sprite.color = Color::srgba(c.red, c.green, c.blue, 0.6);
                    sprite.custom_size = src.custom_size;
                    *vis = Visibility::Visible;
                }
            }
        }
    }

    // pusteni: na jine vezi = pokus o spojeni
    if let Some(p) = released_at {
        for (_, mut vis, _) in q_ghost.iter_mut() { *vis = Visibility::Hidden; }
        drag.from = None;
        if !drag.moved { return; }
        let Some(w) = to_world(p) else { return; };
        let Some(target) = world_to_grid(Side::Player, w.x, w.y) else { return; };
        if target == from { return; }
        let find = |cell: (usize, usize)| q_towers.iter().find(|(_, t, _)| t.side == Side::Player && t.row == cell.0 && t.col == cell.1).map(|(e, t, _)| TowerRef::of(e, t));
        if let (Some(a), Some(b)) = (find(from), find(target)) {
            if !try_merge(&mut commands, &asset_server, &mut sides, Side::Player, a, b) {
                println!("Tyhle veze se spojit nedaji (musi byt stejny druh a rank, max R{})", MAX_TOWER_LEVEL);
            }
        }
    }
}

// KLAVESNICE: sipky posouvaji vybrane policko (vzdy nejake je), Enter stavi / bere a spojuje veze, Esc rusi
pub fn keyboard_cursor_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedCell>,
    mut kc: ResMut<KeyControl>,
    mut sides: ResMut<Sides>,
    mut panels: ResMut<PanelsOpen>,
    asset_server: Res<AssetServer>,
    view: Res<ViewSide>,
    over: Res<MatchOver>,
    drag: Res<TowerDrag>,
    q_towers: Query<(Entity, &Tower, &Sprite), (Without<Selling>, Without<BuildPreview>)>,
    mut q_ghost: Query<(&mut Transform, &mut Visibility, &mut Sprite), (With<BuildPreview>, Without<Tower>)>,
) {
    if view.0 != Side::Player || over.0.is_some() {
        kc.build_armed = None;
        kc.carry = None;
    } else {
        let tower_at = |cell: (usize, usize)| q_towers.iter().find(|(_, t, _)| t.side == Side::Player && t.row == cell.0 && t.col == cell.1);
        // nesena vez mezitim zmizela (prodej, hlidac) -> zrusit
        if let Some(from) = kc.carry { if tower_at(from).is_none() { kc.carry = None; } }

        // sipky: radek 0 je dole (cil), nahoru = vyssi radek
        let mut d: (i32, i32) = (0, 0);
        if keys.just_pressed(KeyCode::ArrowUp) { d.0 += 1; }
        if keys.just_pressed(KeyCode::ArrowDown) { d.0 -= 1; }
        if keys.just_pressed(KeyCode::ArrowLeft) { d.1 -= 1; }
        if keys.just_pressed(KeyCode::ArrowRight) { d.1 += 1; }
        if d != (0, 0) {
            let (r, c) = selected.0.unwrap_or((ROWS / 2, COLS / 2));
            let r = (r as i32 + d.0).clamp(0, ROWS as i32 - 1) as usize;
            let c = (c as i32 + d.1).clamp(0, COLS as i32 - 1) as usize;
            selected.0 = Some((r, c));
            panels.send = false;
        }
        if keys.just_pressed(KeyCode::Escape) { kc.build_armed = None; kc.carry = None; }

        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
            match selected.0 {
                None => selected.0 = Some((ROWS / 2, COLS / 2)),
                Some(cell) => {
                    if let Some(from) = kc.carry {
                        // neseme vez: Enter na stejnem miste = polozit zpet, na stejne vezi stejneho ranku = spojit, jinde nic
                        if from == cell { kc.carry = None; }
                        else if let (Some((ea, ta, _)), Some((eb, tb, _))) = (tower_at(from), tower_at(cell)) {
                            if try_merge(&mut commands, &asset_server, &mut sides, Side::Player, TowerRef::of(ea, ta), TowerRef::of(eb, tb)) { kc.carry = None; }
                        }
                    } else if tower_at(cell).is_some() {
                        kc.build_armed = None;
                        kc.carry = Some(cell); // vzit vez
                    } else if kc.build_armed == Some(cell) {
                        try_build_random(Side::Player, cell.0, cell.1, &mut commands, &mut sides, &asset_server); // druhy Enter = postavit
                        kc.build_armed = None;
                    } else {
                        kc.build_armed = Some(cell); // prvni Enter = pripravit POSTAVIT
                    }
                }
            }
        }
        // priprava stavby plati jen pro policko, na kterem vznikla
        if kc.build_armed.is_some() && kc.build_armed != selected.0 { kc.build_armed = None; }
    }

    // duch nesene veze na vybranem policku (pri tahu mysi ducha ridi tower_drag_system)
    if drag.from.is_some() { return; }
    let src = kc.carry.and_then(|from| q_towers.iter().find(|(_, t, _)| t.side == Side::Player && t.row == from.0 && t.col == from.1));
    for (mut tf, mut vis, mut sprite) in q_ghost.iter_mut() {
        match (src, selected.0) {
            (Some((_, _, src)), Some((r, c))) => {
                let p = grid_to_world(Side::Player, r, c);
                tf.translation.x = p.x;
                tf.translation.y = p.y;
                sprite.image = src.image.clone();
                let col = src.color.to_srgba();
                sprite.color = Color::srgba(col.red, col.green, col.blue, 0.6);
                sprite.custom_size = src.custom_size;
                if *vis != Visibility::Visible { *vis = Visibility::Visible; }
            }
            _ => { if *vis != Visibility::Hidden { *vis = Visibility::Hidden; } }
        }
    }
}

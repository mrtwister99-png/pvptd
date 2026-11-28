
use bevy::prelude::*;
use crate::components::{Side, SwitchViewButton, Tower, Selling, UpgradeLabel, SellLabel, RangeRing, RangeRingNext, HoverHighlight, TowerLevelText, PauseButtonUI, SendToggleButton, SendPanel, TowerActionPanel, UpgradeButton, SellButton, SelectionMarker, SafeRoot, BuildTowerButton, BuildTowerPanel};
use crate::resources::{HoverCell, SelectedCell, Sides, ViewSide, MatchOver, KeyControl, WavePaused, HeldBuilding, PanelsOpen, InputPointer, SafeInsets, GameMode};
use crate::constants::{CELL, LANE_H, LANE_W, CAM_BASE_Y, tower_icon_path, SELL_SECONDS, SELL_REFUND, ROWS, COLS};
use crate::tower_def::get_tower_def;
use crate::utils::{grid_to_world, compute_path};

pub fn tower_build_cost(kind: u8) -> i32 {
    get_tower_def(kind, 1).gold
}

pub fn tower_upgrade_cost(kind: u8, level: u32) -> Option<i32> {
    if level >= crate::constants::tower_rank_cap(kind) {
        return None;
    }
    let multiplier = if crate::constants::is_special_tower(kind) {
        match level {
            1 => 0.80,
            2 => 1.30,
            _ => return None,
        }
    } else {
        match level {
            1 => 0.70,
            2 => 0.90,
            3 => 1.20,
            4 => 1.55,
            5 => 2.00,
            6 => 2.60,
            _ => return None,
        }
    };
    Some((tower_build_cost(kind) as f32 * multiplier).round().max(1.0) as i32)
}

fn selected_build_kind(deck: &[u8], selected: Option<u8>) -> Option<u8> {
    selected.filter(|kind| deck.contains(kind))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::Sides;

    #[derive(Resource, Default)]
    struct BuildAttempt(Option<bool>);

    fn attempt_special_build(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut sides: ResMut<Sides>,
        mut attempt: ResMut<BuildAttempt>,
    ) {
        attempt.0 = Some(try_build_kind(
            Side::Player,
            0,
            0,
            16,
            &mut commands,
            &mut sides,
            &asset_server,
        ));
    }

    #[derive(Resource, Default)]
    struct UpgradeAttempt(Vec<bool>);

    // postavi Bramboromet R1 na [0,0] a zkusi ho 2x vylepsit
    fn attempt_upgrades(mut commands: Commands, asset_server: Res<AssetServer>, mut sides: ResMut<Sides>, mut attempt: ResMut<UpgradeAttempt>, mut step: Local<u32>, towers: Query<(Entity, &Tower)>) {
        *step += 1;
        if *step == 1 {
            assert!(try_build_kind(Side::Player, 0, 0, 1, &mut commands, &mut sides, &asset_server));
            return;
        }
        if let Some((e, t)) = towers.iter().next() {
            attempt.0.push(try_upgrade(&mut commands, &asset_server, &mut sides, Side::Player, TowerRef::of(e, t)));
        }
    }

    #[test]
    fn upgrade_costs_gold_raises_rank_and_stops_without_gold() {
        let mut sides = Sides::default();
        sides.get_mut(Side::Player).gold = 50 + 35; // stavba Brambrometu + upgrade na R2, na R3 (45 g) uz nezbyde
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(AssetPlugin::default())
            .insert_resource(sides).insert_resource(UpgradeAttempt::default())
            .add_systems(Update, attempt_upgrades);
        for _ in 0..3 { app.update(); }
        assert_eq!(app.world().resource::<UpgradeAttempt>().0, vec![true, false]);
        let st = app.world().resource::<Sides>().get(Side::Player);
        assert_eq!(st.gold, 0);
        assert!(st.grid.blocked[0][0]); // upgrade bludiste nemeni
        let mut q = app.world_mut().query::<&Tower>();
        let t = q.iter(app.world()).next().unwrap();
        assert_eq!((t.level, t.invested), (2, 85));
    }

    #[test]
    fn building_choice_must_be_in_the_selected_deck() {
        assert_eq!(selected_build_kind(&[1, 4, 7], Some(4)), Some(4));
        assert_eq!(selected_build_kind(&[1, 4, 7], Some(2)), None);
        assert_eq!(selected_build_kind(&[1, 4, 7], None), None);
    }

    #[test]
    fn basic_upgrades_cost_the_design_percentage_of_the_building_price() {
        assert_eq!(tower_upgrade_cost(1, 1), Some(35));
        assert_eq!(tower_upgrade_cost(1, 6), Some(130));
        assert_eq!(tower_upgrade_cost(1, 7), None);
    }

    #[test]
    fn special_upgrades_cost_eighty_and_one_hundred_thirty_percent_and_stop_at_rank_three() {
        assert_eq!(tower_build_cost(16), 400);
        assert_eq!(tower_upgrade_cost(16, 1), Some(320));
        assert_eq!(tower_upgrade_cost(16, 2), Some(520));
        assert_eq!(tower_upgrade_cost(16, 3), None);
    }

    #[test]
    fn special_build_requires_and_spends_one_diamond_only_after_a_successful_build() {
        let mut sides = Sides::default();
        let player = sides.get_mut(Side::Player);
        player.gold = 500;
        player.deck = vec![16];

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .insert_resource(sides)
            .insert_resource(BuildAttempt::default())
            .add_systems(Update, attempt_special_build);

        app.update();
        assert_eq!(app.world().resource::<BuildAttempt>().0, Some(false));
        assert_eq!(app.world().resource::<Sides>().get(Side::Player).gold, 500);
        assert_eq!(app.world().resource::<Sides>().get(Side::Player).diamonds, 0);

        app.world_mut().resource_mut::<Sides>().get_mut(Side::Player).diamonds = 1;
        app.update();
        assert_eq!(app.world().resource::<BuildAttempt>().0, Some(true));
        assert_eq!(app.world().resource::<Sides>().get(Side::Player).gold, 100);
        assert_eq!(app.world().resource::<Sides>().get(Side::Player).diamonds, 0);
        assert!(app.world().resource::<Sides>().get(Side::Player).grid.blocked[0][0]);
    }

    #[test]
    fn basic_and_special_towers_have_their_gdd_rank_caps() {
        assert_eq!(crate::constants::tower_rank_cap(1), 7);
        assert_eq!(crate::constants::tower_rank_cap(16), 3);
    }

    #[test]
    fn starting_gold_covers_the_four_cheapest_basic_towers() {
        let opening_cost: i32 = [2, 1, 3, 4]
            .iter()
            .map(|&kind| tower_build_cost(kind))
            .sum();
        assert!(opening_cost <= crate::constants::START_GOLD);
    }
}

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
        Tower { side, damage_min: def.dmg_min, damage_max: def.dmg_max, range: def.range, fire_rate, cooldown: 0.0, kind, col, row, level: rank, splash: def.splash, slow_per_hit: def.slow_per, slow_max: def.slow_max, push_chance: def.push_chance, push_dist: def.push_dist, dot_dps: def.dot_dps, dot_duration: def.dot_dur, chain_targets: def.chain, hit_count: 0, last_ability_wave: 0, ability_used_match: false, dj_drop_timer: 10.0, invested },
    )).with_children(|p| {
        p.spawn((Text2d::new(format!("R{}", rank)), TextFont { font_size: 11.0, ..default() }, TextColor(Color::WHITE), Transform::from_xyz(0.0, -18.0, 1.0), TowerLevelText));
        // vez bez ikony: zkratka nazvu doprostred ctverecku, at se daji rozeznat
        if tower_icon_path(kind).is_none() {
            p.spawn((Text2d::new(crate::constants::tower_tag(kind)), TextFont { font_size: 12.0, ..default() }, TextColor(Color::BLACK), Transform::from_xyz(0.0, 3.0, 1.0)));
        }
    });
}

pub fn try_build_kind(side: Side, row: usize, col: usize, kind: u8, commands: &mut Commands, sides: &mut Sides, asset_server: &AssetServer) -> bool {
    let st = sides.get_mut(side);
    if row >= ROWS || col >= COLS || st.grid.blocked[row][col] || !st.deck.contains(&kind) { return false; }
    let cost = tower_build_cost(kind);
    let diamond_cost = i32::from(crate::constants::is_special_tower(kind));
    if st.gold < cost || st.diamonds < diamond_cost { return false; }
    st.gold -= cost;
    st.diamonds -= diamond_cost;
    st.builds += 1;
    st.grid.blocked[row][col] = true;
    st.path.dist = compute_path(&st.grid.blocked);
    spawn_tower(commands, asset_server, side, kind, 1, row, col, cost);
    println!("{:?} postavil {} R1 na [{},{}] za {}G", side, kind, col, row, cost);
    true
}

// kopie toho, co vylepseni (a bot) potrebuje vedet o vezi
#[derive(Clone, Copy)]
pub struct TowerRef { pub e: Entity, pub kind: u8, pub rank: u32, pub row: usize, pub col: usize, pub invested: i32 }
impl TowerRef {
    pub fn of(e: Entity, t: &Tower) -> Self { Self { e, kind: t.kind, rank: t.level, row: t.row, col: t.col, invested: t.invested } }
}

// vylepseni za gold (GDD: veze se nespojuji, rank 1-7 / Special 1-3 se kupuje): vez se nahradi stejnou vezi o rank vys
// na stejnem policku, bludiste se nemeni. Kdyz chybi gold nebo je vez na max ranku, nic se nestane.
pub fn try_upgrade(commands: &mut Commands, asset_server: &AssetServer, sides: &mut Sides, side: Side, t: TowerRef) -> bool {
    let Some(cost) = tower_upgrade_cost(t.kind, t.rank) else { return false; };
    let st = sides.get_mut(side);
    if st.gold < cost { return false; }
    st.gold -= cost;
    commands.entity(t.e).despawn_recursive();
    spawn_tower(commands, asset_server, side, t.kind, t.rank + 1, t.row, t.col, t.invested + cost);
    println!("{:?} vylepsil {} na R{} za {}G", side, t.kind, t.rank + 1, cost);
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
    mode: Res<GameMode>,
) {
    for (ent, tower, mut selling, children) in q_selling.iter_mut() {
        selling.timer.tick(time.delta());
        if selling.timer.finished() || mode.is_test() { // v testu se vez rozbije hned
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

// tlacitko na prazdnem policku postavi zvolenou vez; na vezi koupi dalsi rank
pub fn build_button_input(
    mut commands: Commands,
    mut sides: ResMut<Sides>,
    selected_cell: Res<SelectedCell>,
    held: Res<HeldBuilding>,
    asset_server: Res<AssetServer>,
    over: Res<MatchOver>,
    q_towers: Query<(Entity, &Tower, &Children, Has<Selling>)>,
    mut q_sprites: Query<&mut Sprite, With<Tower>>,
    mut q_level_text: Query<&mut Text2d, With<TowerLevelText>>,
    q_btn: Query<&Interaction, (Changed<Interaction>, With<UpgradeButton>)>,
) {
    if !q_btn.iter().any(|i| *i == Interaction::Pressed) || over.0.is_some() { return; }
    let Some((row, col)) = selected_cell.0 else { return; };
    let tower = q_towers.iter().find(|(_, t, _, _)| t.side == Side::Player && t.row == row && t.col == col);
    let Some((kind, next_rank, invested, cooldown, hit_count, last_ability_wave, ability_used_match, dj_drop_timer, entity, children)) = tower.map(|(e, t, children, selling)| {
        if selling { return (0, 0, 0, 0.0, 0, 0, false, 10.0, Entity::PLACEHOLDER, children); }
        (t.kind, t.level + 1, t.invested, t.cooldown, t.hit_count, t.last_ability_wave, t.ability_used_match, t.dj_drop_timer, e, children)
    }) else {
        let Some(kind) = selected_build_kind(&sides.get(Side::Player).deck, held.0) else { return; };
        try_build_kind(Side::Player, row, col, kind, &mut commands, &mut sides, &asset_server);
        return;
    };
    if entity == Entity::PLACEHOLDER { return; }
    let Some(cost) = tower_upgrade_cost(kind, next_rank - 1) else { return; };
    let st = sides.get_mut(Side::Player);
    if st.gold < cost { return; }
    st.gold -= cost;
    let def = get_tower_def(kind, next_rank);
    commands.entity(entity).insert(Tower {
        side: Side::Player,
        damage_min: def.dmg_min,
        damage_max: def.dmg_max,
        range: def.range,
        fire_rate: 1.0 / def.interval.max(0.1),
        cooldown,
        kind,
        col,
        row,
        level: next_rank,
        splash: def.splash,
        slow_per_hit: def.slow_per,
        slow_max: def.slow_max,
        push_chance: def.push_chance,
        push_dist: def.push_dist,
        dot_dps: def.dot_dps,
        dot_duration: def.dot_dur,
        chain_targets: def.chain,
        hit_count,
        last_ability_wave,
        ability_used_match,
        dj_drop_timer,
        invested: invested + cost,
    });
    if let Ok(mut sprite) = q_sprites.get_mut(entity) {
        if tower_icon_path(kind).is_none() {
            sprite.color = def.color;
        }
    }
    for &child in children.iter() {
        if let Ok(mut text) = q_level_text.get_mut(child) {
            text.0 = format!("R{}", next_rank);
        }
    }
    println!("Vylepseni {:?} veze {} na R{} za {}G", Side::Player, kind, next_rank, cost);
}

pub fn build_choice_input_system(
    mut held: ResMut<HeldBuilding>,
    sides: Res<Sides>,
    view: Res<ViewSide>,
    over: Res<MatchOver>,
    choices: Query<(&Interaction, &BuildTowerButton), Changed<Interaction>>,
) {
    if view.0 != Side::Player || over.0.is_some() { return; }
    for (interaction, choice) in &choices {
        if *interaction == Interaction::Pressed {
            held.0 = selected_build_kind(&sides.get(Side::Player).deck, Some(choice.0));
        }
    }
}

pub fn build_choice_display_system(
    selected_cell: Res<SelectedCell>,
    view: Res<ViewSide>,
    held: Res<HeldBuilding>,
    panels: Res<PanelsOpen>,
    towers: Query<&Tower>,
    mut panel: Query<&mut Node, With<BuildTowerPanel>>,
    mut choices: Query<(&BuildTowerButton, &mut BackgroundColor), Without<BuildTowerPanel>>,
) {
    let empty_cell = selected_cell.0.map_or(false, |(row, col)| {
        !towers.iter().any(|tower| tower.side == Side::Player && tower.row == row && tower.col == col)
    });
    let show = view.0 == Side::Player && empty_cell && !panels.send;
    for mut node in &mut panel {
        node.display = if show { Display::Flex } else { Display::None };
    }
    for (choice, mut color) in &mut choices {
        color.0 = if held.0 == Some(choice.0) {
            Color::srgb(0.55, 0.43, 0.12)
        } else {
            Color::srgb(0.18, 0.18, 0.2)
        };
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
    selected: Res<SelectedCell>,
    view: Res<ViewSide>,
    q_towers: Query<&Tower>,
) {
    let s_clicked = q_s.iter().any(|i| *i == Interaction::Pressed);
    // klavesa S na vlastni vezi = prodej (resi keyboard_cursor_system), jinak panel jednotek
    let on_tower = view.0 == Side::Player && selected.0.map_or(false, |(r, c)| q_towers.iter().any(|t| t.side == Side::Player && t.row == r && t.col == c));
    if (keys.just_pressed(KeyCode::KeyS) && !on_tower) || s_clicked { panels.send = !panels.send; }
}

// okno potvrzeni prodeje: ukazat / skryt, text s cenou, kliknuti na ANO / NE
pub fn sell_confirm_ui_system(
    mut commands: Commands,
    mut kc: ResMut<KeyControl>,
    q_towers: Query<(Entity, &Tower), Without<Selling>>,
    mut q_root: Query<&mut Node, With<crate::components::SellConfirmRoot>>,
    mut q_text: Query<&mut Text, With<crate::components::SellConfirmText>>,
    q_yes: Query<&Interaction, (Changed<Interaction>, With<crate::components::SellYesButton>)>,
    q_no: Query<&Interaction, (Changed<Interaction>, With<crate::components::SellNoButton>)>,
    mode: Res<GameMode>,
) {
    let tower = kc.sell_confirm.and_then(|(r, c)| q_towers.iter().find(|(_, t)| t.side == Side::Player && t.row == r && t.col == c));
    if kc.sell_confirm.is_some() && tower.is_none() { kc.sell_confirm = None; } // vez mezitim zmizela
    let show = tower.is_some();
    for mut n in q_root.iter_mut() { let d = if show { Display::Flex } else { Display::None }; if n.display != d { n.display = d; } }
    let Some((e, t)) = tower else { return; };
    let name = crate::constants::ALL_TOWERS.iter().find(|x| x.0 == t.kind).map_or("?", |x| x.1);
    let s = if mode.is_test() { format!("Rozbit {} R{}?", name, t.level) } else { format!("Prodat {} R{} za +{}G?", name, t.level, sell_price(t.invested)) };
    for mut txt in q_text.iter_mut() { if txt.0 != s { txt.0 = s.clone(); } }
    if q_yes.iter().any(|i| *i == Interaction::Pressed) {
        commands.entity(e).try_insert(Selling { timer: Timer::from_seconds(SELL_SECONDS, TimerMode::Once) });
        kc.sell_confirm = None;
    } else if q_no.iter().any(|i| *i == Interaction::Pressed) {
        kc.sell_confirm = None;
    }
}

// zobrazeni panelu S a tlacitek: prazdne policko = POSTAVIT, vez = UPGRADE / PRODAT
pub fn panels_display_system(
    panels: Res<PanelsOpen>,
    sides: Res<Sides>,
    selected_cell: Res<SelectedCell>,
    held: Res<HeldBuilding>,
    q_towers: Query<(&Tower, Has<Selling>)>,
    mut q_send: Query<&mut Node, (With<SendPanel>, Without<TowerActionPanel>, Without<UpgradeButton>, Without<SellButton>)>,
    mut q_action: Query<&mut Node, (With<TowerActionPanel>, Without<SendPanel>, Without<UpgradeButton>, Without<SellButton>)>,
    mut q_up_label: Query<&mut Text, (With<UpgradeLabel>, Without<SellLabel>)>,
    mut q_sell_label: Query<&mut Text, (With<SellLabel>, Without<UpgradeLabel>)>,
    mut q_up_btn: Query<(&mut BackgroundColor, &mut Node), (With<UpgradeButton>, Without<SellButton>, Without<SendPanel>, Without<TowerActionPanel>)>,
    mut q_sell_btn: Query<(&mut BackgroundColor, &mut Node), (With<SellButton>, Without<UpgradeButton>, Without<SendPanel>, Without<TowerActionPanel>)>,
    mode: Res<GameMode>,
) {
    let st = sides.get(Side::Player);
    let show = |open: bool| if open { Display::Flex } else { Display::None };
    for mut n in q_send.iter_mut() { n.display = show(panels.send); }
    for mut n in q_action.iter_mut() { n.display = show(selected_cell.0.is_some() && !panels.send); }
    let tower = selected_cell.0.and_then(|(r, c)| {
        q_towers.iter().find(|(t, _)| t.side == Side::Player && t.row == r && t.col == c)
    });
    for (_, mut n) in q_up_btn.iter_mut() { n.display = show(selected_cell.0.is_some() && !panels.send); }
    for (_, mut n) in q_sell_btn.iter_mut() { n.display = show(tower.is_some()); }
    match tower {
        None => {
            let kind = selected_build_kind(&st.deck, held.0);
            let cost = kind.map(tower_build_cost);
            let up = match (kind, cost) {
                (Some(kind), Some(cost)) => format!("POSTAVIT\n{}\n{}G", crate::constants::tower_tag(kind), cost),
                _ => "VYBER VEZ".to_string(),
            };
            for mut t in q_up_label.iter_mut() { if t.0 != up { t.0 = up.clone(); } }
            let col = if cost.map_or(false, |cost| st.gold >= cost) { Color::srgb(0.15, 0.6, 0.2) } else { Color::srgb(0.07, 0.15, 0.08) };
            for (mut bg, _) in q_up_btn.iter_mut() { if bg.0 != col { bg.0 = col; } }
        }
        Some((tower, selling)) => {
            let upgrade = tower_upgrade_cost(tower.kind, tower.level)
                .map(|cost| format!("UPGRADE\nR{}\n{}G", tower.level + 1, cost))
                .unwrap_or_else(|| format!("MAX\nR{}", crate::constants::tower_rank_cap(tower.kind)));
            for mut t in q_up_label.iter_mut() { if t.0 != upgrade { t.0 = upgrade.clone(); } }
            let upgrade_cost = tower_upgrade_cost(tower.kind, tower.level);
            let upgrade_color = if upgrade_cost.map_or(false, |cost| st.gold >= cost) {
                Color::srgb(0.15, 0.6, 0.2)
            } else {
                Color::srgb(0.07, 0.15, 0.08)
            };
            for (mut bg, _) in q_up_btn.iter_mut() { if bg.0 != upgrade_color { bg.0 = upgrade_color; } }

            let sell = if mode.is_test() { "ROZBIT".to_string() } else if selling { "ZRUSIT\nPRODEJ".to_string() } else { format!("PRODAT\n+{}G\n({:.0}s)", sell_price(tower.invested), SELL_SECONDS) };
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
            if hovering_up && tower.level < crate::constants::tower_rank_cap(tower.kind) { next = Some(get_tower_def(tower.kind, tower.level + 1).range); }
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
    mut kc: ResMut<KeyControl>,
) {
    let clicked = q_btn.iter().any(|i| *i == Interaction::Pressed);
    if !clicked && !keys.just_pressed(KeyCode::Tab) { return; }
    let from = view.0;
    view.0 = from.other();
    // kamera skoci na stejne misto druhe lajny (zoom i vyska zustavaji)
    if let Ok(mut tf) = cam.get_single_mut() { tf.translation.x += view.0.lane_x() - from.lane_x(); }
    // na lajne soupere se nestavi: zrusit drzenou budovu, vyber a stavebni nabidku
    kc.sell_confirm = None;
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

// KLAVESNICE: sipky posouvaji vybrane policko (vzdy nejake je), Shift+sipky posouvaji o 2 policka,
// Enter na prazdnem policku stavi, Enter na vezi ji vylepsi za gold,
// S na vezi otevre okno prodeje (Enter = ano, Esc = ne), Esc rusi
pub fn keyboard_cursor_system(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedCell>,
    mut kc: ResMut<KeyControl>,
    mut sides: ResMut<Sides>,
    mut panels: ResMut<PanelsOpen>,
    held: Res<HeldBuilding>,
    asset_server: Res<AssetServer>,
    view: Res<ViewSide>,
    over: Res<MatchOver>,
    q_towers: Query<(Entity, &Tower), Without<Selling>>,
) {
    if view.0 != Side::Player || over.0.is_some() {
        kc.sell_confirm = None;
    } else if let Some(cell) = kc.sell_confirm {
        // otevrene okno prodeje: Enter = prodat (spusti odpocet prodeje), Esc = nechat; nic jineho nejde
        let enter = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter);
        if enter {
            if let Some((e, _)) = q_towers.iter().find(|(_, t)| t.side == Side::Player && t.row == cell.0 && t.col == cell.1) {
                commands.entity(e).try_insert(Selling { timer: Timer::from_seconds(SELL_SECONDS, TimerMode::Once) });
            }
            kc.sell_confirm = None;
        } else if keys.just_pressed(KeyCode::Escape) {
            kc.sell_confirm = None;
        }
    } else {
        let tower_at = |cell: (usize, usize)| q_towers.iter().find(|(_, t)| t.side == Side::Player && t.row == cell.0 && t.col == cell.1);

        // sipky: radek 0 je dole (cil), nahoru = vyssi radek
        let mut d: (i32, i32) = (0, 0);
        let step = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) { 2 } else { 1 };
        if keys.just_pressed(KeyCode::ArrowUp) { d.0 += step; }
        if keys.just_pressed(KeyCode::ArrowDown) { d.0 -= step; }
        if keys.just_pressed(KeyCode::ArrowLeft) { d.1 -= step; }
        if keys.just_pressed(KeyCode::ArrowRight) { d.1 += step; }
        if d != (0, 0) {
            let (r, c) = selected.0.unwrap_or((ROWS / 2, COLS / 2));
            let r = (r as i32 + d.0).clamp(0, ROWS as i32 - 1) as usize;
            let c = (c as i32 + d.1).clamp(0, COLS as i32 - 1) as usize;
            selected.0 = Some((r, c));
            panels.send = false;
        }
        // S na vlastni vezi = prodat (nejdriv okno ANO / NE); jinde S otevira panel jednotek (panels_input_system)
        if keys.just_pressed(KeyCode::KeyS) {
            if let Some(cell) = selected.0 { if tower_at(cell).is_some() { kc.sell_confirm = Some(cell); } }
        }

        if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
            match selected.0 {
                None => selected.0 = Some((ROWS / 2, COLS / 2)),
                Some(cell) => {
                    if let Some((e, t)) = tower_at(cell) {
                        try_upgrade(&mut commands, &asset_server, &mut sides, Side::Player, TowerRef::of(e, t));
                    } else if let Some(kind) = selected_build_kind(&sides.get(Side::Player).deck, held.0) {
                        try_build_kind(Side::Player, cell.0, cell.1, kind, &mut commands, &mut sides, &asset_server);
                    }
                }
            }
        }
    }
}

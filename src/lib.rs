use bevy::prelude::*;
mod constants;
mod components;
mod resources;
mod tower_def;
mod wave_def;
mod utils;
mod systems;
mod menu;
mod deck;
mod special;
mod fx;
use constants::{COLS, ROWS, CELL, LANE_W, LANE_H, LANE_Y, FINISH_EXTRA_ROWS, SAFE_TOP, SAFE_BOTTOM, CAM_BASE_Y};
use resources::*;
use components::*;
use utils::{compute_path, grid_to_world};
use systems::*;
use menu::*;
#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Line TD - mobil+PC klik only + bullet + hp bar".into(),
                resolution: Vec2::new(1280.0_f32, 800.0_f32).into(),
                // test na PC: HIDDEN_WINDOW=1 = okno se otevre mimo obrazovku (skryte okno by se neprekreslovalo a hra by stala)
                position: if std::env::var("HIDDEN_WINDOW").is_ok() { WindowPosition::At(IVec2::new(-5000, -5000)) } else { WindowPosition::Automatic },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(fx::FxPlugin)
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, autoplay_system)
        .add_systems(Update, dev_screenshot_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, sell_confirm_ui_system.after(keyboard_cursor_system).run_if(in_state(AppState::Playing)))
        .add_systems(OnEnter(AppState::Menu), spawn_menu)
        .add_systems(OnExit(AppState::Menu), despawn_menu)
        .add_systems(Update, menu_button_system.run_if(in_state(AppState::Menu)))
        .add_systems(OnEnter(AppState::Restarting), start_restarted_game)
        .insert_resource(RoundState::default())
        .insert_resource(MatchOver::default())
        .insert_resource(WavePaused(false))
        .insert_resource(PendingPauseAction::default())
        .insert_resource(Sides::default())
        .insert_resource(ViewSide(Side::Player))
        .insert_resource(WaveCount(0))
        .insert_resource(InputPointer::default())
        .insert_resource(HoverCell(None))
        .insert_resource(SelectedCell(Some((ROWS / 2, COLS / 2)))) // vzdy je nejake policko vybrane (klavesnice)
        .insert_resource(HeldBuilding(None))
        .insert_resource(PanelsOpen::default())
        .insert_resource(Deck::default())
        .insert_resource(KeyControl::default())
        .init_resource::<GameMode>()
        .add_systems(OnEnter(AppState::DeckSelect), deck::spawn_deck_screen)
        .add_systems(OnExit(AppState::DeckSelect), deck::despawn_deck_screen)
        .add_systems(Update, (deck::deck_input_system, deck::deck_display_system).run_if(in_state(AppState::DeckSelect)))
        .insert_resource(DamageNumberPool::default())
        .insert_resource(BulletPool::default())
        .insert_resource(CreepGrid::default())
        .insert_resource(ParticlePool::default())
        .insert_resource(BoltPool::default())
        .insert_resource(HealRingPool::default())
        .insert_resource(SafeInsets::default())
        .add_systems(OnEnter(AppState::Playing), (setup, adopt_ui_system).chain())
        .add_systems(OnEnter(AppState::Playing), fx::setup_fx)
        .add_systems(OnExit(AppState::Playing), cleanup_game)
        .add_systems(OnEnter(AppState::Playing), (setup_damage_pool, setup_bullet_pool, setup_creep_assets, setup_particle_pool, setup_bolt_pool, setup_heal_ring_pool))
        .add_systems(Update, (
            // zadny hrac - jen mys / klik
            pointer_system.before(mouse_hover_system),
            mouse_hover_system.before(mouse_click_select_system),
            mouse_click_select_system,
            range_rings_system,
            sell_tower_input,
            build_button_input,
            keyboard_pause_system,
            pause_button_system_top_left,
            panels_input_system,
            panels_display_system,
            send_unit_system,
            income_system,
            selling_system,
            particle_spawn_system,
            particle_update_system.after(particle_spawn_system),
            bolt_update_system,
            send_panel_scroll_system,
            ignite_visual_system,
        ).run_if(in_state(AppState::Playing)))
        .add_systems(Update, (build_choice_input_system, build_choice_display_system).run_if(in_state(AppState::Playing)))
        .add_systems(Update, (
            tower_shoot,
            move_bullets,
            move_creeps_maze,
            build_creep_grid.after(move_creeps_maze).before(bullet_hit), // mrizka creepu pred zasahy
            dj_drop_system.after(build_creep_grid),
            bullet_hit,
            update_debuffs,
            despawn_dead_and_count,
            round_system.run_if(not_test_mode),
            update_ui,
            update_wave_timer.after(round_system), // UI odpocet az po ticku kola
            update_hp_display,
            update_hp_bar,
            update_outline,
            enrage_system,
            enrage_flash_system,
            heal_system,
            guardian_spawn_system.run_if(not_test_mode),
            guardian_system,
            update_damage_numbers,
        ).run_if(in_state(AppState::Playing)))
        .add_systems(Update, (
            heal_ring_update_system,
            aura_system,
            shoot_back_system,
            tower_slow_system,
            camera_gesture_system,
            shimmer_system,
            guardian_dispatch_system,
            selection_marker_system,
            safe_area_system,
            cheat_button_system,
            tower_aura_system,
            switch_view_system.run_if(not_test_mode),
            game_over_system,
            game_over_button_system,
            bot_system.run_if(not_test_mode),
            tower_info_system,
            opponent_hp_system,
            keyboard_cursor_system,
        ).run_if(in_state(AppState::Playing)))
        .add_systems(Update, pause_menu_system.run_if(in_state(AppState::Playing)))
        .add_systems(Update, test_mode_system.run_if(in_state(AppState::Playing)).run_if(in_test_mode))
        .run();
}

// odchod ze zapasu (do menu): smazat vsechno herni (krome kamery) a vratit stav na zacatek,
// aby dalsi HRAT zacal novy zapas
fn cleanup_game(
    mut commands: Commands,
    q: Query<Entity, (Or<(With<Transform>, With<Node>)>, Without<Camera>, Without<Parent>, Without<Window>)>,
    mut cam: Query<&mut Transform, With<Camera2d>>,
) {
    for e in q.iter() { commands.entity(e).despawn_recursive(); }
    if let Ok(mut tf) = cam.get_single_mut() { *tf = Transform::from_xyz(0.0, CAM_BASE_Y, 0.0); }
    commands.insert_resource(Sides::default());
    commands.insert_resource(ViewSide(Side::Player));
    commands.insert_resource(RoundState::default());
    commands.insert_resource(MatchOver::default());
    commands.insert_resource(WavePaused(false));
    commands.insert_resource(PendingPauseAction::default());
    commands.insert_resource(WaveCount(0));
    commands.insert_resource(SelectedCell(Some((ROWS / 2, COLS / 2))));
    commands.insert_resource(HoverCell(None));
    commands.insert_resource(HeldBuilding(None));
    commands.insert_resource(PanelsOpen::default());
    commands.insert_resource(KeyControl::default());
    // pooly: jejich entity byly smazany, setup je vytvori znovu
    commands.insert_resource(DamageNumberPool::default());
    commands.insert_resource(BulletPool::default());
    commands.insert_resource(CreepGrid::default());
    commands.insert_resource(ParticlePool::default());
    commands.insert_resource(BoltPool::default());
    commands.insert_resource(HealRingPool::default());
}

// test na PC: SCREENSHOT_AT="20,40" + SCREENSHOT_DIR=slozka ulozi screenshoty v danych sekundach zapasu
// (SCREENSHOT_VIEW=bot = kamera na lajne bota; 0,5 s pred snimkem se vybere vlastni vez, at je videt kruh dosahu)
fn dev_screenshot_system(
    mut commands: Commands,
    time: Res<Time>,
    mut start: Local<Option<f32>>,
    mut done: Local<usize>,
    mut view: ResMut<ViewSide>,
    mut cam: Query<&mut Transform, With<Camera2d>>,
    mut selected: ResMut<SelectedCell>,
    towers: Query<&Tower>,
) {
    let Ok(at) = std::env::var("SCREENSHOT_AT") else { return; };
    let times: Vec<f32> = at.split(',').filter_map(|t| t.trim().parse().ok()).collect();
    let t0 = *start.get_or_insert(time.elapsed_secs());
    let t = time.elapsed_secs() - t0;
    if std::env::var("SCREENSHOT_VIEW").map_or(false, |v| v == "bot") && view.0 != Side::Bot {
        view.0 = Side::Bot;
        if let Ok(mut tf) = cam.get_single_mut() { tf.translation.x = Side::Bot.lane_x(); }
    }
    let Some(&next) = times.get(*done) else { return; };
    if t >= next - 0.5 {
        if let Some(tw) = towers.iter().find(|tw| tw.side == view.0 && tw.range > 0.0) { if view.0 == Side::Player { selected.0 = Some((tw.row, tw.col)); } }
    }
    if t >= next {
        let dir = std::env::var("SCREENSHOT_DIR").unwrap_or_else(|_| ".".into());
        let path = format!("{}/shot_{}.png", dir, *done);
        println!("SCREENSHOT {}", path);
        commands.spawn(bevy::render::view::screenshot::Screenshot::primary_window()).observe(bevy::render::view::screenshot::save_to_disk(path));
        *done += 1;
    }
}

// test na PC: s promennou prostredi AUTOPLAY=1 se preskoci menu a rovnou zacne zapas
// (AUTOPLAY=test = rovnou TEST rezim)
// GAME_SPEED=4 = hra bezi 4x rychleji (testy ekonomiky bot proti botovi)
fn autoplay_system(mut next: ResMut<NextState<AppState>>, mut mode: ResMut<GameMode>, mut time: ResMut<Time<Virtual>>) {
    if let Some(speed) = std::env::var("GAME_SPEED").ok().and_then(|v| v.parse::<f32>().ok()) {
        time.set_relative_speed(speed);
        time.set_max_delta(std::time::Duration::from_millis(1000));
    }
    let Ok(v) = std::env::var("AUTOPLAY") else { return; };
    if v == "test" { *mode = GameMode::Test; }
    next.set(AppState::Playing);
}

fn start_restarted_game(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::Playing);
}

fn setup(
    mut commands: Commands,
    mut sides: ResMut<Sides>,
    mut held: ResMut<HeldBuilding>,
    deck: Res<Deck>,
    mode: Res<GameMode>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut range_mats: ResMut<Assets<fx::RangeMaterial>>,
) {
    // kamera se spousti uz v menu (menu::spawn_camera)
    for st in sides.0.iter_mut() { st.path.dist = compute_path(&st.grid.blocked); }
    // balicky: hrac ma svuj z menu, bot si nahodne vybere 5 ze vsech vezi
    // TEST: k dispozici vsechny veze a neomezene penize
    let test = mode.is_test();
    let player_deck: Vec<u8> = if test { constants::ALL_TOWERS.iter().map(|t| t.0).collect() } else { deck.0.clone() };
    {
        let st = sides.get_mut(Side::Player);
        st.deck = player_deck.clone();
        if test { st.gold = TEST_GOLD; st.diamonds = TEST_DIAMONDS; }
    }
    held.0 = player_deck.first().copied();
    println!("Rezim zapasu: {:?}", *mode);
    {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        let mut basic: Vec<u8> = constants::ALL_TOWERS.iter().map(|t| t.0).filter(|kind| !constants::is_special_tower(*kind)).collect();
        let mut special: Vec<u8> = constants::ALL_TOWERS.iter().map(|t| t.0).filter(|kind| constants::is_special_tower(*kind)).collect();
        basic.shuffle(&mut rng);
        special.shuffle(&mut rng);
        basic.truncate(constants::DECK_BASIC_SIZE);
        special.truncate(constants::DECK_SPECIAL_SIZE);
        let mut all = basic;
        all.extend(special);
        all.shuffle(&mut rng);
        println!("Balicek hrace {:?}, bota {:?}", deck.0, all);
        sides.get_mut(Side::Bot).deck = all;
    }

    // dve lajny: hracova a bota (posunuta o SIDE_OFFSET_X); bot ma lehce cervenejsi pozadi
    // TEST: jen vlastni hraci pole (bot neni)
    let lanes: &[Side] = if test { &[Side::Player] } else { &Side::ALL };
    for &side in lanes {
        let lane_x = side.lane_x();
        let bg = if side == Side::Player { Color::srgb(0.16, 0.16, 0.18) } else { Color::srgb(0.2, 0.15, 0.16) };
        commands.spawn((
            Sprite { color: bg, custom_size: Some(Vec2::new(LANE_W, LANE_H)), ..default() },
            Transform::from_xyz(lane_x, LANE_Y, -10.0),
        ));

        for r in 0..ROWS {
            for c in 0..COLS {
                let pos = grid_to_world(side, r, c);
                let is_extra = r < FINISH_EXTRA_ROWS;
                commands.spawn((
                    Sprite {
                        color: if is_extra { Color::srgba(0.18, 0.18, 0.2, 0.5) } else { Color::srgba(0.22, 0.22, 0.24, 0.6) },
                        custom_size: Some(Vec2::new(CELL - 3.0, CELL - 3.0)),
                        ..default()
                    },
                    Transform::from_xyz(pos.x, pos.y, -9.0),
                ));
            }
        }

        let left = lane_x - LANE_W / 2.0;
        let bottom = LANE_Y - LANE_H / 2.0;
        for c in 0..=COLS {
            let x = left + c as f32 * CELL;
            commands.spawn((Sprite { color: Color::srgba(0.35, 0.35, 0.38, 0.6), custom_size: Some(Vec2::new(2.0, LANE_H)), ..default() }, Transform::from_xyz(x, LANE_Y, -8.5), GridLine));
        }
        for r in 0..=ROWS {
            let y = bottom + r as f32 * CELL;
            commands.spawn((Sprite { color: Color::srgba(0.35, 0.35, 0.38, 0.6), custom_size: Some(Vec2::new(LANE_W, 2.0)), ..default() }, Transform::from_xyz(lane_x, y, -8.5), GridLine));
        }

        commands.spawn((Sprite { color: Color::srgb(0.4, 0.4, 0.45), custom_size: Some(Vec2::new(4.0, LANE_H)), ..default() }, Transform::from_xyz(lane_x - LANE_W / 2.0, LANE_Y, -8.0)));
        commands.spawn((Sprite { color: Color::srgb(0.4, 0.4, 0.45), custom_size: Some(Vec2::new(4.0, LANE_H)), ..default() }, Transform::from_xyz(lane_x + LANE_W / 2.0, LANE_Y, -8.0)));
        commands.spawn((Sprite { color: Color::srgb(0.4, 0.4, 0.45), custom_size: Some(Vec2::new(LANE_W, 4.0)), ..default() }, Transform::from_xyz(lane_x, LANE_Y + LANE_H / 2.0, -8.0)));
        commands.spawn((Sprite { color: Color::srgb(0.3, 0.3, 0.35), custom_size: Some(Vec2::new(LANE_W, 4.0)), ..default() }, Transform::from_xyz(lane_x, LANE_Y - LANE_H / 2.0, -8.0)));

        // nadpis nad lajnou, at je jasne, ci je
        let title = if test { "TEST" } else if side == Side::Player { "TY" } else { "BOT" };
        commands.spawn((Text2d::new(title), TextFont { font_size: 22.0, ..default() }, TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)), Transform::from_xyz(lane_x, LANE_Y + LANE_H / 2.0 + 24.0, -7.0)));
    }

    // zelena cilova cara (2 radky nad spodkem) odstranena - cilem je spodni okraj lajny

    // hover highlight
    commands.spawn((Sprite { color: Color::srgba(0.9, 0.9, 0.9, 0.18), custom_size: Some(Vec2::new(CELL - 2.0, CELL - 2.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 7.0), HoverHighlight, Visibility::Hidden));
    // vybrane policko (po kliknuti/klepnuti zustane zvyraznene): zluta vypln + ramecek; pozici nastavuje selection_marker_system
    commands.spawn((
        Sprite { color: Color::srgba(1.0, 0.9, 0.2, 0.22), custom_size: Some(Vec2::new(CELL - 2.0, CELL - 2.0)), ..default() },
        Transform::from_xyz(0.0, 0.0, 7.1),
        Visibility::Hidden,
        SelectionMarker,
    )).with_children(|p| {
        let half = (CELL - 2.0) / 2.0;
        let t = 3.0;
        let c = Color::srgb(1.0, 0.9, 0.2);
        for (x, y, w, h) in [(0.0, half, CELL - 2.0, t), (0.0, -half, CELL - 2.0, t), (-half, 0.0, t, CELL - 2.0), (half, 0.0, t, CELL - 2.0)] {
            p.spawn((Sprite { color: c, custom_size: Some(Vec2::new(w, h)), ..default() }, Transform::from_xyz(x, y, 0.1)));
        }
    });

    // kruh dosahu veze (bily) a kruh dosahu po upgradu (zeleny, ukaze se pri najeti na UPGRADE)
    let ring_mesh = meshes.add(Annulus::new(0.994, 1.0));
    // kruh dosahu = shader (assets/shaders/range.wgsl) na ctverci 1x1, velikost pres scale (2 x polomer)
    let range_mat = range_mats.add(fx::RangeMaterial { params: fx::range_params(Color::srgba(1.0, 1.0, 1.0, 0.9), 100.0) });
    commands.spawn((Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))), MeshMaterial2d(range_mat), Transform::from_xyz(0.0, 0.0, 3.0), Visibility::Hidden, RangeRing));
    commands.spawn((Mesh2d(ring_mesh), MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(0.3, 1.0, 0.3, 0.8)))), Transform::from_xyz(0.0, 0.0, 3.1), Visibility::Hidden, RangeRingNext));

    // === TOP UI ===
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Px(36.0), position_type: PositionType::Absolute, top: Val::Px(SAFE_TOP), left: Val::Px(0.0), flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), padding: UiRect::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgba(0.1, 0.1, 0.12, 0.95)))).with_children(|p| {
        p.spawn((Text::new("Gold: 150"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), GoldText));
        p.spawn((Text::new("Diamonds: 0"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), DiamondText));
        p.spawn((Text::new("Zivoty: 15 | Bot: 15"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), LivesText));
        p.spawn((Text::new("Vlna: 0"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), WaveText));

    });

    // === LEFT TOP pod tim co uz tam je - odpocet + pause ===
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        top: Val::Px(42.0 + SAFE_TOP),
        left: Val::Px(8.0),
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(10.0),
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(6.0)),
        ..default()
    }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)))).with_children(|p| {
        p.spawn((Text::new("Next: 8s"), TextFont { font_size: 16.0, ..default() }, TextColor(Color::srgb(0.9,0.9,0.3)), WaveTimerText));
        p.spawn((
            Button,
            Node { width: Val::Px(70.0), height: Val::Px(30.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.35)),
            PauseButtonUI,
        )).with_children(|pp| {
            pp.spawn((Text::new("PAUSE"), TextFont { font_size: 12.0, ..default() }, TextColor(Color::WHITE)));
        });
    });

    // === vpravo nahore - prepnout pohled na lajnu soupere a zpet (nebo klavesa Tab); v testu souper neni ===
    if !test { commands.spawn((
        Button,
        Node { position_type: PositionType::Absolute, top: Val::Px(44.0 + SAFE_TOP), right: Val::Px(8.0), width: Val::Px(64.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgb(0.55, 0.2, 0.2)),
        SwitchViewButton,
    )).with_children(|p| {
        p.spawn((Text::new("SOUPER"), TextFont { font_size: 12.0, ..default() }, TextColor(Color::WHITE), TextLayout::new_with_justify(JustifyText::Center), SwitchViewLabel));
    }); }

    // === LEFT TOP pod odpoctem vlny - za kolik s prijde income (v zavorce celkovy income); v testu nema smysl ===
    if !test { commands.spawn((Node {
        position_type: PositionType::Absolute,
        	top: Val::Px(90.0 + SAFE_TOP),
        left: Val::Px(8.0),
        padding: UiRect::all(Val::Px(6.0)),
        ..default()
    }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)))).with_children(|p| {
        p.spawn((Text::new("Income in: 7s (+0)"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::srgb(0.5, 0.9, 0.5)), IncomeText));
    }); }

    // === CHEAT (vlevo nahore pod income): kazde stisknuti prida +50 do income (gold za tik); jen pro testovani ===
    // v TEST rezimu je na jeho miste VLNA: posle dalsi automatickou vlnu na vlastni lajnu
    let cheat_node = Node { position_type: PositionType::Absolute, left: Val::Px(8.0), top: Val::Px(if test { 90.0 + SAFE_TOP } else { 130.0 }), width: Val::Px(64.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() };
    if test {
        commands.spawn((Button, cheat_node, BackgroundColor(Color::srgb(0.55, 0.25, 0.6)), TestWaveButton)).with_children(|p| {
            p.spawn((Text::new("VLNA\n+1"), TextFont { font_size: 12.0, ..default() }, TextColor(Color::WHITE), TextLayout::new_with_justify(JustifyText::Center)));
        });
    } else {
        commands.spawn((Button, cheat_node, BackgroundColor(Color::srgb(0.6, 0.15, 0.6)), CheatButton)).with_children(|p| {
            p.spawn((Text::new("CHEAT\n+50"), TextFont { font_size: 11.0, ..default() }, TextColor(Color::WHITE)));
        });
    }

    // === S (vpravo dole) - otevre panel jednotek ===
    commands.spawn((
        Button,
        Node { position_type: PositionType::Absolute, right: Val::Px(16.0), bottom: Val::Px(24.0 + SAFE_BOTTOM), width: Val::Px(64.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgb(0.7, 0.3, 0.1)),
        SendToggleButton,
    )).with_children(|p| {
        p.spawn((Text::new("S"), TextFont { font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
    });

    // panel jednotek - nad S, dva sloupce po 5 viditelnych radcich, posouva se koleckem / tahem prstu (send_panel_scroll_system)
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        right: Val::Px(16.0),
        bottom: Val::Px(96.0 + SAFE_BOTTOM),
        width: Val::Px(134.0),
        height: Val::Px(344.0),
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        align_content: AlignContent::FlexStart,
        column_gap: Val::Px(6.0),
        row_gap: Val::Px(6.0),
        overflow: Overflow::scroll_y(),
        display: Display::None,
        ..default()
    }, SendPanel)).with_children(|p| {
        for kind in wave_def::units_by_cost() { // od nejlevnejsi (nove jednotky 31+ se zaradi podle ceny)
            let def = wave_def::get_unit_def(kind);
            p.spawn((
                Button,
                Node { width: Val::Px(64.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, flex_direction: FlexDirection::Column, row_gap: Val::Px(2.0), ..default() },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.22)),
                SendPanelItem(kind),
            )).with_children(|pp| {
                match def.image {
                    Some(path) => { pp.spawn((ImageNode::new(asset_server.load(path)), Node { width: Val::Px(34.0), height: Val::Px(34.0), ..default() })); }
                    None => { pp.spawn((Node { width: Val::Px(30.0), height: Val::Px(30.0), ..default() }, BackgroundColor(def.color), BorderRadius::MAX)); }
                }
                pp.spawn((Text::new(format!("{} {}g", def.name, def.cost)), TextFont { font_size: 9.0, ..default() }, TextColor(Color::WHITE)));
            });
        }
    });

    // === hlaska hlidace (dole nad tlacitky), na zacatku skryta; ukazuje ji guardian_dispatch_system ===
    commands.spawn((
        Node { position_type: PositionType::Absolute, bottom: Val::Px(100.0 + SAFE_BOTTOM), width: Val::Percent(100.0), justify_content: JustifyContent::Center, ..default() },
        Visibility::Hidden,
        GuardianMsg { timer: Timer::from_seconds(4.0, TimerMode::Once) },
    )).with_children(|p| {
        p.spawn((Node { padding: UiRect::all(Val::Px(10.0)), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)))).with_children(|pp| {
            pp.spawn((Text::new("Hlidac: HAhaha - co to zkousis kamarade? Vsichni hrajeme Fair play !"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::srgb(1.0, 0.85, 0.2))));
        });
    });

    // === okno potvrzeni prodeje (uprostred obrazovky), na zacatku skryte ===
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(50.0),
        top: Val::Percent(40.0),
        width: Val::Px(260.0),
        margin: UiRect { left: Val::Px(-130.0), ..default() },
        padding: UiRect::all(Val::Px(14.0)),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: Val::Px(12.0),
        display: Display::None,
        ..default()
    }, BackgroundColor(Color::srgba(0.05, 0.05, 0.07, 0.95)), BorderColor(Color::srgb(0.8, 0.3, 0.2)), GlobalZIndex(50), SellConfirmRoot)).with_children(|p| {
        p.spawn((Text::new(""), TextFont { font_size: 15.0, ..default() }, TextColor(Color::WHITE), SellConfirmText));
        p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
            for (label, col, yes) in [("ANO (Enter)", Color::srgb(0.75, 0.2, 0.15), true), ("NE (Esc)", Color::srgb(0.3, 0.3, 0.35), false)] {
                let mut b = row.spawn((
                    Button,
                    Node { width: Val::Px(110.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                    BackgroundColor(col),
                ));
                if yes { b.insert(SellYesButton); } else { b.insert(SellNoButton); }
                b.with_children(|bb| { bb.spawn((Text::new(label), TextFont { font_size: 13.0, ..default() }, TextColor(Color::WHITE))); });
            }
        });
    });

    // === pause menu ===
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(14.0),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.78)),
        GlobalZIndex(100),
        PauseMenuRoot,
    )).with_children(|p| {
        p.spawn((Text::new("PAUZA"), TextFont { font_size: 38.0, ..default() }, TextColor(Color::WHITE)));
        p.spawn((Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(10.0),
            ..default()
        }, PauseOptionsRoot)).with_children(|options| {
            for (label, action, color) in [
                ("POKRAČOVAT", None, Color::srgb(0.15, 0.55, 0.25)),
                ("RESTART", Some(PauseAction::Restart), Color::srgb(0.55, 0.35, 0.12)),
                ("MENU", Some(PauseAction::Menu), Color::srgb(0.25, 0.35, 0.55)),
                ("KONEC", Some(PauseAction::Exit), Color::srgb(0.6, 0.2, 0.15)),
            ] {
                let mut button = options.spawn((
                    Button,
                    Node { width: Val::Px(220.0), height: Val::Px(52.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                    BackgroundColor(color),
                ));
                if let Some(action) = action {
                    button.insert(PauseMenuButton(action));
                } else {
                    button.insert(PauseResumeButton);
                }
                button.with_children(|label_parent| {
                    label_parent.spawn((Text::new(label), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
                });
            }
        });
        p.spawn((Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(12.0),
            display: Display::None,
            ..default()
        }, PauseConfirmRoot)).with_children(|confirm| {
            confirm.spawn((Text::new("Opravdu?"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE), PauseConfirmText));
            confirm.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
                for (label, color, yes) in [
                    ("ANO", Color::srgb(0.65, 0.2, 0.15), true),
                    ("NE", Color::srgb(0.3, 0.3, 0.35), false),
                ] {
                    let mut button = row.spawn((
                        Button,
                        Node { width: Val::Px(100.0), height: Val::Px(44.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                        BackgroundColor(color),
                    ));
                    if yes { button.insert(PauseConfirmYesButton); } else { button.insert(PauseConfirmNoButton); }
                    button.with_children(|label_parent| {
                        label_parent.spawn((Text::new(label), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE)));
                    });
                }
            });
        });
    });

    // === nad tlacitky dole uprostred - staty vybrane veze / co postavi POSTAVIT ===
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(50.0),
        bottom: Val::Px(96.0 + SAFE_BOTTOM),
        width: Val::Px(240.0),
        margin: UiRect { left: Val::Px(-120.0), ..default() },
        padding: UiRect::all(Val::Px(8.0)),
        display: Display::None,
        ..default()
    }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)), TowerInfoPanel)).with_children(|p| {
        p.spawn((Text::new(""), TextFont { font_size: 11.0, ..default() }, TextColor(Color::WHITE), TowerInfoText));
    });

    // nabidka vezi k postaveni; v TEST rezimu vsech 25 vezi ve vic radcich
    let per_row = if test { 9 } else { player_deck.len().max(1) };
    let panel_w = per_row as f32 * 48.0 + (per_row as f32 - 1.0) * 6.0 + 12.0;
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            bottom: Val::Px(98.0 + SAFE_BOTTOM),
            width: Val::Px(panel_w),
            margin: UiRect { left: Val::Px(-panel_w / 2.0), ..default() },
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(6.0),
            column_gap: Val::Px(6.0),
            padding: UiRect::all(Val::Px(6.0)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        GlobalZIndex(20),
        BuildTowerPanel,
    )).with_children(|panel| {
        for &kind in &player_deck {
            let def = tower_def::get_tower_def(kind, 1);
            panel.spawn((
                Button,
                Node { width: Val::Px(48.0), height: Val::Px(42.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                BackgroundColor(def.color),
                BuildTowerButton(kind),
            )).with_children(|button| {
                button.spawn((Text::new(constants::tower_tag(kind)), TextFont { font_size: 11.0, ..default() }, TextColor(Color::WHITE)));
            });
        }
    });

    // === dole uprostred - POSTAVIT (zelena, prazdne policko) / PRODAT (cervena, vez); jen kdyz je vybrane policko ===
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(50.0),
        bottom: Val::Px(24.0 + SAFE_BOTTOM),
        margin: UiRect { left: Val::Px(-68.0), ..default() },
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(8.0),
        display: Display::None,
        ..default()
    }, TowerActionPanel)).with_children(|p| {
        p.spawn((
            Button,
            Node { width: Val::Px(64.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgb(0.15, 0.6, 0.2)),
            UpgradeButton,
        )).with_children(|pp| {
            pp.spawn((Text::new("POSTAVIT"), TextFont { font_size: 10.0, ..default() }, TextColor(Color::WHITE), UpgradeLabel));
        });
        p.spawn((
            Button,
            Node { width: Val::Px(64.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgb(0.75, 0.2, 0.15)),
            SellButton,
        )).with_children(|pp| {
            pp.spawn((Text::new("PRODAT"), TextFont { font_size: 10.0, ..default() }, TextColor(Color::WHITE), SellLabel));
        });
    });
}

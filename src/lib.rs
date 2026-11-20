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
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, autoplay_system)
        .add_systems(OnEnter(AppState::Menu), spawn_menu)
        .add_systems(OnExit(AppState::Menu), despawn_menu)
        .add_systems(Update, menu_button_system.run_if(in_state(AppState::Menu)))
        .insert_resource(RoundState::default())
        .insert_resource(MatchOver::default())
        .insert_resource(WavePaused(false))
        .insert_resource(Sides::default())
        .insert_resource(ViewSide(Side::Player))
        .insert_resource(Diamonds(0))
        .insert_resource(WaveCount(0))
        .insert_resource(InputPointer::default())
        .insert_resource(HoverCell(None))
        .insert_resource(SelectedCell(Some((ROWS / 2, COLS / 2)))) // vzdy je nejake policko vybrane (klavesnice)
        .insert_resource(HeldBuilding(None))
        .insert_resource(PanelsOpen::default())
        .insert_resource(Deck::default())
        .insert_resource(TowerDrag::default())
        .insert_resource(KeyControl::default())
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
            freeze_system,
            particle_spawn_system,
            particle_update_system.after(particle_spawn_system),
            bolt_update_system,
            send_panel_scroll_system,
            ignite_visual_system,
        ).run_if(in_state(AppState::Playing)))
        .add_systems(Update, (
            tower_shoot,
            move_bullets,
            move_creeps_maze,
            build_creep_grid.after(move_creeps_maze).before(bullet_hit), // mrizka creepu pred zasahy
            bullet_hit,
            update_debuffs,
            despawn_dead_and_count,
            round_system,
            update_ui,
            update_wave_timer.after(round_system), // UI odpocet az po ticku kola
            update_hp_display,
            update_hp_bar,
            update_outline,
            enrage_system,
            enrage_flash_system,
            heal_system,
            guardian_spawn_system,
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
            guardian_snipe_system,
            selection_marker_system,
            safe_area_system,
            cheat_button_system,
            tower_aura_system,
            switch_view_system,
            game_over_system,
            game_over_button_system,
            tower_drag_system,
            bot_system,
            special_towers_system.after(build_creep_grid),
            tower_info_system,
            opponent_hp_system,
            keyboard_cursor_system.after(tower_drag_system),
        ).run_if(in_state(AppState::Playing)))
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
    commands.insert_resource(WaveCount(0));
    commands.insert_resource(SelectedCell(Some((ROWS / 2, COLS / 2))));
    commands.insert_resource(HoverCell(None));
    commands.insert_resource(HeldBuilding(None));
    commands.insert_resource(PanelsOpen::default());
    commands.insert_resource(TowerDrag::default());
    commands.insert_resource(KeyControl::default());
    // pooly: jejich entity byly smazany, setup je vytvori znovu
    commands.insert_resource(DamageNumberPool::default());
    commands.insert_resource(BulletPool::default());
    commands.insert_resource(CreepGrid::default());
    commands.insert_resource(ParticlePool::default());
    commands.insert_resource(BoltPool::default());
    commands.insert_resource(HealRingPool::default());
}

// test na PC: s promennou prostredi AUTOPLAY=1 se preskoci menu a rovnou zacne zapas
fn autoplay_system(mut next: ResMut<NextState<AppState>>) {
    if std::env::var("AUTOPLAY").is_ok() { next.set(AppState::Playing); }
}

fn setup(
    mut commands: Commands,
    mut sides: ResMut<Sides>,
    deck: Res<Deck>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // kamera se spousti uz v menu (menu::spawn_camera)
    for st in sides.0.iter_mut() { st.path.dist = compute_path(&st.grid.blocked); }
    // balicky: hrac ma svuj z menu, bot si nahodne vybere 5 ze vsech vezi
    sides.get_mut(Side::Player).deck = deck.0.clone();
    {
        use rand::seq::SliceRandom;
        let mut all: Vec<u8> = constants::ALL_TOWERS.iter().map(|t| t.0).collect();
        all.shuffle(&mut rand::thread_rng());
        all.truncate(constants::DECK_SIZE);
        println!("Balicek hrace {:?}, bota {:?}", deck.0, all);
        sides.get_mut(Side::Bot).deck = all;
    }

    // dve lajny: hracova a bota (posunuta o SIDE_OFFSET_X); bot ma lehce cervenejsi pozadi
    for side in Side::ALL {
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
        let title = if side == Side::Player { "TY" } else { "BOT" };
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
    // duch pretahovane veze pod prstem (ikonu a pozici nastavuje tower_drag_system)
    commands.spawn((Sprite::default(), Transform::from_xyz(0.0, 0.0, 8.0), BuildPreview, Visibility::Hidden));

    // kruh dosahu veze (bily) a kruh dosahu po upgradu (zeleny, ukaze se pri najeti na UPGRADE)
    let ring_mesh = meshes.add(Annulus::new(0.994, 1.0));
    commands.spawn((Mesh2d(ring_mesh.clone()), MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(1.0, 1.0, 1.0, 0.45)))), Transform::from_xyz(0.0, 0.0, 3.0), Visibility::Hidden, RangeRing));
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

    // === vpravo nahore - prepnout pohled na lajnu soupere a zpet (nebo klavesa Tab) ===
    commands.spawn((
        Button,
        Node { position_type: PositionType::Absolute, top: Val::Px(44.0 + SAFE_TOP), right: Val::Px(8.0), width: Val::Px(64.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgb(0.55, 0.2, 0.2)),
        SwitchViewButton,
    )).with_children(|p| {
        p.spawn((Text::new("SOUPER"), TextFont { font_size: 12.0, ..default() }, TextColor(Color::WHITE), TextLayout::new_with_justify(JustifyText::Center), SwitchViewLabel));
    });

    // === LEFT TOP pod odpoctem vlny - za kolik s prijde income (v zavorce celkovy income) ===
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        	top: Val::Px(90.0 + SAFE_TOP),
        left: Val::Px(8.0),
        padding: UiRect::all(Val::Px(6.0)),
        ..default()
    }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)))).with_children(|p| {
        p.spawn((Text::new("Income in: 7s (+0)"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::srgb(0.5, 0.9, 0.5)), IncomeText));
    });

    // === CHEAT (vlevo nahore pod income): kazde stisknuti prida +50 do income (gold za tik); jen pro testovani ===
    commands.spawn((
        Button,
        Node { position_type: PositionType::Absolute, left: Val::Px(8.0), top: Val::Px(130.0), width: Val::Px(64.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgb(0.6, 0.15, 0.6)),
        CheatButton,
    )).with_children(|p| {
        p.spawn((Text::new("CHEAT\n+50"), TextFont { font_size: 11.0, ..default() }, TextColor(Color::WHITE)));
    });

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
        for kind in 1..=wave_def::UNIT_COUNT {
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

    // === hlaska hlidace (dole nad tlacitky), na zacatku skryta; ukazuje ji guardian_snipe_system ===
    commands.spawn((
        Node { position_type: PositionType::Absolute, bottom: Val::Px(100.0 + SAFE_BOTTOM), width: Val::Percent(100.0), justify_content: JustifyContent::Center, ..default() },
        Visibility::Hidden,
        GuardianMsg { timer: Timer::from_seconds(4.0, TimerMode::Once) },
    )).with_children(|p| {
        p.spawn((Node { padding: UiRect::all(Val::Px(10.0)), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)))).with_children(|pp| {
            pp.spawn((Text::new("Hlidac: HAhaha - co to zkousis kamarade? Vsichni hrajeme Fair play !"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::srgb(1.0, 0.85, 0.2))));
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

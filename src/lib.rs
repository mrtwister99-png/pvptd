use bevy::prelude::*;
mod constants;
mod components;
mod resources;
mod tower_def;
mod wave_def;
mod utils;
mod systems;
mod menu;
use constants::{COLS, ROWS, CELL, LANE_W, LANE_H, LANE_X, LANE_Y, FINISH_EXTRA_ROWS, SAFE_TOP, SAFE_BOTTOM, CAM_BASE_Y, BASE_INCOME};
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
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(AppState::Menu), spawn_menu)
        .add_systems(OnExit(AppState::Menu), despawn_menu)
        .add_systems(Update, menu_button_system.run_if(in_state(AppState::Menu)))
        .init_state::<AppState>()
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(AppState::Menu), spawn_menu)
        .add_systems(OnExit(AppState::Menu), despawn_menu)
        .add_systems(Update, menu_button_system.run_if(in_state(AppState::Menu)))
        .insert_resource(CreepTimer(Timer::from_seconds(8.0, TimerMode::Repeating)))
        .insert_resource(WavePaused(false))
        .insert_resource(EscapedCount(0))
        .insert_resource(Gold(150))
        .insert_resource(Diamonds(0))
        .insert_resource(WaveCount(0))
        .insert_resource(Income { amount: BASE_INCOME, carry: 0.0, sent: [0; 32], timer: Timer::from_seconds(7.0, TimerMode::Repeating) })
        .insert_resource(InputPointer::default())
        .insert_resource(HoverCell(None))
        .insert_resource(SelectedCell(None))
        .insert_resource(HeldBuilding(None))
        .insert_resource(PanelsOpen::default())
        .insert_resource(Grid { blocked: [[false; COLS]; ROWS] })
        .insert_resource(PathMap { dist: [[None; COLS]; ROWS] })
        .insert_resource(DamageNumberPool::default())
        .insert_resource(BulletPool::default())
        .insert_resource(CreepGrid::default())
        .insert_resource(ParticlePool::default())
        .insert_resource(BoltPool::default())
        .insert_resource(HealRingPool::default())
        .insert_resource(SafeInsets::default())
        .add_systems(OnEnter(AppState::Playing), (setup, adopt_ui_system).chain())
        .add_systems(OnEnter(AppState::Playing), (setup_damage_pool, setup_bullet_pool, setup_creep_assets, setup_particle_pool, setup_bolt_pool, setup_heal_ring_pool))
        .add_systems(Update, (
            // zadny hrac - jen mys / klik
            pointer_system.before(mouse_hover_system),
            mouse_hover_system.before(mouse_click_select_system),
            mouse_click_select_system,
            range_rings_system,
            sell_tower_input,
            upgrade_tower_input,
            keyboard_pause_system,
            pause_button_system_top_left,
            panels_input_system,
            panels_display_system,
            held_ghost_system,
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
            spawn_wave_system,
            update_ui,
            update_wave_timer.after(spawn_wave_system), // 1.2: UI timer az po ticku timeru
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
        ).run_if(in_state(AppState::Playing)))
        .run();
}

fn setup(
    mut commands: Commands,
    grid: Res<Grid>,
    mut path: ResMut<PathMap>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // kamera se spousti uz v menu (menu::spawn_camera)
    path.dist = compute_path(&grid.blocked);

    // pozadi lane
    commands.spawn((
        Sprite { color: Color::srgb(0.16, 0.16, 0.18), custom_size: Some(Vec2::new(LANE_W, LANE_H)), ..default() },
        Transform::from_xyz(LANE_X, LANE_Y, -10.0),
    ));

    for r in 0..ROWS {
        for c in 0..COLS {
            let pos = grid_to_world(r, c);
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

    let left = LANE_X - LANE_W / 2.0;
    let bottom = LANE_Y - LANE_H / 2.0;
    for c in 0..=COLS {
        let x = left + c as f32 * CELL;
        commands.spawn((Sprite { color: Color::srgba(0.35, 0.35, 0.38, 0.6), custom_size: Some(Vec2::new(2.0, LANE_H)), ..default() }, Transform::from_xyz(x, LANE_Y, -8.5), GridLine));
    }
    for r in 0..=ROWS {
        let y = bottom + r as f32 * CELL;
        commands.spawn((Sprite { color: Color::srgba(0.35, 0.35, 0.38, 0.6), custom_size: Some(Vec2::new(LANE_W, 2.0)), ..default() }, Transform::from_xyz(LANE_X, y, -8.5), GridLine));
    }

    commands.spawn((Sprite { color: Color::srgb(0.4, 0.4, 0.45), custom_size: Some(Vec2::new(4.0, LANE_H)), ..default() }, Transform::from_xyz(LANE_X - LANE_W / 2.0, LANE_Y, -8.0)));
    commands.spawn((Sprite { color: Color::srgb(0.4, 0.4, 0.45), custom_size: Some(Vec2::new(4.0, LANE_H)), ..default() }, Transform::from_xyz(LANE_X + LANE_W / 2.0, LANE_Y, -8.0)));
    commands.spawn((Sprite { color: Color::srgb(0.4, 0.4, 0.45), custom_size: Some(Vec2::new(LANE_W, 4.0)), ..default() }, Transform::from_xyz(LANE_X, LANE_Y + LANE_H / 2.0, -8.0)));
    commands.spawn((Sprite { color: Color::srgb(0.3, 0.3, 0.35), custom_size: Some(Vec2::new(LANE_W, 4.0)), ..default() }, Transform::from_xyz(LANE_X, LANE_Y - LANE_H / 2.0, -8.0)));

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
    // duch drzene budovy na kurzoru (ikonu a pozici nastavuje held_ghost_system)
    commands.spawn((Sprite::default(), Transform::from_xyz(0.0, 0.0, 8.0), BuildPreview, Visibility::Hidden));

    // kruh dosahu veze (bily) a kruh dosahu po upgradu (zeleny, ukaze se pri najeti na UPGRADE)
    let ring_mesh = meshes.add(Annulus::new(0.994, 1.0));
    commands.spawn((Mesh2d(ring_mesh.clone()), MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(1.0, 1.0, 1.0, 0.45)))), Transform::from_xyz(0.0, 0.0, 3.0), Visibility::Hidden, RangeRing));
    commands.spawn((Mesh2d(ring_mesh), MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(0.3, 1.0, 0.3, 0.8)))), Transform::from_xyz(0.0, 0.0, 3.1), Visibility::Hidden, RangeRingNext));

    // === TOP UI ===
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Px(36.0), position_type: PositionType::Absolute, top: Val::Px(SAFE_TOP), left: Val::Px(0.0), flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), padding: UiRect::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgba(0.1, 0.1, 0.12, 0.95)))).with_children(|p| {
        p.spawn((Text::new("Gold: 150"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), GoldText));
        p.spawn((Text::new("Diamonds: 0"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), DiamondText));
        p.spawn((Text::new("Proslo: 0/15"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), EscapedText));
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

    // === vpravo nahore - prazdny ctverecek (rezerva, funkce doplnime) ===
    commands.spawn((
        Button,
        Node { position_type: PositionType::Absolute, top: Val::Px(44.0 + SAFE_TOP), right: Val::Px(8.0), width: Val::Px(64.0), height: Val::Px(64.0), ..default() },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.22)),
        ReservedButton,
    ));

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

    // === B (vlevo dole) - otevre panel budov ===
    commands.spawn((
        Button,
        Node { position_type: PositionType::Absolute, left: Val::Px(16.0), bottom: Val::Px(24.0 + SAFE_BOTTOM), width: Val::Px(64.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgb(0.25, 0.35, 0.55)),
        BuildToggleButton,
    )).with_children(|p| {
        p.spawn((Text::new("B"), TextFont { font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
    });

    // panel budov - vysouva se nahoru nad B, na zacatku skryty
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        left: Val::Px(16.0),
        bottom: Val::Px(96.0 + SAFE_BOTTOM),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(6.0),
        display: Display::None,
        ..default()
    }, BuildPanel)).with_children(|p| {
        for (kind, name) in [(1u8, "ARROW"), (2, "CANNON"), (6, "SNIPER"), (3, "FROST"), (4, "FIRE"), (5, "POISON"), (7, "SHOCK")] {
            let def = tower_def::get_tower_def(kind, 1);
            p.spawn((
                Button,
                Node { width: Val::Px(64.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, flex_direction: FlexDirection::Column, ..default() },
                BackgroundColor(def.color),
                BuildPanelItem(kind),
            )).with_children(|pp| {
                pp.spawn((Text::new(format!("{}\n{}G", name, def.gold)), TextFont { font_size: 9.0, ..default() }, TextColor(Color::WHITE)));
            });
        }
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

    // === dole uprostred - UPGRADE (zelena) / PRODAT (cervena), jen kdyz je vybrana postavena vez ===
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
            pp.spawn((Text::new("UPGRADE"), TextFont { font_size: 10.0, ..default() }, TextColor(Color::WHITE), UpgradeLabel));
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

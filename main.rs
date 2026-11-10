
use bevy::prelude::*;

mod constants;
mod components;
mod resources;
mod tower_def;
mod wave_def;
mod utils;
mod systems;

use constants::{COLS, ROWS, CELL, LANE_W, LANE_H, LANE_X, LANE_Y, FINISH_EXTRA_ROWS};
use resources::*;
use components::*;
use utils::{compute_path, grid_to_world};
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Line TD - mobil+PC klik only + bullet + hp bar".into(),
                resolution: Vec2::new(1280.0_f32, 800.0_f32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(CreepTimer(Timer::from_seconds(8.0, TimerMode::Repeating)))
        .insert_resource(BuildMode(true)) // rovnou build mode zapnuty pro mobil
        .insert_resource(WavePaused(false))
        .insert_resource(EscapedCount(0))
        .insert_resource(Gold(150))
        .insert_resource(Diamonds(0))
        .insert_resource(WaveCount(0))
        .insert_resource(SelectedTower(1))
        .insert_resource(HoverCell(None))
        .insert_resource(SelectedCell(None))
        .insert_resource(CurrentShopMode(ShopMode::Normal))
        .insert_resource(PendingBuild(None))
        .insert_resource(PendingSell(None))
        .insert_resource(Grid { blocked: [[false; COLS]; ROWS] })
        .insert_resource(PathMap { dist: [[None; COLS]; ROWS] })
        .add_systems(Startup, setup)
        .add_systems(Update, (
            // zadny hrac - jen mys / klik
            mouse_hover_system,
            mouse_click_select_system,
            build_tower_input,
            sell_tower_input,
            upgrade_tower_input,
            keyboard_pause_system,
            send_wave_button_system,
            pause_button_system_top_left,
            top_right_slots_system,
        ))
        .add_systems(Update, (
            tower_shoot,
            move_bullets,
            move_creeps_maze,
            bullet_hit,
            update_debuffs,
            despawn_dead_and_count,
            spawn_wave_system,
            update_ui,
            update_wave_timer,
            update_hp_display,
            update_hp_bar,
            update_outline,
            update_damage_numbers,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    grid: Res<Grid>,
    mut path: ResMut<PathMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
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

    let finish_y = LANE_Y - LANE_H / 2.0 + FINISH_EXTRA_ROWS as f32 * CELL;
    commands.spawn((Sprite { color: Color::srgb(0.0, 0.8, 0.0), custom_size: Some(Vec2::new(LANE_W, 6.0)), ..default() }, Transform::from_xyz(LANE_X, finish_y, -7.0)));

    // hover highlight
    commands.spawn((Sprite { color: Color::srgba(0.9, 0.9, 0.9, 0.18), custom_size: Some(Vec2::new(CELL - 2.0, CELL - 2.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 7.0), HoverHighlight, Visibility::Hidden));
    commands.spawn((Sprite { color: Color::srgba(1.0, 1.0, 1.0, 0.35), custom_size: Some(Vec2::new(CELL - 6.0, CELL - 6.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 8.0), BuildPreview, Visibility::Visible));
    commands.spawn((Sprite { color: Color::srgba(1.0, 1.0, 1.0, 0.08), custom_size: Some(Vec2::new(100.0, 100.0)), ..default() }, Transform::from_xyz(0.0, 0.0, 7.5), RangePreview, Visibility::Visible));

    // === TOP UI ===
    commands.spawn((Node { width: Val::Percent(100.0), height: Val::Px(36.0), position_type: PositionType::Absolute, top: Val::Px(0.0), left: Val::Px(0.0), flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), padding: UiRect::all(Val::Px(8.0)), ..default() }, BackgroundColor(Color::srgba(0.1, 0.1, 0.12, 0.95)))).with_children(|p| {
        p.spawn((Text::new("Gold: 150"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), GoldText));
        p.spawn((Text::new("Diamonds: 0"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), DiamondText));
        p.spawn((Text::new("Proslo: 0/15"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), EscapedText));
        p.spawn((Text::new("Vlna: 0"), TextFont { font_size: 14.0, ..default() }, TextColor(Color::WHITE), WaveText));
    });

    // === LEFT TOP pod tim co uz tam je - odpocet + pause ===
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        top: Val::Px(42.0),
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

    // === BOTTOM RIGHT - SEND WAVE ===
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(16.0),
        right: Val::Px(16.0),
        ..default()
    },)).with_children(|p| {
        p.spawn((
            Button,
            Node { width: Val::Px(130.0), height: Val::Px(54.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
            BackgroundColor(Color::srgb(0.15, 0.6, 0.2)),
            SendWaveButton,
        )).with_children(|pp| {
            pp.spawn((Text::new("SEND WAVE"), TextFont { font_size: 15.0, ..default() }, TextColor(Color::WHITE)));
        });
    });

    // === TOP RIGHT - 1 ctverecek + pod nej 6 dalsich (celkem 7 slotu) ===
    commands.spawn((Node {
        position_type: PositionType::Absolute,
        top: Val::Px(8.0),
        right: Val::Px(8.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(6.0),
        padding: UiRect::all(Val::Px(6.0)),
        ..default()
    }, BackgroundColor(Color::srgba(0.0,0.0,0.0,0.5)), TopRightSlots)).with_children(|p| {
        for i in 1..=7 {
            let col = match i {
                1 => Color::srgb(0.9, 0.15, 0.15),
                2 => Color::srgb(0.9, 0.85, 0.15),
                3 => Color::srgb(0.2, 0.6, 1.0),
                4 => Color::srgb(1.0, 0.3, 0.1),
                5 => Color::srgb(0.6, 0.9, 1.0),
                6 => Color::srgb(0.4, 0.4, 0.45),
                7 => Color::srgb(0.35, 0.35, 0.38),
                _ => Color::srgb(0.5,0.5,0.5),
            };
            let label = if i <=5 { format!("{}", match i {1=>"ARROW",2=>"CANNON",3=>"FROST",4=>"FIRE",5=>"ELE",_=>""}) } else { format!("SLOT {}", i) };
            p.spawn((
                Button,
                Node { width: Val::Px(64.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, flex_direction: FlexDirection::Column, ..default() },
                BackgroundColor(col),
                TopRightSlot(i),
            )).with_children(|pp| {
                pp.spawn((Text::new(label), TextFont { font_size: 9.0, ..default() }, TextColor(Color::WHITE)));
            });
        }
    });

    // build stats panel
    commands.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), top: Val::Percent(14.0), margin: UiRect { left: Val::Px(-220.0), top: Val::Px(0.0), ..default() }, padding: UiRect::all(Val::Px(10.0)), flex_direction: FlexDirection::Column, row_gap: Val::Px(4.0), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)), BuildStatsPanel, Visibility::Visible)).with_children(|p| {
        p.spawn((Text::new("Klik na mrizku = postavit | I/O/P/L/K vyber | U=upgrade N=prodat"), TextFont { font_size: 12.0, ..default() }, TextColor(Color::WHITE), BuildStatsText));
    });

    // hint dole
    commands.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(12.0), bottom: Val::Px(10.0), ..default() },)).with_children(|p| {
        p.spawn((Text::new("B=build toggle | I/O/P/L/K | U upgrade N sell | SPACE pause | SEND WAVE vpravo dole | kolecka + HP bar + rust per wave + bullet.png"), TextFont { font_size: 8.0, ..default() }, TextColor(Color::srgb(0.55, 0.55, 0.55))));
    });
}

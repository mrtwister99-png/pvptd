use bevy::prelude::*;
use crate::constants::CAM_BASE_Y;

// stav aplikace: nejdriv menu, po stisku HRAT hra
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    Playing,
}

#[derive(Component)]
pub struct MenuRoot;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuAction { Play, Leaderboard, Settings, Exit }

#[derive(Component)]
pub struct MenuButton(pub MenuAction);

// kamera je spolecna pro menu i hru (UI ji potrebuje uz v menu)
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, CAM_BASE_Y, 0.0)));
}

// hlavni menu: nadpis + 4 tlacitka (funguje jen HRAT, ostatni zatim jen na oko)
pub fn spawn_menu(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.1)),
        MenuRoot,
    )).with_children(|p| {
        p.spawn((Text::new("LINE TD"), TextFont { font_size: 48.0, ..default() }, TextColor(Color::srgb(1.0, 0.85, 0.2))));
        for (label, action, col) in [
            ("HRAT", MenuAction::Play, Color::srgb(0.15, 0.6, 0.2)),
            ("ZEBRICEK", MenuAction::Leaderboard, Color::srgb(0.25, 0.3, 0.45)),
            ("NASTAVENI", MenuAction::Settings, Color::srgb(0.25, 0.3, 0.45)),
            ("EXIT", MenuAction::Exit, Color::srgb(0.6, 0.2, 0.15)),
        ] {
            p.spawn((
                Button,
                Node { width: Val::Px(220.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                BackgroundColor(col),
                MenuButton(action),
            )).with_children(|pp| {
                pp.spawn((Text::new(label), TextFont { font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
            });
        }
    });
}

// po opusteni menu se UI menu smaze
pub fn despawn_menu(mut commands: Commands, q: Query<Entity, With<MenuRoot>>) {
    for e in q.iter() { commands.entity(e).despawn_recursive(); }
}

// klepnuti na tlacitko menu: HRAT spusti hru, ostatni zatim nic nedelaji
pub fn menu_button_system(
    q: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next: ResMut<NextState<AppState>>,
) {
    for (interaction, button) in q.iter() {
        if *interaction != Interaction::Pressed { continue; }
        match button.0 {
            MenuAction::Play => next.set(AppState::Playing),
            _ => println!("Menu: tlacitko zatim bez funkce"),
        }
    }
}
use bevy::prelude::*;
use crate::constants::CAM_BASE_Y;

// stav aplikace: nejdriv menu, po stisku HRAT hra
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    Playing,
    DeckSelect, // vyber balicku 5 vezi
    Restarting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseAction { Restart, Menu, Exit }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseOutcome { Restart, Menu, Exit }

pub fn resolve_pause_confirmation(action: PauseAction, confirmed: bool) -> Option<PauseOutcome> {
    if !confirmed { return None; }
    Some(match action {
        PauseAction::Restart => PauseOutcome::Restart,
        PauseAction::Menu => PauseOutcome::Menu,
        PauseAction::Exit => PauseOutcome::Exit,
    })
}

#[derive(Resource, Default)]
pub struct PendingPauseAction(pub Option<PauseAction>);

#[derive(Component)]
pub struct MenuRoot;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MenuAction { Play, Deck, Leaderboard, Settings, Exit, PlayBot, PlayTest, Back }

#[derive(Component)]
pub struct MenuButton(pub MenuAction);

// hlavni tlacitka menu / vyber rezimu po HRAT (prepina se Display)
#[derive(Component)]
pub struct MenuMainList;
#[derive(Component)]
pub struct MenuModeList;

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
        let list = |display| Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(16.0), display, ..default() };
        p.spawn((list(Display::Flex), MenuMainList)).with_children(|p| {
            for (label, action, col) in [
                ("HRAT", MenuAction::Play, Color::srgb(0.15, 0.6, 0.2)),
                ("BALICEK", MenuAction::Deck, Color::srgb(0.55, 0.45, 0.15)),
                ("ZEBRICEK", MenuAction::Leaderboard, Color::srgb(0.25, 0.3, 0.45)),
                ("NASTAVENI", MenuAction::Settings, Color::srgb(0.25, 0.3, 0.45)),
                ("EXIT", MenuAction::Exit, Color::srgb(0.6, 0.2, 0.15)),
            ] { menu_button(p, label, action, col); }
        });
        // po HRAT: proti botovi, nebo testovaci hriste
        p.spawn((list(Display::None), MenuModeList)).with_children(|p| {
            for (label, action, col) in [
                ("VS BOT", MenuAction::PlayBot, Color::srgb(0.15, 0.6, 0.2)),
                ("TEST", MenuAction::PlayTest, Color::srgb(0.55, 0.25, 0.6)),
                ("ZPET", MenuAction::Back, Color::srgb(0.3, 0.3, 0.35)),
            ] { menu_button(p, label, action, col); }
        });
    });
}

fn menu_button(p: &mut ChildBuilder, label: &str, action: MenuAction, col: Color) {
    p.spawn((
        Button,
        Node { width: Val::Px(220.0), height: Val::Px(64.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(col),
        MenuButton(action),
    )).with_children(|pp| {
        pp.spawn((Text::new(label), TextFont { font_size: 22.0, ..default() }, TextColor(Color::WHITE)));
    });
}

// po opusteni menu se UI menu smaze
pub fn despawn_menu(mut commands: Commands, q: Query<Entity, With<MenuRoot>>) {
    for e in q.iter() { commands.entity(e).despawn_recursive(); }
}

// klepnuti na tlacitko menu: HRAT ukaze vyber rezimu (VS BOT / TEST), BALICEK vyber vezi
pub fn menu_button_system(
    q: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
    mut next: ResMut<NextState<AppState>>,
    mut mode: ResMut<crate::resources::GameMode>,
    mut q_main: Query<&mut Node, (With<MenuMainList>, Without<MenuModeList>)>,
    mut q_modes: Query<&mut Node, (With<MenuModeList>, Without<MenuMainList>)>,
) {
    let mut show_modes = |on: bool| {
        for mut n in q_main.iter_mut() { n.display = if on { Display::None } else { Display::Flex }; }
        for mut n in q_modes.iter_mut() { n.display = if on { Display::Flex } else { Display::None }; }
    };
    for (interaction, button) in q.iter() {
        if *interaction != Interaction::Pressed { continue; }
        match button.0 {
            MenuAction::Play => show_modes(true),
            MenuAction::Back => show_modes(false),
            MenuAction::PlayBot => { *mode = crate::resources::GameMode::VsBot; next.set(AppState::Playing); }
            MenuAction::PlayTest => { *mode = crate::resources::GameMode::Test; next.set(AppState::Playing); }
            MenuAction::Deck => next.set(AppState::DeckSelect),
            _ => println!("Menu: tlacitko zatim bez funkce"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmed_pause_actions_return_the_requested_outcome() {
        assert_eq!(
            resolve_pause_confirmation(PauseAction::Restart, true),
            Some(PauseOutcome::Restart)
        );
        assert_eq!(
            resolve_pause_confirmation(PauseAction::Menu, true),
            Some(PauseOutcome::Menu)
        );
        assert_eq!(
            resolve_pause_confirmation(PauseAction::Exit, true),
            Some(PauseOutcome::Exit)
        );
    }

    #[test]
    fn declined_pause_action_does_not_change_game_flow() {
        assert_eq!(
            resolve_pause_confirmation(PauseAction::Restart, false),
            None
        );
    }
}
// Vyber balicku: hrac si z menu (tlacitko BALICEK) vybere 4 Basic a 2 Special veze.
// Ve hre hrac vybere vez z balicku a postavi ji na vybrane policko.
use bevy::prelude::*;
use crate::constants::{ALL_TOWERS, DECK_SIZE, tower_icon_path};
use crate::menu::AppState;
use crate::resources::Deck;
use crate::tower_def::get_tower_def;

#[derive(Component)] pub struct DeckRoot;
#[derive(Component)] pub struct DeckTowerButton(pub u8);
#[derive(Component)] pub struct DeckDoneButton;
#[derive(Component)] pub struct DeckCounterText;
#[derive(Component)] pub struct DeckInfoText;

// popis veze pro vyber balicku: co dela + cisla na R1 a R7
fn deck_info(kind: u8) -> String {
    let name = ALL_TOWERS.iter().find(|t| t.0 == kind).map_or("?", |t| t.1);
    let mut s = format!("{}\n{}", name, crate::special::describe(kind, 1));
    let max_rank = crate::constants::tower_rank_cap(kind);
    for rank in [1u32, max_rank] {
        let d = get_tower_def(kind, rank);
        if d.dmg_max > 0.0 {
            s.push_str(&format!("\nR{}: poskozeni {:.0}-{:.0}, {:.2} utoku/s, dosah {:.0}", rank, d.dmg_min, d.dmg_max, 1.0 / d.interval.max(0.1), d.range));
        }
        if rank == max_rank {
            s.push_str(&format!("\nR{}: {}", rank, crate::special::describe(kind, rank)));
        }
    }
    s
}

pub fn spawn_deck_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(12.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.08, 0.1)),
        DeckRoot,
    )).with_children(|p| {
        p.spawn((Text::new("BALICEK"), TextFont { font_size: 36.0, ..default() }, TextColor(Color::srgb(1.0, 0.85, 0.2))));
        p.spawn((Text::new(format!("Vyber {} Basic + {} Special", crate::constants::DECK_BASIC_SIZE, crate::constants::DECK_SPECIAL_SIZE)), TextFont { font_size: 16.0, ..default() }, TextColor(Color::WHITE), DeckCounterText));
        // mrizka vezi (zalamuje se podle sirky obrazovky)
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(10.0),
            row_gap: Val::Px(10.0),
            max_width: Val::Px(470.0),
            ..default()
        }).with_children(|g| {
            for (kind, name) in ALL_TOWERS {
                let def = get_tower_def(kind, 1);
                g.spawn((
                    Button,
                    Node { width: Val::Px(84.0), height: Val::Px(80.0), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: Val::Px(4.0), border: UiRect::all(Val::Px(3.0)), ..default() },
                    BackgroundColor(Color::srgb(0.18, 0.18, 0.2)),
                    BorderColor(Color::NONE),
                    DeckTowerButton(kind),
                )).with_children(|b| {
                    match tower_icon_path(kind) {
                        Some(path) => { b.spawn((ImageNode::new(asset_server.load(path)), Node { width: Val::Px(40.0), height: Val::Px(40.0), ..default() })); }
                        None => {
                            b.spawn((Node { width: Val::Px(36.0), height: Val::Px(36.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(def.color))).with_children(|sq| {
                                sq.spawn((Text::new(crate::constants::tower_tag(kind)), TextFont { font_size: 11.0, ..default() }, TextColor(Color::BLACK)));
                            });
                        }
                    }
                    b.spawn((Text::new(name), TextFont { font_size: 10.0, ..default() }, TextColor(Color::WHITE)));
                });
            }
        });
        // popis veze, na kterou ukazujes / klepnes
        p.spawn((
            Node { width: Val::Px(440.0), min_height: Val::Px(64.0), padding: UiRect::all(Val::Px(8.0)), ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        )).with_children(|b| {
            b.spawn((Text::new("Najed / klepni na vez pro popis"), TextFont { font_size: 12.0, ..default() }, TextColor(Color::srgb(0.85, 0.85, 0.85)), DeckInfoText));
        });
        p.spawn((
            Button,
            Node { width: Val::Px(200.0), height: Val::Px(56.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::srgb(0.15, 0.6, 0.2)),
            DeckDoneButton,
        )).with_children(|b| {
            b.spawn((Text::new("HOTOVO"), TextFont { font_size: 20.0, ..default() }, TextColor(Color::WHITE)));
        });
    });
}

pub fn despawn_deck_screen(mut commands: Commands, q: Query<Entity, With<DeckRoot>>) {
    for e in q.iter() { commands.entity(e).despawn_recursive(); }
}

// klepnuti na vez = pridat / odebrat z balicku; HOTOVO vyzaduje 4 Basic a 2 Special
pub fn deck_input_system(
    q_towers: Query<(&Interaction, &DeckTowerButton), Changed<Interaction>>,
    q_done: Query<&Interaction, (Changed<Interaction>, With<DeckDoneButton>)>,
    mut deck: ResMut<Deck>,
    mut next: ResMut<NextState<AppState>>,
    mut q_info: Query<&mut Text, With<DeckInfoText>>,
) {
    for (i, b) in q_towers.iter() {
        // najeti mysi nebo klepnuti = ukazat popis veze
        if *i != Interaction::None {
            let s = deck_info(b.0);
            for mut t in q_info.iter_mut() { if t.0 != s { t.0 = s.clone(); } }
        }
        if *i != Interaction::Pressed { continue; }
        if let Some(pos) = deck.0.iter().position(|&k| k == b.0) { deck.0.remove(pos); }
        else if deck.0.len() < DECK_SIZE {
            let special = crate::constants::is_special_tower(b.0);
            let count = deck.0.iter().filter(|&&kind| crate::constants::is_special_tower(kind) == special).count();
            let limit = if special { crate::constants::DECK_SPECIAL_SIZE } else { crate::constants::DECK_BASIC_SIZE };
            if count < limit { deck.0.push(b.0); }
        }
    }
    let basic = deck.0.iter().filter(|&&kind| !crate::constants::is_special_tower(kind)).count();
    let special = deck.0.iter().filter(|&&kind| crate::constants::is_special_tower(kind)).count();
    if q_done.iter().any(|i| *i == Interaction::Pressed)
        && basic == crate::constants::DECK_BASIC_SIZE
        && special == crate::constants::DECK_SPECIAL_SIZE
    {
        next.set(AppState::Menu);
    }
}

// zvyrazneni vybranych vezi, pocitadlo a ztmaveni HOTOVO, dokud neni balicek plny
pub fn deck_display_system(
    deck: Res<Deck>,
    mut q_towers: Query<(&DeckTowerButton, &mut BorderColor, &mut BackgroundColor), Without<DeckDoneButton>>,
    mut q_done: Query<&mut BackgroundColor, With<DeckDoneButton>>,
    mut q_text: Query<&mut Text, With<DeckCounterText>>,
) {
    for (b, mut border, mut bg) in q_towers.iter_mut() {
        let on = deck.0.contains(&b.0);
        border.0 = if on { Color::srgb(1.0, 0.85, 0.2) } else { Color::NONE };
        bg.0 = if on { Color::srgb(0.28, 0.26, 0.15) } else { Color::srgb(0.18, 0.18, 0.2) };
    }
    let basic = deck.0.iter().filter(|&&kind| !crate::constants::is_special_tower(kind)).count();
    let special = deck.0.iter().filter(|&&kind| crate::constants::is_special_tower(kind)).count();
    let full = basic == crate::constants::DECK_BASIC_SIZE && special == crate::constants::DECK_SPECIAL_SIZE;
    for mut bg in q_done.iter_mut() { bg.0 = if full { Color::srgb(0.15, 0.6, 0.2) } else { Color::srgb(0.1, 0.2, 0.12) }; }
    let s = format!("Vybrano {}/{} Basic, {}/{} Special", basic, crate::constants::DECK_BASIC_SIZE, special, crate::constants::DECK_SPECIAL_SIZE);
    for mut t in q_text.iter_mut() { if t.0 != s { t.0 = s.clone(); } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_tower_info_shows_its_actual_top_rank_and_ability() {
        let info = deck_info(crate::special::DOGE_MINER);
        assert!(info.contains("R3:"));
        assert!(!info.contains("R7:"));
        assert!(info.contains("zdvoji gold"));
    }
}

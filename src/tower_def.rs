use bevy::prelude::*;

#[derive(Clone)]
pub struct TowerDef {
    pub name: &'static str,
    pub gold: i32,
    pub dmg_min: f32,
    pub dmg_max: f32,
    pub range: f32,
    pub interval: f32,
    pub splash: f32,
    pub slow_per: f32,
    pub slow_max: f32,
    pub push_chance: f32,
    pub push_dist: f32,
    pub dot_dps: f32,
    pub dot_dur: f32,
    pub chain: u32,
    pub color: Color,
}

pub fn get_tower_def(kind: u8, level: u32) -> TowerDef {
    let mut def = base_tower_def(kind);
    let special = crate::constants::is_special_tower(kind);
    let level = level.clamp(1, crate::constants::tower_rank_cap(kind));
    let rank = level as usize - 1;
    let (damage_mul, interval_mul, range_bonus) = if special {
        (
            [1.0, 1.55, 2.35][rank],
            [1.0, 0.90, 0.80][rank],
            [0.0, 1.0, 2.0][rank],
        )
    } else {
        (
            [1.0, 1.30, 1.65, 2.10, 2.70, 3.45, 4.40][rank],
            [1.0, 0.96, 0.92, 0.87, 0.82, 0.77, 0.72][rank],
            [0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 1.5][rank],
        )
    };

    def.dmg_min *= damage_mul;
    def.dmg_max *= damage_mul;
    def.dot_dps *= damage_mul;
    def.interval *= interval_mul;
    def.range = (def.range + crate::constants::CELL * range_bonus)
        * crate::constants::TOWER_RANGE_SCALE;
    def.splash *= crate::constants::TOWER_RANGE_SCALE;
    if kind == crate::special::TESLA {
        def.chain = if level >= 7 { 4 } else { 2 }; // + prvni zasah = 3 cile, na R7 5 cilu (GDD 7)
    } else if kind == crate::special::KAKTUS && level >= 7 {
        def.dot_dps = 10.0;
    } else if kind == crate::special::PEPA && level >= 7 {
        def.chain = 2;
    }
    def
}

fn base_tower_def(kind: u8) -> TowerDef {
    let cell = crate::constants::CELL;
    let make = |name, gold, damage, interval, range, splash, slow, dot_dps, dot_dur, chain, color| {
        TowerDef {
            name,
            gold,
            dmg_min: damage,
            dmg_max: damage,
            range: range * cell,
            interval,
            splash: splash * cell,
            slow_per: slow,
            slow_max: slow,
            push_chance: 0.0,
            push_dist: 0.0,
            dot_dps,
            dot_dur,
            chain,
            color,
        }
    };

    use Color as C;
    match kind {
        1 => make("Bramboromet", 50, 10.0, 1.00, 4.0, 0.8, 0.0, 0.0, 0.0, 0, C::srgb(0.9, 0.6, 0.2)),
        2 => make("Decko s Nerfkou", 40, 4.0, 0.33, 3.5, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.9, 0.3, 0.55)),
        3 => make("Babicka Bozenka", 70, 24.0, 0.85, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.7, 0.45, 0.25)),
        4 => make("WiFi Router Lagovac", 80, 18.0, 0.90, 5.0, 0.0, 0.12, 0.0, 0.0, 0, C::srgb(0.2, 0.8, 0.9)),
        5 => make("Pichlavy Karel Kaktus", 85, 14.0, 1.00, 3.0, 0.0, 0.0, 2.0, 3.0, 0, C::srgb(0.2, 0.7, 0.25)),
        6 => make("Toustovac 3000", 110, 15.0, 0.80, 3.5, 0.0, 0.0, 8.0, 3.0, 0, C::srgb(1.0, 0.35, 0.1)),
        7 => make("Muchomurka", 100, 8.0, 0.90, 4.0, 0.0, 0.0, 2.0, 5.0, 0, C::srgb(0.3, 0.75, 0.3)),
        8 => make("Mrazak Lednicka", 120, 10.0, 1.10, 4.0, 0.0, 0.25, 0.0, 0.0, 0, C::srgb(0.3, 0.7, 1.0)),
        9 => make("Zasuvka Tesla", 150, 18.0, 1.10, 4.5, 0.0, 0.0, 0.0, 0.0, 2, C::srgb(0.95, 0.85, 0.2)),
        10 => make("Franta Sniper z Okna", 140, 40.0, 2.00, 9.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.4, 0.5, 0.95)),
        11 => make("Katapult Kamen", 180, 60.0, 2.85, 6.0, 1.3, 0.0, 0.0, 0.0, 0, C::srgb(0.55, 0.4, 0.25)),
        12 => {
            let mut def = make("DJ Bass Drop", 160, 25.0, 1.66, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.8, 0.3, 0.9));
            def.push_chance = 1.0;
            def.push_dist = cell * 0.7;
            def
        }
        13 => make("Pepa z Hospody", 90, 12.0, 1.40, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.65, 0.4, 0.2)),
        14 => make("Kasicka Bankomat", 130, 5.0, 1.25, 3.5, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.95, 0.8, 0.15)),
        15 => make("Danovy Urad", 140, 28.0, 1.20, 6.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.55, 0.45, 0.9)),
        16 => make("Chuck Norris", 400, 120.0, 1.25, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.9, 0.2, 0.2)),
        17 => make("Tchyne", 420, 90.0, 1.66, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.7, 0.2, 0.55)),
        18 => make("Shrek Bazina", 350, 80.0, 1.40, 4.0, 0.0, 0.6, 0.0, 0.0, 0, C::srgb(0.2, 0.55, 0.2)),
        19 => make("Gigachad Gym", 380, 60.0, 1.00, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.9, 0.45, 0.2)),
        20 => make("Doge Miner", 300, 40.0, 1.00, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.9, 0.8, 0.2)),
        21 => make("Trollface", 340, 70.0, 1.10, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.65, 0.8, 0.3)),
        22 => make("Haxor", 360, 45.0, 1.00, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.2, 0.9, 0.8)),
        23 => make("Karen Manazerka", 310, 30.0, 1.00, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.95, 0.55, 0.7)),
        24 => make("Cerna Dira do Brna", 450, 100.0, 2.00, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.25, 0.15, 0.4)),
        25 => make("Admin Banhammer", 400, 180.0, 2.50, 4.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.7, 0.7, 0.75)),
        _ => make("Unknown", 999, 1.0, 1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0, C::srgb(0.5, 0.5, 0.5)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roster_contains_the_fifteen_basic_and_ten_special_gdd_towers() {
        let expected = [
            ("BRAMBOROMET", 50, 10.0, 1.00, 4.0),
            ("NERFKA", 40, 4.0, 0.33, 3.5),
            ("BABICKA", 70, 24.0, 0.85, 4.0),
            ("ROUTER", 80, 18.0, 0.90, 5.0),
            ("KAKTUS", 85, 14.0, 1.00, 3.0),
            ("TOUSTOVAC", 110, 15.0, 0.80, 3.5),
            ("MUCHOMURKA", 100, 8.0, 0.90, 4.0),
            ("MRAZAK", 120, 10.0, 1.10, 4.0),
            ("TESLA", 150, 18.0, 1.10, 4.5),
            ("SNIPER", 140, 40.0, 2.00, 9.0),
            ("KATAPULT", 180, 60.0, 2.85, 6.0),
            ("DJ", 160, 25.0, 1.66, 4.0),
            ("PEPA", 90, 12.0, 1.40, 4.0),
            ("KASICKA", 130, 5.0, 1.25, 3.5),
            ("DANOVY URAD", 140, 28.0, 1.20, 6.0),
            ("CHUCK NORRIS", 400, 120.0, 1.25, 4.0),
            ("TCHYNE", 420, 90.0, 1.66, 4.0),
            ("SHREK BAZINA", 350, 80.0, 1.40, 4.0),
            ("GIGACHAD GYM", 380, 60.0, 1.00, 4.0),
            ("DOGE MINER", 300, 40.0, 1.00, 4.0),
            ("TROLLFACE", 340, 70.0, 1.10, 4.0),
            ("HAXOR", 360, 45.0, 1.00, 4.0),
            ("KAREN MANAZERKA", 310, 30.0, 1.00, 4.0),
            ("CERNA DIRA DO BRNA", 450, 100.0, 2.00, 4.0),
            ("ADMIN BANHAMMER", 400, 180.0, 2.50, 4.0),
        ];
        assert_eq!(crate::constants::ALL_TOWERS.len(), expected.len());
        for ((kind, actual), (name, cost, damage, interval, range_cells)) in
            crate::constants::ALL_TOWERS.iter().zip(expected)
        {
            assert_eq!(*actual, name);
            let def = get_tower_def(*kind, 1);
            assert_eq!(def.gold, cost);
            assert!((def.dmg_min - damage).abs() < 0.001);
            assert!((def.interval - interval).abs() < 0.001);
            assert!((def.range - range_cells * crate::constants::CELL * crate::constants::TOWER_RANGE_SCALE).abs() < 0.001);
        }
    }

    #[test]
    fn basic_upgrades_use_the_design_damage_and_interval_multipliers() {
        let base = get_tower_def(1, 1);
        let upgraded = get_tower_def(1, 2);

        assert!((upgraded.dmg_min - base.dmg_min * 1.30).abs() < 0.001);
        assert!((upgraded.dmg_max - base.dmg_max * 1.30).abs() < 0.001);
        assert!((upgraded.interval - base.interval * 0.96).abs() < 0.001);
        assert!((upgraded.range - base.range).abs() < 0.001);
    }

    #[test]
    fn rank_seven_uses_the_design_damage_multiplier() {
        let base = get_tower_def(1, 1);
        let upgraded = get_tower_def(1, 7);

        assert!((upgraded.dmg_min - base.dmg_min * 4.40).abs() < 0.001);
        assert!((upgraded.dmg_max - base.dmg_max * 4.40).abs() < 0.001);
        assert!((upgraded.interval - base.interval * 0.72).abs() < 0.001);
    }

    #[test]
    fn rank_seven_tesla_chains_to_five_targets_and_kaktus_bleeds_for_ten_dps() {
        assert_eq!(get_tower_def(crate::special::TESLA, 7).chain, 4);
        assert!((get_tower_def(crate::special::KAKTUS, 7).dot_dps - 10.0).abs() < 0.001);
    }
}

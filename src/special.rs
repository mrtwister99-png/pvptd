pub const BRAMBOROMET: u8 = 1;
pub const NERFKA: u8 = 2;
pub const BABICKA: u8 = 3;
pub const ROUTER: u8 = 4;
pub const KAKTUS: u8 = 5;
pub const TOUSTOVAC: u8 = 6;
pub const MUCHOMURKA: u8 = 7;
pub const MRAZAK: u8 = 8;
pub const TESLA: u8 = 9;
pub const SNIPER: u8 = 10;
pub const KATAPULT: u8 = 11;
pub const DJ: u8 = 12;
pub const PEPA: u8 = 13;
pub const KASICKA: u8 = 14;
pub const DANOVY_URAD: u8 = 15;
pub const CHUCK_NORRIS: u8 = 16;
pub const TCHYNE: u8 = 17;
pub const SHREK: u8 = 18;
pub const GIGACHAD: u8 = 19;
pub const DOGE_MINER: u8 = 20;
pub const TROLLFACE: u8 = 21;
pub const HAXOR: u8 = 22;
pub const KAREN: u8 = 23;
pub const BLACK_HOLE: u8 = 24;
pub const BANHAMMER: u8 = 25;

pub fn shoots(kind: u8) -> bool {
    (1..=25).contains(&kind)
        && !matches!(kind, KASICKA | GIGACHAD | DOGE_MINER)
}

pub fn special_ability_multiplier(rank: u32) -> f32 {
    [1.0, 1.20, 1.45][(rank.clamp(1, 3) - 1) as usize]
}

pub fn describe(kind: u8, rank: u32) -> String {
    match kind {
        BRAMBOROMET => if rank >= 7 { "Splash, 15% sance na stun 0,5 s".into() } else { "Vybusny splash".into() },
        NERFKA => "+5% rychlost strelby za kazdou sousedni Nerfku".into(),
        BABICKA => "Kazdy 6. zasah udeli dvojnasobne poskozeni a stun".into(),
        ROUTER => if rank >= 7 { "Slow 12%, na R7 disconnect na 1,5 s".into() } else { "Slow 12%".into() },
        KAKTUS => if rank >= 7 { "Snizuje armor, bleed az 10 dmg/s".into() } else { "Snizuje armor a zpusobuje bleed".into() },
        TOUSTOVAC => if rank >= 7 { "Burn 8 dmg/s na 3 s, smrtici exploze na R7".into() } else { "Burn 8 dmg/s na 3 s".into() },
        MUCHOMURKA => "Stackujici jed; 5 stacku zpusobi 2 s halucinaci".into(),
        MRAZAK => if rank >= 7 { "Slow, freeze 25% na 1,2 s, aura -10% AS".into() } else { "Zpomaluje a muze zmrazit".into() },
        TESLA => format!("Chain na {} cilu; R7 muze omracit skupinu", if rank >= 7 { 5 } else { 3 }),
        SNIPER => if rank >= 7 { "40% kritika x4, poprava malych pod 20% HP".into() } else { "20% kritika x3".into() },
        KATAPULT => if rank >= 7 { "Tezky splash, 20% sance na dvojity zasah".into() } else { "Tezky splash".into() },
        DJ => "Knockback; kazdych 10 s DROP odhodi creepy a udeli 100 dmg".into(),
        PEPA => if rank >= 7 { "Opilost; R7 haze 3 flasky".into() } else { "Opilost muze obratit smer pohybu".into() },
        KASICKA => "Pridava gold kazdou vlnu a vyplatu kazdych 5 vln".into(),
        DANOVY_URAD => "Cil dostava +8% poskozeni na 2 s, vypne heal a stit".into(),
        CHUCK_NORRIS => if rank >= 3 { "True damage, poprava pod 30% HP, boss -20% max HP".into() } else { "True damage, poprava pod 30% HP".into() },
        TCHYNE => if rank >= 3 { "Kritika, snizuje max HP, boss stun 3 s".into() } else { "Kritika, snizuje max HP".into() },
        SHREK => if rank >= 3 { "Slow 60%; jednou za vlnu vrati creepy na start".into() } else { "Slow 60%".into() },
        GIGACHAD => if rank >= 3 { "Aura: veze +20% dmg a +8% rychlost".into() } else { "Aura: veze +12% dmg v dosahu 2 poli".into() },
        DOGE_MINER => if rank >= 3 { "Gold za vlnu; jednou za hru zdvoji gold na 10 s".into() } else { "Gold za kazdou vlnu".into() },
        TROLLFACE => if rank >= 3 { "Teleportuje dozadu; R3 prohodi nejsilnejsi a nejslabsi".into() } else { "25% sance teleportovat dozadu o 2 pole".into() },
        HAXOR => if rank >= 3 { "15% hack; jednou za vlnu zmrazi vsechny na 2 s".into() } else { "15% sance hacknout creepa".into() },
        KAREN => if rank >= 3 { "Boss -15% rychlost, silence, -10% max HP".into() } else { "Boss -15% utocna rychlost a rychlost, silence".into() },
        BLACK_HOLE => if rank >= 3 { "Vtahuje creepy; 10% sance teleportovat k souperi s 2x HP".into() } else { "Vtahuje creepy do stredu".into() },
        BANHAMMER => "Kazdy 5. zasah odebere 12% HP (bossum 2%)".into(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn economy_and_aura_towers_do_not_fire_projectiles() {
        assert!(!shoots(KASICKA));
        assert!(!shoots(GIGACHAD));
        assert!(!shoots(DOGE_MINER));
        assert!(shoots(SHREK));
    }
}

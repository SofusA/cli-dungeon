use crate::{
    AbilityScaling, Dice,
    types::{AbilityScoreBonus, ArmorPoints, Gold},
};

pub struct WeaponAttackStats {
    pub primary_ability: AbilityScaling,
    pub hit_bonus: AbilityScoreBonus,
    pub attack_dices: Vec<Dice>,
    pub attack_bonus: AbilityScoreBonus,
}

pub struct Weapon {
    pub name: String,
    pub cost: Gold,
    pub attack_stats: WeaponAttackStats,
    pub allow_offhand: bool,
    pub armor_bonus: ArmorPoints,
}

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum WeaponType {
    Dagger,
    Shortsword,
    Longsword,
    Shield,
}

impl WeaponType {
    fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn from_weapon_str(string: &str) -> Option<Self> {
        let string = string.to_lowercase();
        match string.as_str() {
            "dagger" => Some(Self::Dagger),
            "shortsword" => Some(Self::Shortsword),
            "longsword" => Some(Self::Longsword),
            "shield" => Some(Self::Shield),
            _ => None,
        }
    }

    pub fn to_weapon(&self) -> Weapon {
        match self {
            WeaponType::Dagger => Weapon {
                name: self.to_name(),
                cost: Gold::new(5),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Dexterity,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D4],
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                armor_bonus: ArmorPoints::new(0),
            },
            WeaponType::Shortsword => Weapon {
                name: self.to_name(),
                cost: Gold::new(50),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Either,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D6],
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                armor_bonus: ArmorPoints::new(0),
            },
            WeaponType::Longsword => Weapon {
                name: self.to_name(),
                cost: Gold::new(50),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D8],
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: false,
                armor_bonus: ArmorPoints::new(0),
            },
            WeaponType::Shield => Weapon {
                name: self.to_name(),
                cost: Gold::new(30),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![],
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                armor_bonus: ArmorPoints::new(2),
            },
        }
    }
}

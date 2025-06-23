use crate::{
    Dice,
    abilities::AbilityScaling,
    types::{AbilityScoreBonus, ArmorPoints, Gold, Strength},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WeaponAttackStats {
    pub primary_ability: AbilityScaling,
    pub hit_bonus: AbilityScoreBonus,
    pub attack_dices: Vec<Dice>,
    pub versatile_attack_dices: Option<Vec<Dice>>,
    pub attack_bonus: AbilityScoreBonus,
}

pub struct Weapon {
    pub name: String,
    pub cost: Gold,
    pub attack_stats: WeaponAttackStats,
    pub allow_offhand: bool,
    pub two_handed: bool,
    pub strength_requirement: Strength,
    pub armor_bonus: ArmorPoints,
}

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum WeaponType {
    Dagger,
    Shortsword,
    Rapier,
    Longsword,
    GreatSword,
    GreatAxe,
    Shield,
    MonsterNone,
    MonsterD4,
    MonsterD4P1,
    MonsterD4P2,
    MonsterD6,
    MonsterD6P1,
    MonsterD6P2,
    MonsterD8,
    MonsterD8P1,
    MonsterD8P2,
    MonsterD8P3,
    MonsterD10,
    MonsterD10P1,
    MonsterD10P2,
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
            "rapier" => Some(Self::Rapier),
            "longsword" => Some(Self::Longsword),
            "greatsword" => Some(Self::GreatSword),
            "greataxe" => Some(Self::GreatAxe),
            "shield" => Some(Self::Shield),
            _ => None,
        }
    }

    pub fn to_weapon(&self) -> Weapon {
        match self {
            WeaponType::Dagger => Weapon {
                name: self.to_name(),
                cost: Gold::new(20),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Versatile,
                    hit_bonus: AbilityScoreBonus::new(1),
                    attack_dices: vec![Dice::D4],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::Shortsword => Weapon {
                name: self.to_name(),
                cost: Gold::new(100),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Versatile,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D6],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::Rapier => Weapon {
                name: self.to_name(),
                cost: Gold::new(200),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Dexterity,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D4, Dice::D4],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(8),
            },
            WeaponType::Longsword => Weapon {
                name: self.to_name(),
                cost: Gold::new(100),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D8],
                    versatile_attack_dices: Some(vec![Dice::D10]),
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: false,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(12),
            },
            WeaponType::GreatSword => Weapon {
                name: self.to_name(),
                cost: Gold::new(200),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D6, Dice::D6],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: false,
                two_handed: true,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(14),
            },
            WeaponType::GreatAxe => Weapon {
                name: self.to_name(),
                cost: Gold::new(200),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D12],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: false,
                two_handed: true,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(14),
            },
            WeaponType::Shield => Weapon {
                name: self.to_name(),
                cost: Gold::new(30),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(3),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterNone => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD4 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D4],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD4P1 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D4],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(1),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD4P2 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D4],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(2),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD6 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D6],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD6P1 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D6],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(1),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD6P2 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D6],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(2),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD8 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D8],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD8P1 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D8],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(1),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD8P2 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D8],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(2),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD8P3 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D8],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(3),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD10 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D10],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD10P1 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D10],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(1),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
            WeaponType::MonsterD10P2 => Weapon {
                name: self.to_name(),
                cost: Gold::new(0),
                attack_stats: WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D10],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(2),
                },
                allow_offhand: true,
                two_handed: false,
                armor_bonus: ArmorPoints::new(0),
                strength_requirement: Strength::new(0),
            },
        }
    }
}

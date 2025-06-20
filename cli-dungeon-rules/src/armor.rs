use crate::types::{AbilityScoreBonus, ArmorPoints, Gold, Strength};

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum ArmorType {
    Leather,
    StudedLeather,
    ChainShirt,
    BreastPlate,
    HalfPlate,
    ChainMail,
    Splint,
}

#[derive(Debug, Clone)]
pub struct Armor {
    pub name: String,
    pub cost: Gold,
    pub armor_bonus: ArmorPoints,
    pub max_dexterity_bonus: AbilityScoreBonus,
    pub strength_requirement: Strength,
}

impl ArmorType {
    fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn from_armor_str(string: &str) -> Option<Self> {
        let string = string.to_lowercase();
        match string.as_str() {
            "leather" => Some(Self::Leather),
            "chainmail" => Some(Self::BreastPlate),
            "splint" => Some(Self::Splint),
            _ => None,
        }
    }

    pub fn to_armor(&self) -> Armor {
        match self {
            ArmorType::Leather => Armor {
                name: self.to_name(),
                cost: Gold::new(30),
                armor_bonus: ArmorPoints::new(1),
                max_dexterity_bonus: AbilityScoreBonus::new(6),
                strength_requirement: Strength::new(8),
            },
            ArmorType::StudedLeather => Armor {
                name: self.to_name(),
                cost: Gold::new(150),
                armor_bonus: ArmorPoints::new(2),
                max_dexterity_bonus: AbilityScoreBonus::new(6),
                strength_requirement: Strength::new(8),
            },
            ArmorType::ChainShirt => Armor {
                name: self.to_name(),
                cost: Gold::new(100),
                armor_bonus: ArmorPoints::new(3),
                max_dexterity_bonus: AbilityScoreBonus::new(2),
                strength_requirement: Strength::new(8),
            },
            ArmorType::BreastPlate => Armor {
                name: self.to_name(),
                cost: Gold::new(150),
                armor_bonus: ArmorPoints::new(4),
                max_dexterity_bonus: AbilityScoreBonus::new(2),
                strength_requirement: Strength::new(10),
            },
            ArmorType::HalfPlate => Armor {
                name: self.to_name(),
                cost: Gold::new(250),
                armor_bonus: ArmorPoints::new(5),
                max_dexterity_bonus: AbilityScoreBonus::new(2),
                strength_requirement: Strength::new(12),
            },
            ArmorType::ChainMail => Armor {
                name: self.to_name(),
                cost: Gold::new(150),
                armor_bonus: ArmorPoints::new(6),
                max_dexterity_bonus: AbilityScoreBonus::new(2),
                strength_requirement: Strength::new(14),
            },
            ArmorType::Splint => Armor {
                name: self.to_name(),
                cost: Gold::new(200),
                armor_bonus: ArmorPoints::new(5),
                max_dexterity_bonus: AbilityScoreBonus::new(0),
                strength_requirement: Strength::new(16),
            },
        }
    }
}

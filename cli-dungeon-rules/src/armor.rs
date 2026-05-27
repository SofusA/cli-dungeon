use crate::{
    normalize_name,
    types::{AbilityScoreBonus, ArmorPoints, Gold, Strength},
};

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum ArmorType {
    Leather,
    StuddedLeather,
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
        match self {
            ArmorType::Leather => "Leather",
            ArmorType::StuddedLeather => "Studded Leather",
            ArmorType::BreastPlate => "Breastplate",
            ArmorType::HalfPlate => "Half Plate",
            ArmorType::ChainMail => "Chain Mail",
            ArmorType::Splint => "Splint",
        }
        .to_string()
    }

    pub fn from_armor_str(string: &str) -> Option<Self> {
        let normalized = normalize_name(string);

        match normalized.as_str() {
            "leather" => Some(Self::Leather),
            "studdedleather" | "studedleather" => Some(Self::StuddedLeather),
            "breastplate" => Some(Self::BreastPlate),
            "halfplate" => Some(Self::HalfPlate),
            "chainmail" => Some(Self::ChainMail),
            "splint" => Some(Self::Splint),
            _ => None,
        }
    }

    pub fn to_armor(&self) -> Armor {
        match self {
            ArmorType::Leather => Armor {
                name: self.to_name(),
                cost: Gold::new(50),
                armor_bonus: ArmorPoints::new(1),
                max_dexterity_bonus: AbilityScoreBonus::new(6),
                strength_requirement: Strength::new(8),
            },
            ArmorType::StuddedLeather => Armor {
                name: self.to_name(),
                cost: Gold::new(200),
                armor_bonus: ArmorPoints::new(2),
                max_dexterity_bonus: AbilityScoreBonus::new(6),
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
                max_dexterity_bonus: AbilityScoreBonus::new(1),
                strength_requirement: Strength::new(12),
            },
            ArmorType::ChainMail => Armor {
                name: self.to_name(),
                cost: Gold::new(150),
                armor_bonus: ArmorPoints::new(6),
                max_dexterity_bonus: AbilityScoreBonus::new(0),
                strength_requirement: Strength::new(14),
            },
            ArmorType::Splint => Armor {
                name: self.to_name(),
                cost: Gold::new(300),
                armor_bonus: ArmorPoints::new(7),
                max_dexterity_bonus: AbilityScoreBonus::new(0),
                strength_requirement: Strength::new(16),
            },
        }
    }
}

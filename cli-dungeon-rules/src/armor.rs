use derive_more::{Add, Deref, Display};

use crate::{
    abilities::{AbilityScore, AbilityScoreBonus, Strength},
    types::Gold,
};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum ArmorType {
    Leather,
    Chainmail,
    Splint,
}

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
            "chainmail" => Some(Self::Chainmail),
            "splint" => Some(Self::Splint),
            _ => None,
        }
    }

    pub fn to_armor(&self) -> Armor {
        match self {
            ArmorType::Leather => Armor {
                name: self.to_name(),
                cost: Gold(30),
                armor_bonus: ArmorPoints(1),
                max_dexterity_bonus: AbilityScoreBonus(6),
                strength_requirement: Strength(AbilityScore(8)),
            },
            ArmorType::Chainmail => Armor {
                name: self.to_name(),
                cost: Gold(150),
                armor_bonus: ArmorPoints(4),
                max_dexterity_bonus: AbilityScoreBonus(4),
                strength_requirement: Strength(AbilityScore(14)),
            },
            ArmorType::Splint => Armor {
                name: self.to_name(),
                cost: Gold(200),
                armor_bonus: ArmorPoints(7),
                max_dexterity_bonus: AbilityScoreBonus(0),
                strength_requirement: Strength(AbilityScore(16)),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, Deref, Add, Display)]
pub struct ArmorPoints(pub i16);

impl From<AbilityScoreBonus> for ArmorPoints {
    fn from(value: AbilityScoreBonus) -> Self {
        Self(*value)
    }
}

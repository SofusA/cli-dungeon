use crate::types::{AbilityScoreBonus, ArmorPoints, Gold, Strength};

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
                cost: Gold::new(30),
                armor_bonus: ArmorPoints::new(1),
                max_dexterity_bonus: AbilityScoreBonus::new(6),
                strength_requirement: Strength::new(8),
            },
            ArmorType::Chainmail => Armor {
                name: self.to_name(),
                cost: Gold::new(150),
                armor_bonus: ArmorPoints::new(4),
                max_dexterity_bonus: AbilityScoreBonus::new(4),
                strength_requirement: Strength::new(14),
            },
            ArmorType::Splint => Armor {
                name: self.to_name(),
                cost: Gold::new(200),
                armor_bonus: ArmorPoints::new(7),
                max_dexterity_bonus: AbilityScoreBonus::new(0),
                strength_requirement: Strength::new(16),
            },
        }
    }
}

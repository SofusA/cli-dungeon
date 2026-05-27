use crate::{
    normalize_name,
    types::{Constitution, Dexterity, Strength, Wisdom},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AbilityScaling {
    Strength,
    Dexterity,
    Versatile,
    Wisdom,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, serde::Deserialize, serde::Serialize)]
pub enum AbilityType {
    Strength,
    Dexterity,
    Constitution,
    Wisdom,
}

impl AbilityType {
    pub fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn from_ability_str(string: &str) -> Option<Self> {
        let string = normalize_name(string);
        match string.as_str() {
            "strength" => Some(Self::Strength),
            "dexterity" => Some(Self::Dexterity),
            "constitution" => Some(Self::Constitution),
            "wisdom" => Some(Self::Wisdom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AbilityScores {
    pub strength: Strength,
    pub dexterity: Dexterity,
    pub constitution: Constitution,
    pub wisdom: Wisdom,
}

impl AbilityScores {
    pub fn new(strength: i16, dexterity: i16, constitution: i16, wisdom: i16) -> Self {
        Self {
            strength: Strength::new(strength),
            dexterity: Dexterity::new(dexterity),
            constitution: Constitution::new(constitution),
            wisdom: Wisdom::new(wisdom),
        }
    }
}

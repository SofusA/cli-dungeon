use crate::types::{Constitution, Dexterity, Strength};

pub enum AbilityScaling {
    Strength,
    Dexterity,
    Either,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, serde::Deserialize, serde::Serialize)]
pub enum AbilityType {
    Strength,
    Dexterity,
    Constitution,
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
        let string = string.to_lowercase();
        match string.as_str() {
            "strength" => Some(Self::Strength),
            "dexterity" => Some(Self::Dexterity),
            "constitution" => Some(Self::Constitution),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AbilityScores {
    pub strength: Strength,
    pub dexterity: Dexterity,
    pub constitution: Constitution,
}

impl AbilityScores {
    pub fn new(strength: i16, dexterity: i16, constitution: i16) -> Self {
        Self {
            strength: Strength::new(strength),
            dexterity: Dexterity::new(dexterity),
            constitution: Constitution::new(constitution),
        }
    }
}

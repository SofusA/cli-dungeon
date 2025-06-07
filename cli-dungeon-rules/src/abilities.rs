use derive_more::{Add, Deref};

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

#[derive(
    Debug,
    serde::Deserialize,
    serde::Serialize,
    Deref,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Add,
    Clone,
    Copy,
)]
pub struct AbilityScore(pub u16);

#[derive(
    Debug,
    serde::Deserialize,
    serde::Serialize,
    Deref,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Add,
    Clone,
    Copy,
)]
pub struct Strength(pub AbilityScore);
#[derive(
    Debug,
    serde::Deserialize,
    serde::Serialize,
    Deref,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Add,
    Clone,
    Copy,
)]
pub struct Dexterity(pub AbilityScore);

#[derive(
    Debug,
    serde::Deserialize,
    serde::Serialize,
    Deref,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Add,
    Clone,
    Copy,
)]
pub struct Constitution(pub AbilityScore);

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, Deref, Add, PartialEq, Eq)]
pub struct AbilityScoreBonus(pub i16);

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AbilityScores {
    pub strength: Strength,
    pub dexterity: Dexterity,
    pub constitution: Constitution,
}

impl AbilityScores {
    pub fn new(strength: u16, dexterity: u16, constitution: u16) -> Self {
        Self {
            strength: Strength(AbilityScore(strength)),
            dexterity: Dexterity(AbilityScore(dexterity)),
            constitution: Constitution(AbilityScore(constitution)),
        }
    }
}

impl AbilityScore {
    pub fn ability_score_bonus(&self) -> AbilityScoreBonus {
        AbilityScoreBonus((self.0 as i16 - 10) / 2)
    }
}

use crate::types::{ArmorPoints, Strength, Turn};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct ActiveCondition {
    pub duration: Turn,
    pub condition_type: ConditionType,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum ConditionType {
    Weaken,
}

pub struct Condition {
    pub armor_bonus: Option<ArmorPoints>,
    pub strength_bonus: Option<Strength>,
}

impl ConditionType {
    pub fn to_condition(&self) -> Condition {
        match self {
            ConditionType::Weaken => Condition {
                armor_bonus: None,
                strength_bonus: Some(Strength::new(-1)),
            },
        }
    }
}

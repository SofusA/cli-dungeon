use crate::types::{ArmorPoints, Strength, Turn};

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct ActiveCondition {
    pub duration: Option<Turn>,
    pub condition_type: ConditionType,
}

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum ConditionType {
    Weaken,
}

pub struct Condition {
    pub name: String,
    pub armor_bonus: Option<ArmorPoints>,
    pub strength_bonus: Option<Strength>,
}

impl ConditionType {
    fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }
    pub fn to_condition(&self) -> Condition {
        match self {
            ConditionType::Weaken => Condition {
                name: self.to_name(),
                armor_bonus: None,
                strength_bonus: Some(Strength::new(-1)),
            },
        }
    }
}

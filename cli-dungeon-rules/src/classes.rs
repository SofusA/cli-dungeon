use crate::abilities::AbilityType;

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum ClassType {
    Monster,
    Fighter,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct LevelUpChoice {
    pub ability_increment: AbilityType,
    pub class: ClassType,
}

impl ClassType {
    pub fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn from_class_str(string: &str) -> Option<Self> {
        let string = string.to_lowercase();
        match string.as_str() {
            "fighter" => Some(Self::Fighter),
            _ => None,
        }
    }
}

use crate::{
    conditions::{ActiveCondition, ConditionType},
    types::Turn,
    weapons::WeaponAttackStats,
};

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum SpellType {
    Weaken,
}

pub enum SpellAction {
    Condition(ActiveCondition),
    Projectile(WeaponAttackStats),
}

pub struct Spell {
    pub name: String,
    pub action: Option<SpellAction>,
    pub bonus_action: Option<SpellAction>,
}

impl SpellType {
    fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn to_spell(&self) -> Spell {
        match self {
            SpellType::Weaken => Spell {
                name: self.to_name(),
                action: Some(SpellAction::Condition(ActiveCondition {
                    duration: Some(Turn::new(2)),
                    condition_type: ConditionType::Weaken,
                })),
                bonus_action: None,
            },
        }
    }
}

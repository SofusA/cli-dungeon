use crate::{
    Dice,
    abilities::AbilityScaling,
    spells::SpellType,
    types::{AbilityScoreBonus, Gold, HealthPoints},
    weapons::WeaponAttackStats,
};

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum ItemType {
    Stone,
    ScrollOfWeaken,
    MinorHealingPotion,
}

pub enum ActionType {
    Action(ItemAction),
    BonusAction(ItemAction),
}

pub struct Item {
    pub name: String,
    pub cost: Gold,
    pub action: ActionType,
}

pub enum ItemAction {
    Spell(SpellType),
    Projectile(WeaponAttackStats),
    Healing(HealthPoints),
}

impl ItemType {
    fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn to_item(&self) -> Item {
        match self {
            ItemType::Stone => Item {
                name: self.to_name(),
                cost: Gold::new(1),
                action: ActionType::Action(ItemAction::Projectile(WeaponAttackStats {
                    primary_ability: AbilityScaling::Strength,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D4],
                    attack_bonus: AbilityScoreBonus::new(0),
                })),
            },
            ItemType::ScrollOfWeaken => Item {
                name: self.to_name(),
                cost: Gold::new(1000),
                action: ActionType::Action(ItemAction::Spell(SpellType::Weaken)),
            },
            ItemType::MinorHealingPotion => Item {
                name: self.to_name(),
                cost: Gold::new(500),
                action: ActionType::BonusAction(ItemAction::Healing(HealthPoints::new(10))),
            },
        }
    }
}

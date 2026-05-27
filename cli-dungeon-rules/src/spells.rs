use crate::{
    Dice,
    abilities::AbilityScaling,
    conditions::{ActiveCondition, ConditionType},
    types::AbilityScoreBonus,
    types::Turn,
    weapons::WeaponAttackStats,
};

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum SpellType {
    Weaken,
    Cripple,
    Poison,
    Expose,
    Exhaust,
    Firebolt,
    IceShard,
    LightningStrike,
    ArcaneMissile,
    ShadowBolt,
    Strength,
    Agility,
    Fortify,
    Focus,
    Reckless,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpellAction {
    Condition(ActiveCondition),
    Projectile(WeaponAttackStats),
}

pub struct Spell {
    pub name: String,
    pub action: SpellActionType,
}

pub enum SpellActionType {
    Action(SpellAction),
    BonusAction(SpellAction),
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
                action: SpellActionType::Action(SpellAction::Condition(ActiveCondition {
                    remaining_turns: Some(Turn::new(2)),
                    condition_type: ConditionType::Weaken,
                })),
            },
            SpellType::Cripple => Spell {
                name: self.to_name(),
                action: SpellActionType::Action(SpellAction::Condition(ActiveCondition {
                    remaining_turns: Some(Turn::new(2)),
                    condition_type: ConditionType::Crippled,
                })),
            },
            SpellType::Poison => Spell {
                name: self.to_name(),
                action: SpellActionType::Action(SpellAction::Condition(ActiveCondition {
                    remaining_turns: Some(Turn::new(3)),
                    condition_type: ConditionType::Poisoned,
                })),
            },
            SpellType::Expose => Spell {
                name: self.to_name(),
                action: SpellActionType::Action(SpellAction::Condition(ActiveCondition {
                    remaining_turns: Some(Turn::new(2)),
                    condition_type: ConditionType::Exposed,
                })),
            },
            SpellType::Exhaust => Spell {
                name: self.to_name(),
                action: SpellActionType::Action(SpellAction::Condition(ActiveCondition {
                    remaining_turns: Some(Turn::new(2)),
                    condition_type: ConditionType::Exhausted,
                })),
            },
            SpellType::Firebolt => Spell {
                name: self.to_name(),
                action: SpellActionType::Action(SpellAction::Projectile(WeaponAttackStats {
                    primary_ability: AbilityScaling::Wisdom,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D6, Dice::D6],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                    condition_on_hit: Some(ActiveCondition {
                        remaining_turns: Some(Turn::new(2)),
                        condition_type: ConditionType::Burning,
                    }),
                })),
            },
            SpellType::IceShard => Spell {
                name: self.to_name(),
                action: SpellActionType::Action(SpellAction::Projectile(WeaponAttackStats {
                    primary_ability: AbilityScaling::Wisdom,
                    hit_bonus: AbilityScoreBonus::new(1),
                    attack_dices: vec![Dice::D4, Dice::D6],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                    condition_on_hit: Some(ActiveCondition {
                        remaining_turns: Some(Turn::new(2)),
                        condition_type: ConditionType::Exhausted,
                    }),
                })),
            },
            SpellType::LightningStrike => Spell {
                name: self.to_name(),
                action: SpellActionType::Action(SpellAction::Projectile(WeaponAttackStats {
                    primary_ability: AbilityScaling::Wisdom,
                    hit_bonus: AbilityScoreBonus::new(-1),
                    attack_dices: vec![Dice::D10],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(2),
                    condition_on_hit: Some(ActiveCondition {
                        remaining_turns: Some(Turn::new(1)),
                        condition_type: ConditionType::Exposed,
                    }),
                })),
            },
            SpellType::ArcaneMissile => Spell {
                name: self.to_name(),
                action: SpellActionType::Action(SpellAction::Projectile(WeaponAttackStats {
                    primary_ability: AbilityScaling::Wisdom,
                    hit_bonus: AbilityScoreBonus::new(2),
                    attack_dices: vec![Dice::D4, Dice::D4],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                    condition_on_hit: None,
                })),
            },
            SpellType::ShadowBolt => Spell {
                name: self.to_name(),
                action: SpellActionType::Action(SpellAction::Projectile(WeaponAttackStats {
                    primary_ability: AbilityScaling::Wisdom,
                    hit_bonus: AbilityScoreBonus::new(0),
                    attack_dices: vec![Dice::D8],
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(1),
                    condition_on_hit: Some(ActiveCondition {
                        remaining_turns: Some(Turn::new(2)),
                        condition_type: ConditionType::Weaken,
                    }),
                })),
            },
            SpellType::Strength => Spell {
                name: self.to_name(),
                action: SpellActionType::BonusAction(SpellAction::Condition(ActiveCondition {
                    remaining_turns: Some(Turn::new(3)),
                    condition_type: ConditionType::Strong,
                })),
            },
            SpellType::Agility => Spell {
                name: self.to_name(),
                action: SpellActionType::BonusAction(SpellAction::Condition(ActiveCondition {
                    remaining_turns: Some(Turn::new(3)),
                    condition_type: ConditionType::Agile,
                })),
            },
            SpellType::Fortify => Spell {
                name: self.to_name(),
                action: SpellActionType::BonusAction(SpellAction::Condition(ActiveCondition {
                    remaining_turns: Some(Turn::new(3)),
                    condition_type: ConditionType::Fortified,
                })),
            },
            SpellType::Focus => Spell {
                name: self.to_name(),
                action: SpellActionType::BonusAction(SpellAction::Condition(ActiveCondition {
                    remaining_turns: Some(Turn::new(2)),
                    condition_type: ConditionType::Focused,
                })),
            },
            SpellType::Reckless => Spell {
                name: self.to_name(),
                action: SpellActionType::BonusAction(SpellAction::Condition(ActiveCondition {
                    remaining_turns: Some(Turn::new(2)),
                    condition_type: ConditionType::Reckless,
                })),
            },
        }
    }

    pub fn spell_action(&self) -> SpellAction {
        match self.to_spell().action {
            SpellActionType::Action(action) | SpellActionType::BonusAction(action) => action,
        }
    }
}

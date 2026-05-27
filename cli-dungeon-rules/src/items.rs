use crate::{
    Dice,
    abilities::AbilityScaling,
    conditions::{ActiveCondition, ConditionType},
    normalize_name, roll,
    spells::{SpellAction, SpellType},
    types::{AbilityScoreBonus, Gold, HealthPoints, Turn},
    weapons::WeaponAttackStats,
};

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum ItemType {
    Stone,
    ScrollOfWeaken,
    ScrollOfCripple,
    ScrollOfPoison,
    ScrollOfIceShard,
    ScrollOfFirebolt,
    PotionOfHealing,
    PotionOfStrength,
    PotionOfAgility,
    PotionOfFortitude,
    FireBomb,
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct HealingStats {
    pub dice: Vec<Dice>,
    pub bonus: HealthPoints,
}

impl HealingStats {
    pub fn roll(self) -> HealthPoints {
        HealthPoints::new(self.dice.iter().map(roll).sum()) + self.bonus
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemAction {
    Spell(SpellAction),
    Projectile(WeaponAttackStats),
    Healing(HealingStats),
}

impl ItemType {
    fn to_name(self) -> String {
        match self {
            ItemType::Stone => "Stone",
            ItemType::ScrollOfWeaken => "Scroll of Weaken",
            ItemType::ScrollOfCripple => "Scroll of Cripple",
            ItemType::ScrollOfPoison => "Scroll of Poison",
            ItemType::ScrollOfIceShard => "Scroll of Ice Shard",
            ItemType::ScrollOfFirebolt => "Scroll of Firebolt",
            ItemType::PotionOfHealing => "Potion of healing",
            ItemType::PotionOfStrength => "Potion of Strength",
            ItemType::PotionOfAgility => "Potion of Agility",
            ItemType::PotionOfFortitude => "Potion of Fortitude",
            ItemType::FireBomb => "Fire bomb",
        }
        .to_string()
    }

    pub fn from_item_str(string: &str) -> Option<Self> {
        let normalized = normalize_name(string);

        match normalized.as_str() {
            "stone" => Some(Self::Stone),
            "scrollofweaken" => Some(Self::ScrollOfWeaken),
            "scrollofcripple" => Some(Self::ScrollOfCripple),
            "scrollofpoison" => Some(Self::ScrollOfPoison),
            "scrolloficeshard" => Some(Self::ScrollOfIceShard),
            "scrolloffirebolt" => Some(Self::ScrollOfFirebolt),
            "potionofhealing" => Some(Self::PotionOfHealing),
            "potionofstrength" => Some(Self::PotionOfStrength),
            "potionofagility" => Some(Self::PotionOfAgility),
            "potionoffortitude" => Some(Self::PotionOfFortitude),
            "firebomb" => Some(Self::FireBomb),
            _ => None,
        }
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
                    versatile_attack_dices: None,
                    attack_bonus: AbilityScoreBonus::new(0),
                    condition_on_hit: None,
                })),
            },
            ItemType::ScrollOfWeaken => Item {
                name: self.to_name(),
                cost: Gold::new(250),
                action: ActionType::Action(ItemAction::Spell(SpellType::Weaken.spell_action())),
            },
            ItemType::ScrollOfCripple => Item {
                name: self.to_name(),
                cost: Gold::new(300),
                action: ActionType::Action(ItemAction::Spell(SpellType::Cripple.spell_action())),
            },
            ItemType::ScrollOfPoison => Item {
                name: self.to_name(),
                cost: Gold::new(350),
                action: ActionType::Action(ItemAction::Spell(SpellType::Poison.spell_action())),
            },
            ItemType::ScrollOfIceShard => Item {
                name: self.to_name(),
                cost: Gold::new(400),
                action: ActionType::Action(ItemAction::Spell(SpellType::IceShard.spell_action())),
            },
            ItemType::ScrollOfFirebolt => Item {
                name: self.to_name(),
                cost: Gold::new(400),
                action: ActionType::Action(ItemAction::Spell(SpellType::Firebolt.spell_action())),
            },
            ItemType::PotionOfHealing => Item {
                name: self.to_name(),
                cost: Gold::new(100),
                action: ActionType::BonusAction(ItemAction::Healing(HealingStats {
                    dice: vec![Dice::D4, Dice::D4],
                    bonus: HealthPoints::new(2),
                })),
            },
            ItemType::PotionOfStrength => Item {
                name: self.to_name(),
                cost: Gold::new(250),
                action: ActionType::BonusAction(ItemAction::Spell(
                    SpellType::Strength.spell_action(),
                )),
            },
            ItemType::PotionOfAgility => Item {
                name: self.to_name(),
                cost: Gold::new(250),
                action: ActionType::BonusAction(ItemAction::Spell(
                    SpellType::Agility.spell_action(),
                )),
            },
            ItemType::PotionOfFortitude => Item {
                name: self.to_name(),
                cost: Gold::new(300),
                action: ActionType::BonusAction(ItemAction::Spell(
                    SpellType::Fortify.spell_action(),
                )),
            },
            ItemType::FireBomb => Item {
                name: self.to_name(),
                cost: Gold::new(150),
                action: ActionType::Action(ItemAction::Projectile(WeaponAttackStats {
                    primary_ability: AbilityScaling::Dexterity,
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
        }
    }

    pub fn item_action(&self) -> ItemAction {
        match self.to_item().action {
            ActionType::Action(action) | ActionType::BonusAction(action) => action,
        }
    }
}

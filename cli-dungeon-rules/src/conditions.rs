use crate::{
    Dice,
    types::{ArmorPoints, Constitution, Dexterity, Strength, Turn, Wisdom},
};

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct ActiveCondition {
    pub remaining_turns: Option<Turn>,
    pub condition_type: ConditionType,
}

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum ConditionType {
    /// Strength -1
    Weaken,
    /// Dexterity -2
    Crippled,
    /// Constitution -2
    Poisoned,
    /// Armor -3
    Exposed,
    /// Strength, dexterity, constitution -1
    Exhausted,
    /// Strength +2
    Strong,
    /// Dexterity +2
    Agile,
    /// Constitution, armor +2
    Fortified,
    /// Wisdom up
    Focused,
    /// Armor -2, strength +3
    Reckless,
    /// Armor +3, strength -1
    Guarded,
    /// 1D4 tick
    Burning,
    /// Strength +1
    StrengthMinor,
    /// Dexterity +1
    DexterityMinor,
    /// Constitution +1
    ConstitutionMinor,
    /// Armor +1
    ArmorMinor,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Condition {
    pub name: String,
    pub armor_bonus: Option<ArmorPoints>,
    pub strength_bonus: Option<Strength>,
    pub dexterity_bonus: Option<Dexterity>,
    pub constitution_bonus: Option<Constitution>,
    pub wisdom_bonus: Option<Wisdom>,
    pub tick_damage: Option<Dice>,
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
                dexterity_bonus: None,
                constitution_bonus: None,
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::Crippled => Condition {
                name: self.to_name(),
                armor_bonus: None,
                strength_bonus: None,
                dexterity_bonus: Some(Dexterity::new(-2)),
                constitution_bonus: None,
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::Poisoned => Condition {
                name: self.to_name(),
                armor_bonus: None,
                strength_bonus: None,
                dexterity_bonus: None,
                constitution_bonus: Some(Constitution::new(-2)),
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::Exposed => Condition {
                name: self.to_name(),
                armor_bonus: Some(ArmorPoints::new(-3)),
                strength_bonus: None,
                dexterity_bonus: None,
                constitution_bonus: None,
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::Exhausted => Condition {
                name: self.to_name(),
                armor_bonus: None,
                strength_bonus: Some(Strength::new(-1)),
                dexterity_bonus: Some(Dexterity::new(-1)),
                constitution_bonus: Some(Constitution::new(-1)),
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::Strong => Condition {
                name: self.to_name(),
                armor_bonus: None,
                strength_bonus: Some(Strength::new(2)),
                dexterity_bonus: None,
                constitution_bonus: None,
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::Agile => Condition {
                name: self.to_name(),
                armor_bonus: None,
                strength_bonus: None,
                dexterity_bonus: Some(Dexterity::new(2)),
                constitution_bonus: None,
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::Fortified => Condition {
                name: self.to_name(),
                armor_bonus: Some(ArmorPoints::new(2)),
                strength_bonus: None,
                dexterity_bonus: None,
                constitution_bonus: Some(Constitution::new(2)),
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::Focused => Condition {
                name: self.to_name(),
                armor_bonus: None,
                strength_bonus: None,
                dexterity_bonus: None,
                constitution_bonus: None,
                wisdom_bonus: Some(Wisdom::new(2)),
                tick_damage: None,
            },
            ConditionType::Reckless => Condition {
                name: self.to_name(),
                armor_bonus: Some(ArmorPoints::new(-2)),
                strength_bonus: Some(Strength::new(3)),
                dexterity_bonus: None,
                constitution_bonus: None,
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::Guarded => Condition {
                name: self.to_name(),
                armor_bonus: Some(ArmorPoints::new(3)),
                strength_bonus: Some(Strength::new(-1)),
                dexterity_bonus: None,
                constitution_bonus: None,
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::StrengthMinor => Condition {
                name: self.to_name(),
                armor_bonus: None,
                strength_bonus: Some(Strength::new(1)),
                dexterity_bonus: None,
                constitution_bonus: None,
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::DexterityMinor => Condition {
                name: self.to_name(),
                armor_bonus: None,
                strength_bonus: None,
                dexterity_bonus: Some(Dexterity::new(1)),
                constitution_bonus: None,
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::ConstitutionMinor => Condition {
                name: self.to_name(),
                armor_bonus: None,
                strength_bonus: None,
                dexterity_bonus: None,
                constitution_bonus: Some(Constitution::new(1)),
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::ArmorMinor => Condition {
                name: self.to_name(),
                armor_bonus: Some(ArmorPoints::new(1)),
                strength_bonus: None,
                dexterity_bonus: None,
                constitution_bonus: None,
                wisdom_bonus: None,
                tick_damage: None,
            },
            ConditionType::Burning => Condition {
                name: self.to_name(),
                armor_bonus: None,
                strength_bonus: None,
                dexterity_bonus: None,
                constitution_bonus: None,
                wisdom_bonus: None,
                tick_damage: Some(Dice::D4),
            },
        }
    }
}

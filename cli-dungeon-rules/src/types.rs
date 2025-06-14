use derive_more::{Add, AddAssign, Deref, Display, Sub, SubAssign};
use serde::{Deserialize, Serialize};

macro_rules! wrapped_type {
    ($name:ident, $inner:ty) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            Display,
            Deserialize,
            Serialize,
            Deref,
            Add,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Sub,
            AddAssign,
            SubAssign,
        )]
        pub struct $name($inner);

        impl $name {
            pub fn new(value: $inner) -> Self {
                Self(value)
            }
        }
    };
}

macro_rules! ability_wrapped_type {
    ($name:ident) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            Display,
            Deserialize,
            Serialize,
            Deref,
            Add,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Sub,
            AddAssign,
            SubAssign,
        )]
        pub struct $name(AbilityScore);

        impl $name {
            pub fn new(value: i16) -> Self {
                Self(AbilityScore::new(value))
            }
        }
    };
}

wrapped_type!(Experience, u32);
wrapped_type!(Level, u16);
wrapped_type!(HealthPoints, i16);
wrapped_type!(Gold, u16);
wrapped_type!(ArmorPoints, i16);
wrapped_type!(AbilityScore, i16);
wrapped_type!(AbilityScoreBonus, i16);
wrapped_type!(Turn, i16);
wrapped_type!(QuestPoint, u16);
ability_wrapped_type!(Strength);
ability_wrapped_type!(Dexterity);
ability_wrapped_type!(Constitution);

impl AbilityScore {
    pub fn ability_score_bonus(&self) -> AbilityScoreBonus {
        AbilityScoreBonus((self.0 - 10) / 2)
    }
}

impl From<AbilityScoreBonus> for ArmorPoints {
    fn from(value: AbilityScoreBonus) -> Self {
        Self(*value)
    }
}

impl Gold {
    pub fn sell_value(&self) -> Self {
        Gold::new(**self / 2)
    }
}

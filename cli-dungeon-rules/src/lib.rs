use crate::{
    character::Character,
    types::{AbilityScoreBonus, HealthPoints},
};

pub mod abilities;
pub mod armor;
pub mod character;
pub mod classes;
pub mod conditions;
pub mod items;
pub mod jewelry;
pub mod loot;
pub mod monsters;
pub mod spells;
pub mod types;
pub mod weapons;

pub fn roll(dice: &Dice) -> i16 {
    let max = match dice {
        Dice::D4 => 4,
        Dice::D6 => 6,
        Dice::D8 => 8,
        Dice::D10 => 10,
        Dice::D12 => 12,
        Dice::D20 => 20,
    };

    rand::random_range(1..=max)
}

pub fn roll_success(dice: &Dice) -> bool {
    let max = match dice {
        Dice::D4 => 4,
        Dice::D6 => 6,
        Dice::D8 => 8,
        Dice::D10 => 10,
        Dice::D12 => 12,
        Dice::D20 => 20,
    };

    rand::random_range(1..=max) == max
}

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum Dice {
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
}

pub struct AttackStats {
    pub attack_dice: Vec<Dice>,
    pub attack_bonus: AbilityScoreBonus,
    pub hit_bonus: AbilityScoreBonus,
}

#[derive(Default, Clone)]
pub enum Status {
    #[default]
    Resting,
    Questing,
    Fighting(i64),
}

#[derive(Clone)]
pub struct Encounter {
    pub id: i64,
    pub rotation: Vec<Character>,
    pub dead_characters: Vec<Character>,
}

#[derive(Debug, Clone)]
pub struct Hit {
    pub damage: HealthPoints,
    pub critical_hit: bool,
    pub character_name: String,
}

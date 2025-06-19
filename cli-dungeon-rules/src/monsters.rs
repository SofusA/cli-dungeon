use rand::seq::IndexedRandom;

use crate::{
    Dice,
    abilities::{AbilityScores, AbilityType},
    armor::ArmorType,
    classes::{ClassType, LevelUpChoice},
    conditions::ActiveCondition,
    items::ItemType,
    jewelry::JewelryType,
    roll_success,
    types::{Experience, Gold, Level},
    weapons::WeaponType,
};

pub fn get_monster_encounter(level: Level) -> Vec<MonsterType> {
    let index = *level as usize;

    monster_catalogue()
        .get(index)
        .unwrap()
        .choose(&mut rand::rng())
        .unwrap()
        .to_vec()
}

fn monster_catalogue() -> Vec<Vec<Vec<MonsterType>>> {
    vec![
        vec![vec![MonsterType::Slime, MonsterType::TestMonsterWithDagger]],
        vec![vec![MonsterType::Wolf]],
        vec![
            vec![MonsterType::Wolf, MonsterType::DireWolf],
            vec![MonsterType::Wolf, MonsterType::Wolf],
        ],
    ]
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum MonsterType {
    TestMonster,
    TestMonsterWithDagger,
    TestMonsterWithLeatherArmor,
    TestMonsterWithRingOfProtectionAndStone,
    Slime,
    Wolf,
    DireWolf,
}

impl MonsterType {
    pub fn to_monster(self) -> MonsterDefinition {
        match self {
            MonsterType::TestMonster => MonsterDefinition::new(
                "Test monster",
                AbilityScores::new(4, 2, 14),
                Gold::new(1),
                None,
                None,
                None,
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
            MonsterType::TestMonsterWithDagger => MonsterDefinition::new(
                "Test monster",
                AbilityScores::new(4, 2, 14),
                Gold::new(1),
                None,
                None,
                None,
                vec![],
                vec![(WeaponType::Dagger, None)],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
            MonsterType::TestMonsterWithLeatherArmor => MonsterDefinition::new(
                "Test monster",
                AbilityScores::new(4, 2, 2),
                Gold::new(1),
                None,
                None,
                None,
                vec![],
                vec![],
                vec![(ArmorType::Leather, None)],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
            MonsterType::TestMonsterWithRingOfProtectionAndStone => MonsterDefinition::new(
                "Test monster",
                AbilityScores::new(4, 2, 14),
                Gold::new(1),
                None,
                None,
                None,
                vec![JewelryType::RingOfProtection],
                vec![],
                vec![],
                vec![(JewelryType::RingOfProtection, None)],
                vec![(ItemType::Stone, None)],
                vec![],
                vec![],
            ),
            MonsterType::Slime => MonsterDefinition::new(
                "Slime",
                AbilityScores::new(4, 2, 2),
                Gold::new(5),
                None,
                None,
                None,
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
            MonsterType::Wolf => MonsterDefinition::new(
                "Wolf",
                AbilityScores::new(8, 10, 9),
                Gold::new(5),
                None,
                None,
                None,
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![LevelUpChoice {
                    ability_increment: AbilityType::Dexterity,
                    class: ClassType::Monster,
                }],
                vec![],
            ),
            MonsterType::DireWolf => MonsterDefinition::new(
                "Dire wolf",
                AbilityScores::new(8, 9, 9),
                Gold::new(5),
                None,
                None,
                None,
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![
                    LevelUpChoice {
                        ability_increment: AbilityType::Dexterity,
                        class: ClassType::Monster,
                    },
                    LevelUpChoice {
                        ability_increment: AbilityType::Constitution,
                        class: ClassType::Monster,
                    },
                ],
                vec![],
            ),
        }
    }
}

pub struct MonsterDefinition {
    pub name: String,
    pub base_ability_scores: AbilityScores,
    pub gold: Gold,
    pub experience: Experience,
    pub equipped_weapon: Option<WeaponType>,
    pub equipped_offhand: Option<WeaponType>,
    pub equipped_armor: Option<ArmorType>,
    pub equipped_jewelry: Vec<JewelryType>,
    pub weapon_inventory: Vec<WeaponType>,
    pub armor_inventory: Vec<ArmorType>,
    pub jewelry_inventory: Vec<JewelryType>,
    pub item_inventory: Vec<ItemType>,
    pub levels: Vec<LevelUpChoice>,
    pub active_conditions: Vec<ActiveCondition>,
}

impl MonsterDefinition {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        base_ability_scores: AbilityScores,
        gold: Gold,
        equipped_weapon: Option<WeaponType>,
        equipped_offhand: Option<WeaponType>,
        equipped_armor: Option<ArmorType>,
        equipped_jewelry: Vec<JewelryType>,
        weapon_inventory: Vec<(WeaponType, Option<Dice>)>,
        armor_inventory: Vec<(ArmorType, Option<Dice>)>,
        jewelry_inventory: Vec<(JewelryType, Option<Dice>)>,
        item_inventory: Vec<(ItemType, Option<Dice>)>,
        levels: Vec<LevelUpChoice>,
        active_conditions: Vec<ActiveCondition>,
    ) -> Self {
        let weapon_inventory: Vec<_> = weapon_inventory
            .into_iter()
            .flat_map(|item| match item.1 {
                Some(dice) => {
                    if roll_success(&dice) {
                        Some(item.0)
                    } else {
                        None
                    }
                }
                None => Some(item.0),
            })
            .collect();

        let armor_inventory: Vec<_> = armor_inventory
            .into_iter()
            .flat_map(|item| match item.1 {
                Some(dice) => {
                    if roll_success(&dice) {
                        Some(item.0)
                    } else {
                        None
                    }
                }
                None => Some(item.0),
            })
            .collect();

        let jewelry_inventory: Vec<_> = jewelry_inventory
            .into_iter()
            .flat_map(|item| match item.1 {
                Some(dice) => {
                    if roll_success(&dice) {
                        Some(item.0)
                    } else {
                        None
                    }
                }
                None => Some(item.0),
            })
            .collect();

        let item_inventory: Vec<_> = item_inventory
            .into_iter()
            .flat_map(|item| match item.1 {
                Some(dice) => {
                    if roll_success(&dice) {
                        Some(item.0)
                    } else {
                        None
                    }
                }
                None => Some(item.0),
            })
            .collect();

        Self {
            name: name.to_string(),
            base_ability_scores,
            gold,
            experience: Experience::new(0),
            equipped_weapon,
            equipped_offhand,
            equipped_armor,
            equipped_jewelry,
            weapon_inventory,
            armor_inventory,
            jewelry_inventory,
            item_inventory,
            levels,
            active_conditions,
        }
    }
}

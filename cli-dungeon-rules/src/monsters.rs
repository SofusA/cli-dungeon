use rand::seq::IndexedRandom;

use crate::{
    abilities::{AbilityScores, AbilityType},
    armor::ArmorType,
    classes::{ClassType, LevelUpChoice},
    types::{Experience, Gold},
    weapons::WeaponType,
};

pub fn get_monster_encounter(rating: usize) -> Vec<MonsterType> {
    monster_catalogue()
        .get(rating)
        .unwrap()
        .choose(&mut rand::rng())
        .unwrap()
        .to_vec()
}

fn monster_catalogue() -> Vec<Vec<Vec<MonsterType>>> {
    vec![
        vec![vec![MonsterType::Slime]],
        vec![vec![MonsterType::Wolf]],
        vec![
            vec![MonsterType::Wolf, MonsterType::DireWolf],
            vec![MonsterType::Wolf, MonsterType::Wolf],
        ],
        vec![vec![MonsterType::Wolf, MonsterType::DireWolf]],
        vec![vec![MonsterType::Wolf, MonsterType::DireWolf]],
        vec![vec![MonsterType::Wolf, MonsterType::DireWolf]],
        vec![vec![MonsterType::Wolf, MonsterType::DireWolf]],
        vec![vec![MonsterType::Wolf, MonsterType::DireWolf]],
        vec![vec![MonsterType::Wolf, MonsterType::DireWolf]],
        vec![vec![MonsterType::Wolf, MonsterType::DireWolf]],
        vec![vec![MonsterType::Wolf, MonsterType::DireWolf]],
    ]
}

#[derive(Clone, Copy)]
pub enum MonsterType {
    TestMonsterWithDagger,
    TestMonsterWithLeatherArmor,
    Slime,
    Wolf,
    DireWolf,
}

impl MonsterType {
    pub fn to_monster(self) -> MonsterDefinition {
        match self {
            MonsterType::TestMonsterWithDagger => MonsterDefinition::new(
                "Test monster",
                AbilityScores::new(4, 2, 2),
                Gold::new(1),
                None,
                None,
                None,
                vec![WeaponType::Dagger],
                vec![],
                vec![],
            )
            .unwrap(),
            MonsterType::TestMonsterWithLeatherArmor => MonsterDefinition::new(
                "Test monster",
                AbilityScores::new(4, 2, 2),
                Gold::new(1),
                None,
                None,
                None,
                vec![],
                vec![ArmorType::Leather],
                vec![],
            )
            .unwrap(),
            MonsterType::Slime => MonsterDefinition::new(
                "Slime",
                AbilityScores::new(4, 2, 2),
                Gold::new(5),
                None,
                None,
                None,
                vec![],
                vec![ArmorType::Leather],
                vec![],
            )
            .unwrap(),
            MonsterType::Wolf => MonsterDefinition::new(
                "Wolf",
                AbilityScores::new(8, 9, 9),
                Gold::new(5),
                None,
                None,
                None,
                vec![],
                vec![],
                vec![LevelUpChoice {
                    ability_increment: AbilityType::Dexterity,
                    class: ClassType::Monster,
                }],
            )
            .unwrap(),
            MonsterType::DireWolf => MonsterDefinition::new(
                "Dire wolf",
                AbilityScores::new(8, 9, 9),
                Gold::new(5),
                None,
                None,
                None,
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
            )
            .unwrap(),
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
    pub weapon_inventory: Vec<WeaponType>,
    pub armor_inventory: Vec<ArmorType>,
    pub levels: Vec<LevelUpChoice>,
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
        weapon_inventory: Vec<WeaponType>,
        armor_inventory: Vec<ArmorType>,
        levels: Vec<LevelUpChoice>,
    ) -> Result<Self, String> {
        if let Some(ref weapon) = equipped_weapon {
            if !weapon_inventory.contains(weapon) {
                return Err("Equipped weapon not in inventory".into());
            }
        }
        if let Some(ref offhand) = equipped_offhand {
            if !weapon_inventory.contains(offhand) {
                return Err("Equipped offhand not in inventory".into());
            }
        }
        if let Some(ref armor) = equipped_armor {
            if !armor_inventory.contains(armor) {
                return Err("Equipped armor not in inventory".into());
            }
        }

        Ok(Self {
            name: name.to_string(),
            base_ability_scores,
            gold,
            experience: Experience::new(0),
            equipped_weapon,
            equipped_offhand,
            equipped_armor,
            weapon_inventory,
            armor_inventory,
            levels,
        })
    }
}

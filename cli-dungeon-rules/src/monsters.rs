use rand::{
    rngs::ThreadRng,
    seq::{IndexedRandom, IteratorRandom},
};

use crate::{
    Dice,
    abilities::{AbilityScores, AbilityType},
    armor::ArmorType,
    classes::{ClassType, LevelUpChoice},
    conditions::ActiveCondition,
    items::ItemType,
    jewelry::JewelryType,
    roll_success,
    types::{Constitution, Dexterity, Gold, Level, Strength},
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
        vec![
            vec![MonsterType::BeastLevel00],
            vec![MonsterType::BeastLevel00, MonsterType::BeastLevel00],
            vec![MonsterType::BeastLevel01],
            vec![MonsterType::BanditLevel0],
        ],
        vec![
            vec![MonsterType::Wolf],
            vec![MonsterType::BanditLevel1],
            vec![MonsterType::BeastLevel01, MonsterType::BeastLevel01],
        ],
        vec![
            vec![MonsterType::Wolf, MonsterType::Wolf],
            vec![MonsterType::GiantSpider],
            vec![MonsterType::DireWolf],
            vec![MonsterType::BanditLevel1, MonsterType::BanditLevel1],
        ],
        vec![
            vec![MonsterType::GiantToad],
            vec![MonsterType::Wolf, MonsterType::DireWolf],
            vec![MonsterType::BanditLevel2, MonsterType::BanditLevel2],
            vec![MonsterType::BanditLevel3],
        ],
        vec![
            vec![MonsterType::BanditLevel3, MonsterType::BanditLevel3],
            vec![MonsterType::BanditLevel4],
        ],
        vec![
            vec![MonsterType::BanditLevel4, MonsterType::BanditLevel4],
            vec![MonsterType::BanditLevel4Dex, MonsterType::BanditLevel4],
            vec![MonsterType::BanditLevel4Dex, MonsterType::BanditLevel4Dex],
            vec![MonsterType::BanditLevel5],
            vec![MonsterType::BanditLevel5Dex],
        ],
        vec![
            vec![MonsterType::BanditLevel5, MonsterType::BanditLevel5],
            vec![MonsterType::BanditLevel5Dex, MonsterType::BanditLevel5],
            vec![MonsterType::BanditLevel5Dex, MonsterType::BanditLevel5Dex],
            vec![MonsterType::BanditLevel6],
            vec![MonsterType::BanditLevel6Dex],
        ],
        vec![
            vec![MonsterType::BanditLevel6, MonsterType::BanditLevel6],
            vec![MonsterType::BanditLevel6Dex, MonsterType::BanditLevel6],
            vec![MonsterType::BanditLevel6Dex, MonsterType::BanditLevel6Dex],
            vec![MonsterType::BanditLevel7],
            vec![MonsterType::BanditLevel7Dex],
        ],
        vec![
            vec![MonsterType::BanditLevel7, MonsterType::BanditLevel7],
            vec![MonsterType::BanditLevel7Dex, MonsterType::BanditLevel7],
            vec![MonsterType::BanditLevel7Dex, MonsterType::BanditLevel7Dex],
            vec![MonsterType::BanditLevel8],
            vec![MonsterType::BanditLevel8Dex],
        ],
        vec![
            vec![MonsterType::BanditLevel8, MonsterType::BanditLevel8],
            vec![MonsterType::BanditLevel8Dex, MonsterType::BanditLevel8],
            vec![MonsterType::BanditLevel8Dex, MonsterType::BanditLevel8Dex],
            vec![MonsterType::BanditLevel9],
            vec![MonsterType::BanditLevel9Dex],
        ],
        vec![
            vec![MonsterType::BanditLevel9, MonsterType::BanditLevel9],
            vec![MonsterType::BanditLevel9Dex, MonsterType::BanditLevel9],
            vec![MonsterType::BanditLevel9Dex, MonsterType::BanditLevel9Dex],
            vec![MonsterType::BanditLevel10],
            vec![MonsterType::BanditLevel10Dex],
        ],
        vec![
            vec![MonsterType::BanditLevel10, MonsterType::BanditLevel10],
            vec![MonsterType::BanditLevel10Dex, MonsterType::BanditLevel10],
            vec![MonsterType::BanditLevel10Dex, MonsterType::BanditLevel10Dex],
            vec![MonsterType::BanditLevel11],
            vec![MonsterType::BanditLevel11Dex],
        ],
        vec![
            vec![MonsterType::BanditLevel11, MonsterType::BanditLevel11],
            vec![MonsterType::BanditLevel11Dex, MonsterType::BanditLevel11],
            vec![MonsterType::BanditLevel11Dex, MonsterType::BanditLevel11Dex],
            vec![MonsterType::BanditLevel12],
            vec![MonsterType::BanditLevel12Dex],
        ],
    ]
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum MonsterType {
    TestMonster,
    TestMonsterWithDagger,
    TestMonsterWithLeatherArmor,
    TestMonsterWithRingOfProtectionAndStone,
    BeastLevel00,
    BeastLevel01,
    BanditLevel0,
    BanditLevel1,
    BanditLevel2,
    BanditLevel3,
    BanditLevel4,
    BanditLevel4Dex,
    BanditLevel5,
    BanditLevel5Dex,
    BanditLevel6,
    BanditLevel6Dex,
    BanditLevel7,
    BanditLevel7Dex,
    BanditLevel8,
    BanditLevel8Dex,
    BanditLevel9,
    BanditLevel9Dex,
    BanditLevel10,
    BanditLevel10Dex,
    BanditLevel11,
    BanditLevel11Dex,
    BanditLevel12,
    BanditLevel12Dex,
    Wolf,
    DireWolf,
    GiantSpider,
    GiantToad,
}

impl MonsterType {
    pub fn to_monster(self) -> MonsterDefinition {
        match self {
            MonsterType::BanditLevel0 => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 0, 0),
                Gold::new(10),
                vec![Some(WeaponType::Dagger)],
                vec![Some(WeaponType::MonsterNone)],
                vec![Some(ArmorType::Leather)],
                vec![(WeaponType::Dagger, Some(Dice::D4))],
                vec![(ArmorType::Leather, Some(Dice::D4))],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D6)),
                ],
            ),
            MonsterType::BanditLevel1 => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 0, 1),
                Gold::new(20),
                vec![Some(WeaponType::Dagger), Some(WeaponType::Shortsword)],
                vec![Some(WeaponType::Dagger), None],
                vec![Some(ArmorType::Leather), Some(ArmorType::ChainShirt)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D8)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::ChainShirt, Some(Dice::D8)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel2 => MonsterDefinition::new_simple(
                &["Bandit"],
                (2, 0, 0),
                Gold::new(50),
                vec![Some(WeaponType::Dagger), Some(WeaponType::Shortsword)],
                vec![
                    Some(WeaponType::Dagger),
                    Some(WeaponType::Shortsword),
                    Some(WeaponType::Shield),
                    None,
                ],
                vec![Some(ArmorType::Leather), Some(ArmorType::StudedLeather)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Rapier, Some(Dice::D8)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::StudedLeather, Some(Dice::D8)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel3 => MonsterDefinition::new_simple(
                &["Bandit"],
                (2, 0, 1),
                Gold::new(50),
                vec![Some(WeaponType::Shortsword)],
                vec![Some(WeaponType::Shield), None],
                vec![
                    Some(ArmorType::Leather),
                    Some(ArmorType::ChainShirt),
                    Some(ArmorType::BreastPlate),
                ],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Longsword, Some(Dice::D8)),
                    (WeaponType::Rapier, Some(Dice::D8)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::ChainShirt, Some(Dice::D8)),
                    (ArmorType::BreastPlate, Some(Dice::D8)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel4 => MonsterDefinition::new_simple(
                &["Bandit"],
                (4, 0, 0),
                Gold::new(100),
                vec![
                    Some(WeaponType::Dagger),
                    Some(WeaponType::Shortsword),
                    Some(WeaponType::Longsword),
                ],
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Shield)],
                vec![Some(ArmorType::Leather), Some(ArmorType::StudedLeather)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Rapier, Some(Dice::D6)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Longsword, Some(Dice::D6)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::StudedLeather, Some(Dice::D4)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel4Dex => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 2, 2),
                Gold::new(100),
                vec![
                    Some(WeaponType::Dagger),
                    Some(WeaponType::Shortsword),
                    Some(WeaponType::Rapier),
                ],
                vec![Some(WeaponType::Shortsword)],
                vec![Some(ArmorType::Leather), Some(ArmorType::StudedLeather)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Rapier, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::StudedLeather, Some(Dice::D4)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel5 => MonsterDefinition::new_simple(
                &["Bandit"],
                (4, 0, 1),
                Gold::new(100),
                vec![Some(WeaponType::Longsword)],
                vec![Some(WeaponType::Shield), None],
                vec![
                    Some(ArmorType::ChainShirt),
                    Some(ArmorType::BreastPlate),
                    Some(ArmorType::HalfPlate),
                ],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Longsword, Some(Dice::D4)),
                ],
                vec![
                    (ArmorType::ChainShirt, Some(Dice::D6)),
                    (ArmorType::BreastPlate, Some(Dice::D6)),
                    (ArmorType::HalfPlate, Some(Dice::D6)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel5Dex => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 4, 1),
                Gold::new(100),
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Rapier)],
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Dagger)],
                vec![Some(ArmorType::Leather), Some(ArmorType::StudedLeather)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Rapier, Some(Dice::D4)),
                ],
                vec![(ArmorType::StudedLeather, Some(Dice::D6))],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel6 => MonsterDefinition::new_simple(
                &["Bandit"],
                (6, 0, 0),
                Gold::new(100),
                vec![Some(WeaponType::Longsword), Some(WeaponType::GreatSword)],
                vec![None],
                vec![
                    Some(ArmorType::ChainShirt),
                    Some(ArmorType::BreastPlate),
                    Some(ArmorType::HalfPlate),
                ],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Longsword, Some(Dice::D4)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::ChainShirt, Some(Dice::D6)),
                    (ArmorType::BreastPlate, Some(Dice::D6)),
                    (ArmorType::HalfPlate, Some(Dice::D6)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel6Dex => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 4, 2),
                Gold::new(100),
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Rapier)],
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Dagger)],
                vec![Some(ArmorType::Leather), Some(ArmorType::StudedLeather)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Rapier, Some(Dice::D4)),
                ],
                vec![(ArmorType::StudedLeather, Some(Dice::D6))],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel7 => MonsterDefinition::new_simple(
                &["Bandit"],
                (6, 0, 1),
                Gold::new(200),
                vec![
                    Some(WeaponType::Longsword),
                    Some(WeaponType::GreatSword),
                    Some(WeaponType::GreatAxe),
                ],
                vec![None],
                vec![
                    Some(ArmorType::ChainShirt),
                    Some(ArmorType::BreastPlate),
                    Some(ArmorType::HalfPlate),
                    Some(ArmorType::ChainMail),
                ],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Longsword, Some(Dice::D4)),
                    (WeaponType::GreatSword, Some(Dice::D8)),
                    (WeaponType::GreatAxe, Some(Dice::D8)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::ChainShirt, Some(Dice::D6)),
                    (ArmorType::BreastPlate, Some(Dice::D6)),
                    (ArmorType::HalfPlate, Some(Dice::D6)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel7Dex => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 4, 3),
                Gold::new(200),
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Rapier)],
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Dagger)],
                vec![Some(ArmorType::Leather), Some(ArmorType::StudedLeather)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Rapier, Some(Dice::D4)),
                ],
                vec![(ArmorType::StudedLeather, Some(Dice::D6))],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel8 => MonsterDefinition::new_simple(
                &["Bandit"],
                (6, 0, 2),
                Gold::new(200),
                vec![
                    Some(WeaponType::Longsword),
                    Some(WeaponType::GreatSword),
                    Some(WeaponType::GreatAxe),
                ],
                vec![None],
                vec![
                    Some(ArmorType::ChainShirt),
                    Some(ArmorType::BreastPlate),
                    Some(ArmorType::HalfPlate),
                    Some(ArmorType::ChainMail),
                ],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Longsword, Some(Dice::D4)),
                    (WeaponType::GreatSword, Some(Dice::D8)),
                    (WeaponType::GreatAxe, Some(Dice::D8)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::ChainShirt, Some(Dice::D6)),
                    (ArmorType::BreastPlate, Some(Dice::D6)),
                    (ArmorType::HalfPlate, Some(Dice::D6)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel8Dex => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 6, 2),
                Gold::new(200),
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Rapier)],
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Dagger)],
                vec![Some(ArmorType::Leather), Some(ArmorType::StudedLeather)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Rapier, Some(Dice::D4)),
                ],
                vec![(ArmorType::StudedLeather, Some(Dice::D6))],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel9 => MonsterDefinition::new_simple(
                &["Bandit"],
                (6, 0, 3),
                Gold::new(200),
                vec![
                    Some(WeaponType::Longsword),
                    Some(WeaponType::GreatSword),
                    Some(WeaponType::GreatAxe),
                ],
                vec![None],
                vec![
                    Some(ArmorType::ChainShirt),
                    Some(ArmorType::BreastPlate),
                    Some(ArmorType::HalfPlate),
                    Some(ArmorType::ChainMail),
                ],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Longsword, Some(Dice::D4)),
                    (WeaponType::GreatSword, Some(Dice::D8)),
                    (WeaponType::GreatAxe, Some(Dice::D8)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::ChainShirt, Some(Dice::D6)),
                    (ArmorType::BreastPlate, Some(Dice::D6)),
                    (ArmorType::HalfPlate, Some(Dice::D6)),
                    (ArmorType::ChainMail, Some(Dice::D6)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel9Dex => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 6, 3),
                Gold::new(200),
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Rapier)],
                vec![Some(WeaponType::Shortsword)],
                vec![Some(ArmorType::Leather), Some(ArmorType::StudedLeather)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Rapier, Some(Dice::D4)),
                ],
                vec![(ArmorType::StudedLeather, Some(Dice::D6))],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel10 => MonsterDefinition::new_simple(
                &["Bandit"],
                (6, 0, 4),
                Gold::new(200),
                vec![
                    Some(WeaponType::Longsword),
                    Some(WeaponType::GreatSword),
                    Some(WeaponType::GreatAxe),
                ],
                vec![None],
                vec![
                    Some(ArmorType::ChainShirt),
                    Some(ArmorType::BreastPlate),
                    Some(ArmorType::HalfPlate),
                    Some(ArmorType::ChainMail),
                    Some(ArmorType::Splint),
                ],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Longsword, Some(Dice::D4)),
                    (WeaponType::GreatSword, Some(Dice::D8)),
                    (WeaponType::GreatAxe, Some(Dice::D8)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::ChainShirt, Some(Dice::D6)),
                    (ArmorType::BreastPlate, Some(Dice::D6)),
                    (ArmorType::HalfPlate, Some(Dice::D6)),
                    (ArmorType::ChainMail, Some(Dice::D6)),
                    (ArmorType::Splint, Some(Dice::D6)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel10Dex => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 6, 4),
                Gold::new(200),
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Rapier)],
                vec![Some(WeaponType::Shortsword)],
                vec![Some(ArmorType::Leather), Some(ArmorType::StudedLeather)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Rapier, Some(Dice::D4)),
                ],
                vec![(ArmorType::StudedLeather, Some(Dice::D6))],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel11 => MonsterDefinition::new_simple(
                &["Bandit"],
                (6, 0, 5),
                Gold::new(200),
                vec![
                    Some(WeaponType::Longsword),
                    Some(WeaponType::GreatSword),
                    Some(WeaponType::GreatAxe),
                ],
                vec![None],
                vec![
                    Some(ArmorType::ChainShirt),
                    Some(ArmorType::BreastPlate),
                    Some(ArmorType::HalfPlate),
                    Some(ArmorType::ChainMail),
                    Some(ArmorType::Splint),
                ],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Longsword, Some(Dice::D4)),
                    (WeaponType::GreatSword, Some(Dice::D8)),
                    (WeaponType::GreatAxe, Some(Dice::D8)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::ChainShirt, Some(Dice::D6)),
                    (ArmorType::BreastPlate, Some(Dice::D6)),
                    (ArmorType::HalfPlate, Some(Dice::D6)),
                    (ArmorType::ChainMail, Some(Dice::D6)),
                    (ArmorType::Splint, Some(Dice::D6)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel11Dex => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 6, 5),
                Gold::new(200),
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Rapier)],
                vec![Some(WeaponType::Shortsword)],
                vec![Some(ArmorType::Leather), Some(ArmorType::StudedLeather)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Rapier, Some(Dice::D4)),
                ],
                vec![(ArmorType::StudedLeather, Some(Dice::D6))],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel12 => MonsterDefinition::new_simple(
                &["Bandit"],
                (6, 0, 6),
                Gold::new(200),
                vec![
                    Some(WeaponType::Longsword),
                    Some(WeaponType::GreatSword),
                    Some(WeaponType::GreatAxe),
                ],
                vec![None],
                vec![
                    Some(ArmorType::ChainShirt),
                    Some(ArmorType::BreastPlate),
                    Some(ArmorType::HalfPlate),
                    Some(ArmorType::ChainMail),
                    Some(ArmorType::Splint),
                ],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Longsword, Some(Dice::D4)),
                    (WeaponType::GreatSword, Some(Dice::D8)),
                    (WeaponType::GreatAxe, Some(Dice::D8)),
                ],
                vec![
                    (ArmorType::Leather, Some(Dice::D4)),
                    (ArmorType::ChainShirt, Some(Dice::D6)),
                    (ArmorType::BreastPlate, Some(Dice::D6)),
                    (ArmorType::HalfPlate, Some(Dice::D6)),
                    (ArmorType::ChainMail, Some(Dice::D6)),
                    (ArmorType::Splint, Some(Dice::D6)),
                ],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BanditLevel12Dex => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 6, 6),
                Gold::new(200),
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Rapier)],
                vec![Some(WeaponType::Shortsword)],
                vec![Some(ArmorType::Leather), Some(ArmorType::StudedLeather)],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Rapier, Some(Dice::D4)),
                ],
                vec![(ArmorType::StudedLeather, Some(Dice::D6))],
                vec![],
                vec![
                    (ItemType::ScrollOfWeaken, Some(Dice::D8)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                    (ItemType::PotionOfHealing, Some(Dice::D4)),
                ],
            ),
            MonsterType::BeastLevel00 => MonsterDefinition::new_simple(
                &["Rat", "Lizard", "Pig", "Frog", "Crap"],
                (0, 0, 0),
                Gold::new(5),
                vec![None],
                vec![Some(WeaponType::MonsterNone)],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
            MonsterType::BeastLevel01 => MonsterDefinition::new_simple(
                &["Badger", "Goat", "Weasel"],
                (0, 0, 0),
                Gold::new(10),
                vec![None],
                vec![Some(WeaponType::MonsterD4)],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
            MonsterType::Wolf => MonsterDefinition::new_simple(
                &["Wolf"],
                (0, 1, 0),
                Gold::new(10),
                vec![None],
                vec![Some(WeaponType::MonsterNone)],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
            MonsterType::DireWolf => MonsterDefinition::new_simple(
                &["Dire wolf"],
                (0, 2, 0),
                Gold::new(30),
                vec![None],
                vec![None],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![(ItemType::PotionOfHealing, Some(Dice::D4))],
            ),
            MonsterType::GiantSpider => MonsterDefinition::new_simple(
                &["Giant spider"],
                (1, 1, 0),
                Gold::new(70),
                vec![Some(WeaponType::MonsterD8)],
                vec![Some(WeaponType::MonsterNone)],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![(ItemType::PotionOfHealing, Some(Dice::D4))],
            ),
            MonsterType::GiantToad => MonsterDefinition::new_simple(
                &["Giant toad"],
                (2, 1, 0),
                Gold::new(100),
                vec![Some(WeaponType::MonsterD10)],
                vec![Some(WeaponType::MonsterNone)],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![(ItemType::PotionOfHealing, Some(Dice::D4))],
            ),
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
        }
    }
}

pub struct MonsterDefinition {
    pub name: String,
    pub base_ability_scores: AbilityScores,
    pub gold: Gold,
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
    pub fn new_simple(
        name: &[&str],
        levels: (i16, i16, i16),
        gold: Gold,
        equipped_weapon: Vec<Option<WeaponType>>,
        equipped_offhand: Vec<Option<WeaponType>>,
        equipped_armor: Vec<Option<ArmorType>>,
        weapon_inventory: Vec<(WeaponType, Option<Dice>)>,
        armor_inventory: Vec<(ArmorType, Option<Dice>)>,
        jewelry_inventory: Vec<(JewelryType, Option<Dice>)>,
        item_inventory: Vec<(ItemType, Option<Dice>)>,
    ) -> Self {
        let mut rng = rand::rng();
        let weapon_inventory = roll_items(weapon_inventory);
        let armor_inventory = roll_items(armor_inventory);
        let jewelry_inventory = roll_items(jewelry_inventory);
        let item_inventory = roll_items(item_inventory);

        let level = levels.0 + levels.1 + levels.2;

        let str = 8 + 2 * levels.0;
        let dex = 8 + 2 * levels.1;
        let con = 8 + 2 * levels.2 - level * 2;

        let base_ability_scores = AbilityScores {
            strength: Strength::new(str),
            dexterity: Dexterity::new(dex),
            constitution: Constitution::new(con),
        };

        let levels = (0..level)
            .map(|_| LevelUpChoice {
                ability_increment: AbilityType::Constitution,
                class: ClassType::Monster,
            })
            .collect();

        let name = name.choose(&mut rand::rng()).unwrap();

        let equipped_weapon = roll_equipped(equipped_weapon, &mut rng);
        let equipped_offhand = roll_equipped(equipped_offhand, &mut rng);
        let equipped_armor = roll_equipped(equipped_armor, &mut rng);

        Self {
            name: name.to_string(),
            base_ability_scores,
            gold,
            equipped_weapon,
            equipped_offhand,
            equipped_armor,
            equipped_jewelry: vec![],
            weapon_inventory,
            armor_inventory,
            jewelry_inventory,
            item_inventory,
            levels,
            active_conditions: vec![],
        }
    }

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
        let weapon_inventory = roll_items(weapon_inventory);
        let armor_inventory = roll_items(armor_inventory);
        let jewelry_inventory = roll_items(jewelry_inventory);
        let item_inventory = roll_items(item_inventory);

        Self {
            name: name.to_string(),
            base_ability_scores,
            gold,
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

fn roll_items<T>(items: Vec<(T, Option<Dice>)>) -> Vec<T> {
    items
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
        .collect()
}

fn roll_equipped<T>(items: Vec<Option<T>>, rng: &mut ThreadRng) -> Option<T> {
    items.into_iter().choose(rng).unwrap()
}

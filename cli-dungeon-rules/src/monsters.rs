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
        vec![vec![MonsterType::Wolf], vec![MonsterType::BanditLevel1]],
        vec![
            vec![MonsterType::Wolf, MonsterType::DireWolf],
            vec![MonsterType::Wolf, MonsterType::Wolf],
            vec![MonsterType::GiantSpider],
            vec![MonsterType::GiantToad],
            vec![MonsterType::BanditLevel2Str],
            vec![MonsterType::BanditLevel2Dex],
            vec![MonsterType::BanditLevel1, MonsterType::BanditLevel1],
        ],
        vec![
            vec![MonsterType::BanditLevel2Str, MonsterType::BanditLevel2Dex],
            vec![MonsterType::BanditLevel3Str],
            vec![MonsterType::BanditLevel3Dex],
            vec![MonsterType::BanditLevel3Str2H],
        ],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
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
    BanditLevel2Dex,
    BanditLevel2Str,
    BanditLevel3Dex,
    BanditLevel3Str,
    BanditLevel3Str2H,
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
                (-2, -2, -2),
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
                0,
            ),
            MonsterType::BanditLevel1 => MonsterDefinition::new_simple(
                &["Bandit"],
                (2, 2, 2),
                Gold::new(10),
                vec![Some(WeaponType::Dagger), Some(WeaponType::Shortsword)],
                vec![Some(WeaponType::Dagger), Some(WeaponType::Shield), None],
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
                1,
            ),
            MonsterType::BanditLevel2Dex => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 4, 4),
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
                2,
            ),
            MonsterType::BanditLevel2Str => MonsterDefinition::new_simple(
                &["Bandit"],
                (4, 0, 4),
                Gold::new(50),
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Longsword)],
                vec![Some(WeaponType::Shield), None],
                vec![
                    Some(ArmorType::Leather),
                    Some(ArmorType::ChainShirt),
                    Some(ArmorType::ChainMail),
                ],
                vec![
                    (WeaponType::Dagger, Some(Dice::D4)),
                    (WeaponType::Shortsword, Some(Dice::D4)),
                    (WeaponType::Longsword, Some(Dice::D4)),
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
                2,
            ),
            MonsterType::BanditLevel3Dex => MonsterDefinition::new_simple(
                &["Bandit"],
                (0, 6, 4),
                Gold::new(100),
                vec![
                    Some(WeaponType::Dagger),
                    Some(WeaponType::Shortsword),
                    Some(WeaponType::Rapier),
                ],
                vec![Some(WeaponType::Shortsword), Some(WeaponType::Shield)],
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
                3,
            ),
            MonsterType::BanditLevel3Str => MonsterDefinition::new_simple(
                &["Bandit"],
                (6, 0, 4),
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
                3,
            ),
            MonsterType::BanditLevel3Str2H => MonsterDefinition::new_simple(
                &["Bandit"],
                (14, 0, 0),
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
                3,
            ),
            MonsterType::BeastLevel00 => MonsterDefinition::new_simple(
                &["Rat", "Lizard", "Pig", "Frog", "Crap"],
                (-2, -2, -2),
                Gold::new(5),
                vec![None],
                vec![Some(WeaponType::MonsterNone)],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![],
                0,
            ),
            MonsterType::BeastLevel01 => MonsterDefinition::new_simple(
                &["Badger", "Goat", "Weasel"],
                (-2, 1, -2),
                Gold::new(10),
                vec![None],
                vec![Some(WeaponType::MonsterD4)],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![],
                0,
            ),
            MonsterType::Wolf => MonsterDefinition::new_simple(
                &["Wolf"],
                (-1, 0, -1),
                Gold::new(5),
                vec![None],
                vec![Some(WeaponType::MonsterNone)],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![],
                0,
            ),
            MonsterType::DireWolf => MonsterDefinition::new_simple(
                &["Dire wolf"],
                (0, 0, 0),
                Gold::new(5),
                vec![None],
                vec![None],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![(ItemType::PotionOfHealing, Some(Dice::D4))],
                1,
            ),
            MonsterType::GiantSpider => MonsterDefinition::new_simple(
                &["Giant Spider"],
                (2, 3, 1),
                Gold::new(20),
                vec![None],
                vec![Some(WeaponType::MonsterD8P2)],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![(ItemType::PotionOfHealing, Some(Dice::D4))],
                2,
            ),
            MonsterType::GiantToad => MonsterDefinition::new_simple(
                &["Giant Spider"],
                (2, 1, 1),
                Gold::new(20),
                vec![None],
                vec![Some(WeaponType::MonsterD10P2)],
                vec![None],
                vec![],
                vec![],
                vec![],
                vec![(ItemType::PotionOfHealing, Some(Dice::D4))],
                2,
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
        bonus: (i16, i16, i16),
        gold: Gold,
        equipped_weapon: Vec<Option<WeaponType>>,
        equipped_offhand: Vec<Option<WeaponType>>,
        equipped_armor: Vec<Option<ArmorType>>,
        weapon_inventory: Vec<(WeaponType, Option<Dice>)>,
        armor_inventory: Vec<(ArmorType, Option<Dice>)>,
        jewelry_inventory: Vec<(JewelryType, Option<Dice>)>,
        item_inventory: Vec<(ItemType, Option<Dice>)>,
        level: u16,
    ) -> Self {
        let mut rng = rand::rng();
        let weapon_inventory = roll_items(weapon_inventory);
        let armor_inventory = roll_items(armor_inventory);
        let jewelry_inventory = roll_items(jewelry_inventory);
        let item_inventory = roll_items(item_inventory);

        let str = 10 + 2 * bonus.0;
        let dex = 10 + 2 * bonus.1;
        let con = 10 + 2 * bonus.2 - level as i16;

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

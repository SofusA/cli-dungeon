use crate::{
    abilities::{AbilityScores, AbilityType},
    armor::ArmorType,
    classes::{ClassType, LevelUpChoice},
    types::{Experience, Gold},
    weapons::WeaponType,
};

pub enum MonsterType {
    Wolf,
    DireWolf,
}

impl MonsterType {
    pub fn to_monster(self) -> MonsterDefinition {
        match self {
            MonsterType::Wolf => MonsterDefinition::new(
                "Wolf",
                AbilityScores::new(8, 9, 9),
                Gold::new(5),
                Experience::new(0),
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
                Experience::new(0),
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
        experience: Experience,
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
            experience,
            equipped_weapon,
            equipped_offhand,
            equipped_armor,
            weapon_inventory,
            armor_inventory,
            levels,
        })
    }
}

use std::collections::HashSet;

use cli_dungeon_database::{CharacterInfo, DatabaseError};
use cli_dungeon_rules::{
    Dice, Hit,
    abilities::{AbilityScores, AbilityType},
    armor::ArmorType,
    classes::{ClassType, LevelUpChoice},
    experience_gain, roll,
    types::{Experience, Gold},
    weapons::WeaponType,
};
use futures::future::join_all;
use rand::seq::{IndexedRandom, IteratorRandom};
use thiserror::Error;

pub mod character;
pub mod shop;

pub async fn play(
    force: bool,
    character_info: CharacterInfo,
) -> Result<Option<Vec<TurnOutcome>>, GameError> {
    if !cli_dungeon_database::validate_player(&character_info).await? {
        return Err(GameError::Dead);
    };

    if roll(&Dice::D20) == 4 || force {
        return Ok(Some(encountor(character_info.id).await));
    }
    Ok(None)
}

#[derive(Debug, Clone)]
pub enum TurnOutcome {
    Miss(String),
    Attack(Attack),
    Hit(Hit),
    Death(String),
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub attacker_name: String,
    pub attacked_name: String,
}

#[derive(Clone, Copy)]
struct FightParticipant {
    id: i64,
    party_id: i64,
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Character is dead")]
    Dead,

    #[error("Ability scores must sum to 10")]
    AbilitySumError,

    #[error("Weapon cannot be wielded in offhand")]
    NotOffHandWeapon,

    #[error("Your character is not strong enough")]
    InsufficientStrength,

    #[error("Insufficient gold")]
    InsufficientGold,

    #[error("Insufficient experience for level up")]
    InsufficientExperience,

    #[error("Unknown Item. Spelling error?")]
    UnknownItem,

    #[error("Unknown weapon. Spelling error?")]
    UnknownWeapon,

    #[error("Unknown armor. Spelling error?")]
    UnknownArmor,

    #[error("Unknown class. Spelling error?")]
    UnknownClass,

    #[error(transparent)]
    Database(#[from] DatabaseError),
}

async fn encountor(player_id: i64) -> Vec<TurnOutcome> {
    let wolf_id = cli_dungeon_database::create_character(
        "Wolf",
        AbilityScores::new(8, 9, 9),
        Gold(5),
        Experience(0),
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
    .await
    .id;
    let dire_wolf_id = cli_dungeon_database::create_character(
        "Dire wolf",
        AbilityScores::new(8, 9, 9),
        Gold(5),
        Experience(0),
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
    .await
    .id;

    let player = FightParticipant {
        id: player_id,
        party_id: 1,
    };
    let wolf = FightParticipant {
        id: wolf_id,
        party_id: 2,
    };
    let dire_wolf = FightParticipant {
        id: dire_wolf_id,
        party_id: 2,
    };

    let outcome = fight(vec![player, wolf, dire_wolf]).await;

    cli_dungeon_database::delete_character(dire_wolf_id).await;
    cli_dungeon_database::delete_character(wolf_id).await;

    outcome
}

async fn fight(participants: Vec<FightParticipant>) -> Vec<TurnOutcome> {
    let mut outcome_list: Vec<TurnOutcome> = vec![];

    let mut dead: Vec<i64> = vec![];

    let mut rotation: Vec<_> = participants
        .into_iter()
        .map(|participant| (participant, roll(&Dice::D20)))
        .collect();

    rotation.sort_by_key(|initiative| initiative.1);
    rotation.reverse();

    let mut participant_rotation: Vec<_> = rotation
        .into_iter()
        .map(|initiative| initiative.0)
        .collect();

    loop {
        for character_inititiative in participant_rotation.clone() {
            let character = cli_dungeon_database::get_character(character_inititiative.id)
                .await
                .unwrap();

            let other_character_participant = participant_rotation
                .iter()
                .filter(|character| character.party_id != character_inititiative.party_id)
                .filter(|character| character.id != character_inititiative.id)
                .choose(&mut rand::rng())
                .unwrap();

            let mut other_character =
                cli_dungeon_database::get_character(other_character_participant.id)
                    .await
                    .unwrap();

            let outcome = other_character.attacked(&character.attack_stats());
            outcome_list.push(TurnOutcome::Attack(Attack {
                attacker_name: character.name.clone(),
                attacked_name: other_character.name.clone(),
            }));
            match outcome {
                Some(outcome) => {
                    outcome_list.push(TurnOutcome::Hit(outcome));

                    cli_dungeon_database::set_character_health(
                        other_character_participant.id,
                        other_character.current_health,
                    )
                    .await;

                    if !other_character.is_alive() {
                        outcome_list.push(TurnOutcome::Death(other_character.name));
                        participant_rotation.retain(|character| character.id != other_character.id);
                        dead.push(other_character.id);

                        let same_party: Vec<_> = participant_rotation
                            .iter()
                            .filter(|character| {
                                character.party_id == character_inititiative.party_id
                            })
                            .collect();

                        let experience_gained = Experience(
                            *experience_gain(other_character.level_up_choices)
                                / same_party.len() as u32,
                        );

                        for character_info in same_party {
                            let character = cli_dungeon_database::get_character(character_info.id)
                                .await
                                .unwrap();
                            let new_xp = character.experience + experience_gained;
                            cli_dungeon_database::set_character_experience(
                                character_info.id,
                                new_xp,
                            )
                            .await;
                        }
                    }
                }
                None => outcome_list.push(TurnOutcome::Miss(character.name)),
            }

            let parties_left = {
                let unique_party_ids: HashSet<i64> =
                    participant_rotation.iter().map(|p| p.party_id).collect();
                unique_party_ids.len()
            };

            if parties_left == 1 {
                let dead_characters: Vec<_> = join_all(
                    dead.into_iter()
                        .map(async |id| cli_dungeon_database::get_character(id).await.unwrap()),
                )
                .await;

                let total_gold: u16 = dead_characters
                    .iter()
                    .map(|character| *character.gold)
                    .sum();
                let split_gold = total_gold / participant_rotation.len() as u16;

                let weapon_loot: Vec<WeaponType> = dead_characters
                    .iter()
                    .flat_map(|character| character.weapon_inventory.clone())
                    .collect();
                let armor_loot: Vec<ArmorType> = dead_characters
                    .iter()
                    .flat_map(|character| character.armor_inventory.clone())
                    .collect();

                for character_id in participant_rotation.iter().map(|rotation| rotation.id) {
                    let character = cli_dungeon_database::get_character(character_id)
                        .await
                        .unwrap();
                    let new_gold = character.gold + Gold(split_gold);
                    cli_dungeon_database::set_character_gold(character_id, new_gold).await;
                }

                for weapon in weapon_loot {
                    let recipient = participant_rotation.choose(&mut rand::rng()).unwrap();

                    cli_dungeon_database::add_weapon_to_inventory(recipient.id, weapon)
                        .await
                        .unwrap();
                }
                for armor in armor_loot {
                    let recipient = participant_rotation.choose(&mut rand::rng()).unwrap();

                    cli_dungeon_database::add_armor_to_inventory(recipient.id, armor)
                        .await
                        .unwrap();
                }

                return outcome_list;
            }
        }
    }
}

use std::collections::HashSet;

use anyhow::{Result, bail};
use cli_dungeon_database::CharacterInfo;
use cli_dungeon_rules::{Dice, Hit, Hitable, roll};
use thiserror::Error;

pub async fn play(force: bool, character_info: CharacterInfo) -> Result<Option<Vec<TurnOutcome>>> {
    if !cli_dungeon_database::validate_player(&character_info).await? {
        bail!(GameError::Dead)
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
}

async fn encountor(player_id: i64) -> Vec<TurnOutcome> {
    let wolf_id = cli_dungeon_database::create_character("Wolf", 6, 11, Dice::D4, 3)
        .await
        .id;
    let dire_wolf_id = cli_dungeon_database::create_character("Dire wolf", 12, 12, Dice::D4, 3)
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
            let character = cli_dungeon_database::get_character(character_inititiative.id).await;

            let other_character_participant = participant_rotation
                .iter()
                .filter(|character| character.party_id != character_inititiative.party_id)
                .find(|character| character.id != character_inititiative.id)
                .unwrap();

            let mut other_character =
                cli_dungeon_database::get_character(other_character_participant.id).await;

            let outcome = other_character.attacked(&character.hit_bonus, &character.attack_dice);
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
                return outcome_list;
            }
        }
    }
}

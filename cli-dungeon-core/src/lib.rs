use character::get_character;
use cli_dungeon_database::{CharacterInfo, DatabaseError, Pool};
use cli_dungeon_rules::{
    AttackStats, Character, Dice, Encounter, Hit, Status,
    armor::ArmorType,
    experience_gain,
    items::ItemType,
    jewelry::JewelryType,
    roll,
    types::{Experience, Gold, Level, QuestPoint, Turn},
    weapons::WeaponType,
};
use futures::future::join_all;
use rand::seq::{IndexedRandom, IteratorRandom};
use std::collections::HashSet;
use thiserror::Error;

pub mod character;
pub mod shop;

pub enum PlayOutcome {
    NothingNew(Status),
    NewFight(Vec<TurnOutcome>),
}

async fn advance_turn(pool: &Pool, character: &Character) {
    let new_conditions: Vec<_> = character
        .active_conditions
        .clone()
        .into_iter()
        .filter(|condition| condition.duration < Turn::new(0))
        .map(|mut condition| {
            condition.duration -= Turn::new(1);
            condition
        })
        .collect();

    cli_dungeon_database::set_character_conditions(pool, character.id, new_conditions).await;
}

pub async fn play(
    pool: &Pool,
    force: bool,
    character_info: &CharacterInfo,
) -> Result<PlayOutcome, GameError> {
    let character = get_character(pool, character_info).await?;

    if character.quest_points == QuestPoint::new(100) {
        // TODO: Get the loooooot
        return Ok(PlayOutcome::NothingNew(character.status));
    }

    if let Status::Questing = character.status {
        if roll(&Dice::D20) == 4 || force {
            let outcome = new_encounter(pool, character_info.id).await;
            return Ok(PlayOutcome::NewFight(outcome));
        }
    }

    cli_dungeon_database::set_character_quest_points(
        pool,
        &character.id,
        character.quest_points + QuestPoint::new(1),
    )
    .await;

    advance_turn(pool, &character).await;

    Ok(PlayOutcome::NothingNew(character.status))
}

pub async fn get_encounter(
    pool: &Pool,
    character_info: &CharacterInfo,
) -> Result<Encounter, GameError> {
    let status = get_status(pool, character_info).await?;

    let Status::Fighting(encounter_id) = status else {
        return Err(GameError::NotFighting);
    };

    Ok(cli_dungeon_database::get_encounter(pool, &encounter_id).await?)
}

async fn get_status(pool: &Pool, character_info: &CharacterInfo) -> Result<Status, GameError> {
    if !cli_dungeon_database::validate_player(pool, character_info).await? {
        return Err(GameError::Dead);
    };

    Ok(
        cli_dungeon_database::get_character(pool, &character_info.id)
            .await?
            .status,
    )
}

async fn new_encounter(pool: &Pool, player_id: i64) -> Vec<TurnOutcome> {
    let player = cli_dungeon_database::get_character(pool, &player_id)
        .await
        .unwrap();

    let enemy_party_id = cli_dungeon_database::create_party(pool).await;

    let monsters: Vec<_> = join_all(
        cli_dungeon_rules::monsters::get_monster_encounter(player.level())
            .iter()
            .map(async |enemy| {
                cli_dungeon_database::create_monster(pool, *enemy, enemy_party_id)
                    .await
                    .id
            }),
    )
    .await;

    let mut participants = monsters.clone();
    participants.push(player_id);

    let mut initiative: Vec<_> = participants
        .into_iter()
        .map(|participant| (participant, roll(&Dice::D20)))
        .collect();
    initiative.sort_by_key(|initiative| initiative.1);
    initiative.reverse();

    let rotation: Vec<i64> = initiative
        .into_iter()
        .map(|initiative| initiative.0)
        .collect();

    let encounter_id = cli_dungeon_database::create_encounter(pool, rotation.clone()).await;

    for character_id in rotation.iter() {
        cli_dungeon_database::set_encounter_id(pool, character_id, Some(encounter_id)).await;
    }

    let mut outcome: Vec<TurnOutcome> = vec![];

    loop {
        let encounter = cli_dungeon_database::get_encounter(pool, &encounter_id)
            .await
            .unwrap();

        match encounter.rotation.first() {
            Some(turn) => {
                if turn.player {
                    break;
                }

                outcome.append(&mut monster_take_turn(pool, turn, &encounter).await);
            }
            None => break,
        }
    }

    outcome
}

pub enum Action {
    Attack,
}

pub enum BonusAction {
    OffHandAttack,
}

async fn monster_take_turn(
    pool: &Pool,
    monster: &Character,
    encounter: &Encounter,
) -> Vec<TurnOutcome> {
    let action = Action::Attack;
    let bonus_action = BonusAction::OffHandAttack;

    let target = encounter
        .rotation
        .iter()
        .filter(|character| character.party != monster.party)
        .map(|character| character.id)
        .choose(&mut rand::rng());

    character_take_turn(
        pool,
        monster,
        encounter,
        action,
        bonus_action,
        target,
        target,
    )
    .await
}

async fn handle_attack(
    pool: &Pool,
    active_character: &Character,
    attack_stats: &AttackStats,
    target: Option<i64>,
    rotation: &mut Vec<Character>,
    dead_list: &mut Vec<Character>,
) -> Vec<TurnOutcome> {
    let mut outcome_list = vec![];

    {
        let Some(target) = target else {
            outcome_list.push(TurnOutcome::Miss(active_character.name.clone()));
            return outcome_list;
        };

        if !rotation
            .iter()
            .map(|character| character.id)
            .collect::<Vec<_>>()
            .contains(&target)
        {
            outcome_list.push(TurnOutcome::Miss(active_character.name.clone()));
            return outcome_list;
        }
        let mut target = cli_dungeon_database::get_character(pool, &target)
            .await
            .unwrap();
        let action_outcome = target.attacked(attack_stats);

        match action_outcome {
            Some(outcome) => {
                outcome_list.push(TurnOutcome::Hit(outcome));
                cli_dungeon_database::set_character_health(pool, &target.id, target.current_health)
                    .await;

                if !target.is_alive() {
                    outcome_list.push(TurnOutcome::Death(target.name.clone()));

                    rotation.retain(|character| character.id != target.id);
                    let same_party: Vec<_> = rotation
                        .iter()
                        .filter(|character| character.party == active_character.party)
                        .collect();

                    let experience_gained = Experience::new(
                        *experience_gain(Level::new(target.level_up_choices.len() as u16))
                            / same_party.len() as u32,
                    );

                    dead_list.push(target);

                    for character_info in same_party {
                        let character =
                            cli_dungeon_database::get_character(pool, &character_info.id)
                                .await
                                .unwrap();
                        let new_xp = character.experience + experience_gained;
                        cli_dungeon_database::set_character_experience(
                            pool,
                            &character_info.id,
                            new_xp,
                        )
                        .await;
                    }
                }
            }
            None => {
                outcome_list.push(TurnOutcome::Miss(active_character.name.clone()));
            }
        }
    }
    outcome_list
}

async fn character_take_turn(
    pool: &Pool,
    active_character: &Character,
    encounter: &Encounter,
    action: Action,
    bonus_action: BonusAction,
    target: Option<i64>,
    bonus_action_target: Option<i64>,
) -> Vec<TurnOutcome> {
    let mut outcome_list = vec![];
    let mut new_dead_list = encounter.dead_characters.clone();
    let mut new_rotation = encounter.rotation.clone();

    outcome_list.push(TurnOutcome::StartTurn(active_character.name.clone()));

    match action {
        Action::Attack => {
            let mut outcome = handle_attack(
                pool,
                active_character,
                &active_character.attack_stats(),
                target,
                &mut new_rotation,
                &mut new_dead_list,
            )
            .await;
            outcome_list.append(&mut outcome);
        }
    }

    match bonus_action {
        BonusAction::OffHandAttack => {
            let mut outcome = handle_attack(
                pool,
                active_character,
                &active_character.off_hand_attack_stats(),
                bonus_action_target,
                &mut new_rotation,
                &mut new_dead_list,
            )
            .await;
            outcome_list.append(&mut outcome);
        }
    }

    let parties_left = {
        let unique_party_ids: HashSet<i64> = new_rotation.iter().map(|p| p.party).collect();
        unique_party_ids.len()
    };

    if parties_left == 1 {
        let total_gold: u16 = new_dead_list.iter().map(|character| *character.gold).sum();
        let split_gold = total_gold / new_rotation.len() as u16;

        let weapon_loot: Vec<WeaponType> = new_dead_list
            .iter()
            .flat_map(|character| character.weapon_inventory.clone())
            .collect();
        let armor_loot: Vec<ArmorType> = new_dead_list
            .iter()
            .flat_map(|character| character.armor_inventory.clone())
            .collect();
        let jewelry_loot: Vec<JewelryType> = new_dead_list
            .iter()
            .flat_map(|character| character.jewelry_inventory.clone())
            .collect();
        let item_loot: Vec<ItemType> = new_dead_list
            .iter()
            .flat_map(|character| character.item_inventory.clone())
            .collect();

        for character in new_rotation.iter() {
            let new_gold = character.gold + Gold::new(split_gold);
            cli_dungeon_database::set_character_gold(pool, &character.id, new_gold).await;
            cli_dungeon_database::set_character_status(pool, &character.id, Status::Questing).await;
        }

        for weapon in weapon_loot {
            let recipient = new_rotation.choose(&mut rand::rng()).unwrap();

            cli_dungeon_database::add_weapon_to_inventory(pool, &recipient.id, weapon)
                .await
                .unwrap();
        }
        for armor in armor_loot {
            let recipient = new_rotation.choose(&mut rand::rng()).unwrap();

            cli_dungeon_database::add_armor_to_inventory(pool, &recipient.id, armor)
                .await
                .unwrap();
        }
        for jewelry in jewelry_loot {
            let recipient = new_rotation.choose(&mut rand::rng()).unwrap();

            cli_dungeon_database::add_jewelry_to_inventory(pool, &recipient.id, jewelry)
                .await
                .unwrap();
        }
        for item in item_loot {
            let recipient = new_rotation.choose(&mut rand::rng()).unwrap();

            cli_dungeon_database::add_item_to_inventory(pool, &recipient.id, item)
                .await
                .unwrap();
        }
    }

    if let Some(first) = new_rotation.first().cloned() {
        new_rotation.remove(0);
        new_rotation.push(first);
    }

    cli_dungeon_database::update_encounter(
        pool,
        encounter.id,
        new_rotation.iter().map(|character| character.id).collect(),
        new_dead_list.iter().map(|character| character.id).collect(),
    )
    .await;

    advance_turn(pool, active_character).await;

    outcome_list
}

pub async fn take_turn(
    pool: &Pool,
    character_info: &CharacterInfo,
    action: Action,
    bonus_action: BonusAction,
    target: Option<i64>,
    bonus_action_target: Option<i64>,
) -> Result<Vec<TurnOutcome>, GameError> {
    let mut outcome = vec![];

    if !cli_dungeon_database::validate_player(pool, character_info).await? {
        return Err(GameError::Dead);
    };

    let active_character = cli_dungeon_database::get_character(pool, &character_info.id)
        .await
        .unwrap();

    let Status::Fighting(encounter_id) = active_character.status else {
        return Err(GameError::NotFighting);
    };

    let encounter = cli_dungeon_database::get_encounter(pool, &encounter_id)
        .await
        .unwrap();

    if encounter.rotation.first().unwrap().id != active_character.id {
        return Err(GameError::NotPlayerTurn);
    }

    outcome.append(
        &mut character_take_turn(
            pool,
            &active_character,
            &encounter,
            action,
            bonus_action,
            target,
            bonus_action_target,
        )
        .await,
    );

    loop {
        let encounter = cli_dungeon_database::get_encounter(pool, &encounter_id)
            .await
            .unwrap();

        match encounter.rotation.first() {
            Some(turn) => {
                if turn.player {
                    break;
                }

                outcome.append(&mut monster_take_turn(pool, turn, &encounter).await);
            }
            None => break,
        }
    }

    Ok(outcome)
}

#[derive(Debug, Clone)]
pub enum TurnOutcome {
    Miss(String),
    Attack(Attack),
    Hit(Hit),
    Death(String),
    StartTurn(String),
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub attacker_name: String,
    pub attacked_name: String,
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Character is dead")]
    Dead,

    #[error("Character is not in a fight")]
    NotFighting,

    #[error("It is not your turn!")]
    NotPlayerTurn,

    #[error("Ability scores must sum to 10")]
    AbilitySumError,

    #[error("Weapon cannot be wielded in offhand")]
    NotOffHandWeapon,

    #[error("Your character is not strong enough")]
    InsufficientStrength,

    #[error("Insufficient gold")]
    InsufficientGold,

    #[error("No short rests remaining")]
    InsufficientShortRests,

    #[error("Insufficient experience for level up")]
    InsufficientExperience,

    #[error("Unknown Item. Spelling error?")]
    UnknownItem,

    #[error("Unknown weapon. Spelling error?")]
    UnknownWeapon,

    #[error("Unknown armor. Spelling error?")]
    UnknownArmor,

    #[error("Unknown jewelry. Spelling error?")]
    UnknownJewelry,

    #[error("Unknown class. Spelling error?")]
    UnknownClass,

    #[error("Weapon not in inventory")]
    WeaponNotInInventory,
    #[error("Armor not in inventory")]
    ArmorNotInInventory,
    #[error("Weapon not in inventory")]
    JewelryNotInInventory,

    #[error("Too many jewelries equipped. Unequip one first")]
    TooManyJewelriesEquipped,

    #[error(transparent)]
    Database(#[from] DatabaseError),
}

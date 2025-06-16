use character::{get_character, validate_player};
use cli_dungeon_database::{CharacterInfo, Pool};
use cli_dungeon_rules::{
    Dice, Encounter, Status,
    loot::{self, Loot},
    roll,
    types::QuestPoint,
};
use errors::GameError;
use futures::future::join_all;
use turn::{TurnOutcome, advance_turn, monster_take_turn};

pub mod character;
pub mod errors;
pub mod shop;
pub mod turn;

pub enum PlayOutcome {
    NothingNew(Status),
    NewFight(Vec<TurnOutcome>),
    CompletedQuest(Loot),
}

pub async fn play(
    pool: &Pool,
    force: bool,
    character_info: &CharacterInfo,
) -> Result<PlayOutcome, GameError> {
    let character = get_character(pool, character_info).await?;

    if character.quest_points == QuestPoint::new(100) {
        let Loot {
            weapons,
            armor,
            items,
            jewelry,
        } = loot::get_loot(character.level());

        for weapon in weapons.iter() {
            cli_dungeon_database::add_weapon_to_inventory(pool, &character.id, *weapon).await?
        }
        for armor in armor.iter() {
            cli_dungeon_database::add_armor_to_inventory(pool, &character.id, *armor).await?
        }
        for item in items.iter() {
            cli_dungeon_database::add_item_to_inventory(pool, &character.id, *item).await?
        }
        for jewelry in jewelry.iter() {
            cli_dungeon_database::add_jewelry_to_inventory(pool, &character.id, *jewelry).await?
        }

        cli_dungeon_database::set_character_quest_points(pool, &character.id, QuestPoint::new(0))
            .await;

        let loot = Loot {
            weapons,
            armor,
            items,
            jewelry,
        };
        return Ok(PlayOutcome::CompletedQuest(loot));
    }

    if let Status::Questing = character.status
        && (roll(&Dice::D20) == 4 || force)
    {
        let outcome = new_encounter(pool, character_info.id).await;
        return Ok(PlayOutcome::NewFight(outcome));
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
    validate_player(pool, character_info).await?;

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

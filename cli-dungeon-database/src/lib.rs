use std::sync::OnceLock;

use cli_dungeon_rules::{
    Character,
    abilities::AbilityScores,
    armor::ArmorType,
    classes::LevelUpChoice,
    max_health,
    types::{Experience, Gold, HealthPoints, Level},
    weapons::WeaponType,
};
use serde_json::to_string;
use sqlx::types::Json;
use thiserror::Error;

static POOL: OnceLock<sqlx::Pool<sqlx::Sqlite>> = OnceLock::new();

macro_rules! acquire {
    () => {
        match POOL.get() {
            Some(pool) => pool.acquire().await.unwrap(),
            None => {
                init().await;
                POOL.get().unwrap().acquire().await.unwrap()
            }
        }
    };
}

#[derive(Debug, sqlx::FromRow)]
struct CharacterRow {
    rowid: i64,
    secret: i64,
    name: String,
    player: bool,
    base_ability_scores: Json<AbilityScores>,
    current_health: i64,
    gold: i64,
    experience: i64,
    equipped_weapon: Option<Json<WeaponType>>,
    equipped_offhand: Option<Json<WeaponType>>,
    equipped_armor: Option<Json<ArmorType>>,
    weapon_inventory: Json<Vec<WeaponType>>,
    armor_inventory: Json<Vec<ArmorType>>,
    level_up_choices: Json<Vec<LevelUpChoice>>,
}

impl From<CharacterRow> for Character {
    fn from(row: CharacterRow) -> Self {
        Character {
            id: row.rowid,
            name: row.name,
            player: row.player,
            gold: Gold(row.gold as u16),
            experience: Experience(row.experience as u32),
            base_ability_scores: row.base_ability_scores.0,
            current_health: HealthPoints(row.current_health as i16),
            equipped_weapon: row.equipped_weapon.map(|weapon| weapon.0),
            equipped_offhand: row.equipped_offhand.map(|weapon| weapon.0),
            equipped_armor: row.equipped_armor.map(|weapon| weapon.0),
            weapon_inventory: row.weapon_inventory.0,
            armor_inventory: row.armor_inventory.0,
            level_up_choices: row.level_up_choices.0,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CharacterInfo {
    pub id: i64,
    pub secret: i64,
}

pub async fn create_player_character(name: &str, ability_scores: AbilityScores) -> CharacterInfo {
    let mut connection = acquire!();

    let base_ability_scores_serialized = serde_json::to_string(&ability_scores).unwrap();
    let health = max_health(&ability_scores.constitution, Level(0));
    let secret = rand::random_range(1..=10000);
    let weapon_inventory: Vec<WeaponType> = vec![];
    let armor_inventory: Vec<ArmorType> = vec![];
    let levels: Vec<LevelUpChoice> = vec![];
    let weapon_inventory_json = serde_json::to_string(&weapon_inventory).unwrap();
    let armor_inventory_json = serde_json::to_string(&armor_inventory).unwrap();
    let levels_json = serde_json::to_string(&levels).unwrap();

    let result = sqlx::query!(
        r#"
            insert into characters (secret, name, player, gold, experience, base_ability_scores, current_health, weapon_inventory, armor_inventory, level_up_choices)
            values ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        secret,
        name,
        true,
        100,
        0,
        base_ability_scores_serialized,
        *health,
        weapon_inventory_json,
        armor_inventory_json,
        levels_json
    )
    .execute(&mut *connection)
    .await
    .unwrap();

    CharacterInfo {
        id: result.last_insert_rowid(),
        secret,
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn create_character(
    name: &str,
    ability_scores: AbilityScores,
    gold: Gold,
    experience: Experience,
    equipped_weapon: Option<WeaponType>,
    equipped_offhand: Option<WeaponType>,
    equipped_armor: Option<ArmorType>,
    weapon_inventory: Vec<WeaponType>,
    armor_inventory: Vec<ArmorType>,
    levels: Vec<LevelUpChoice>,
) -> CharacterInfo {
    let mut connection = acquire!();
    let base_ability_scores_serialized = serde_json::to_string(&ability_scores).unwrap();
    let health = max_health(&ability_scores.constitution, Level(0));
    let secret = rand::random_range(1..=10000);
    let equipped_weapon = equipped_weapon.map(|w| serde_json::to_string(&w).unwrap());
    let equipped_offhand = equipped_offhand.map(|w| serde_json::to_string(&w).unwrap());
    let equipped_armor = equipped_armor.map(|a| serde_json::to_string(&a).unwrap());

    let weapon_inventory_json = serde_json::to_string(&weapon_inventory).unwrap();
    let armor_inventory_json = serde_json::to_string(&armor_inventory).unwrap();
    let levels_json = serde_json::to_string(&levels).unwrap();

    let result = sqlx::query!(
        r#"
            insert into characters (secret, name, player, gold, experience, base_ability_scores, current_health, equipped_weapon, equipped_offhand, equipped_armor, weapon_inventory, armor_inventory, level_up_choices)
            values ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        "#,
        secret,
        name,
        false,
        *gold,
        *experience,
        base_ability_scores_serialized,
        *health,
        equipped_weapon,
        equipped_offhand,
        equipped_armor,
        weapon_inventory_json,
        armor_inventory_json,
        levels_json
    )
    .execute(&mut *connection)
    .await
    .unwrap();

    CharacterInfo {
        id: result.last_insert_rowid(),
        secret,
    }
}

pub async fn set_active_character(character: CharacterInfo) {
    let mut connection = acquire!();
    sqlx::query!(
        r#"
        update active_character
        set id=$1, secret = $2
        "#,
        character.id,
        character.secret
    )
    .execute(&mut *connection)
    .await
    .unwrap();
}

pub async fn get_active_character() -> Result<CharacterInfo, DatabaseError> {
    let mut connection = acquire!();

    let result = sqlx::query_as!(
        CharacterInfo,
        r#"
        select id, secret from active_character limit 1
        "#
    )
    .fetch_optional(&mut *connection)
    .await;

    match result.unwrap() {
        Some(result) => Ok(result),
        None => Err(DatabaseError::NoActiveCharacter),
    }
}

pub async fn validate_player(character_info: &CharacterInfo) -> Result<bool, DatabaseError> {
    let character = get_character_row(character_info.id).await?;
    if character.secret != character_info.secret {
        return Err(DatabaseError::WrongSecret);
    }
    Ok(Character::from(character).is_alive())
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Mismatch in character secret. Is this your character?")]
    WrongSecret,

    #[error("No active character. Create one with 'cli-dungeon create'")]
    NoActiveCharacter,

    #[error("Character not found. Did you create one?")]
    CharacterNotFound,
}

pub async fn get_character(id: i64) -> Result<Character, DatabaseError> {
    let row = get_character_row(id).await?;

    Ok(row.into())
}

async fn get_character_row(id: i64) -> Result<CharacterRow, DatabaseError> {
    let mut connection = acquire!();

    let result = sqlx::query_as!(
        CharacterRow,
        r#"
        select
        rowid,
        name,
        secret,
        player,
        gold,
        experience,
        base_ability_scores as "base_ability_scores: Json<AbilityScores>",
        current_health,
        equipped_weapon as "equipped_weapon: Json<WeaponType>",
        equipped_offhand as "equipped_offhand: Json<WeaponType>",
        equipped_armor as "equipped_armor: Json<ArmorType>",
        weapon_inventory as "weapon_inventory: Json<Vec<WeaponType>>",
        armor_inventory as "armor_inventory: Json<Vec<ArmorType>>",
        level_up_choices as "level_up_choices: Json<Vec<LevelUpChoice>>"
        from characters where rowid = $1"#,
        id
    )
    .fetch_one(&mut *connection)
    .await;

    match result {
        Ok(row) => Ok(row),
        Err(_) => Err(DatabaseError::CharacterNotFound),
    }
}

pub async fn set_character_health(id: i64, health: HealthPoints) {
    let mut connection = acquire!();
    let result = sqlx::query!(
        "update characters set current_health = $2 where rowid = $1",
        id,
        *health
    )
    .execute(&mut *connection)
    .await;

    result.unwrap();
}

pub async fn set_character_gold(id: i64, gold: Gold) {
    let mut connection = acquire!();
    let result = sqlx::query!(
        "update characters set gold = $2 where rowid = $1",
        id,
        *gold
    )
    .execute(&mut *connection)
    .await;

    result.unwrap();
}

pub async fn set_character_experience(id: i64, experience: Experience) {
    let mut connection = acquire!();

    let result = sqlx::query!(
        "update characters set experience = $2 where rowid = $1",
        id,
        *experience
    )
    .execute(&mut *connection)
    .await;

    result.unwrap();
}

pub async fn add_level_up_choice(
    character_id: i64,
    choice: LevelUpChoice,
) -> Result<(), DatabaseError> {
    let character = get_character(character_id).await?;
    let mut choices = character.level_up_choices;
    choices.push(choice);

    let mut connection = acquire!();

    let inventory_json = to_string(&choices).unwrap();

    let result = sqlx::query!(
        "update characters set level_up_choices = $2 where rowid = $1",
        character_id,
        inventory_json
    )
    .execute(&mut *connection)
    .await;

    result.unwrap();
    Ok(())
}

pub async fn add_weapon_to_inventory(
    character_id: i64,
    weapon: WeaponType,
) -> Result<(), DatabaseError> {
    let character = get_character(character_id).await?;
    let mut new_inventory = character.weapon_inventory;
    new_inventory.push(weapon);

    let mut connection = acquire!();

    let inventory_json = to_string(&new_inventory).unwrap();

    let result = sqlx::query!(
        "update characters set weapon_inventory = $2 where rowid = $1",
        character_id,
        inventory_json
    )
    .execute(&mut *connection)
    .await;

    result.unwrap();
    Ok(())
}

pub async fn add_armor_to_inventory(
    character_id: i64,
    armor: ArmorType,
) -> Result<(), DatabaseError> {
    let character = get_character(character_id).await?;
    let mut new_inventory = character.armor_inventory;
    new_inventory.push(armor);

    let mut connection = acquire!();

    let inventory_json = to_string(&new_inventory).unwrap();

    let result = sqlx::query!(
        "update characters set armor_inventory = $2 where rowid = $1",
        character_id,
        inventory_json
    )
    .execute(&mut *connection)
    .await;

    result.unwrap();
    Ok(())
}

pub async fn equip_weapon(character_id: i64, weapon: WeaponType) {
    let mut connection = acquire!();
    let weapon_json = to_string(&weapon).unwrap();
    let result = sqlx::query!(
        "update characters set equipped_weapon = $2 where rowid = $1",
        character_id,
        weapon_json
    )
    .execute(&mut *connection)
    .await;

    result.unwrap();
}

pub async fn equip_offhand(character_id: i64, weapon: WeaponType) {
    let mut connection = acquire!();
    let weapon_json = to_string(&weapon).unwrap();
    let result = sqlx::query!(
        "update characters set equipped_offhand = $2 where rowid = $1",
        character_id,
        weapon_json
    )
    .execute(&mut *connection)
    .await;

    result.unwrap();
}

pub async fn equip_armor(character_id: i64, armor: ArmorType) {
    let mut connection = acquire!();
    let armor_json = to_string(&armor).unwrap();
    let result = sqlx::query!(
        "update characters set equipped_armor = $2 where rowid = $1",
        character_id,
        armor_json
    )
    .execute(&mut *connection)
    .await;

    result.unwrap();
}

pub async fn delete_character(id: i64) {
    let mut connection = acquire!();
    let result = sqlx::query!("delete from characters where rowid = $1", id)
        .execute(&mut *connection)
        .await;

    result.unwrap();
}

async fn init() {
    let database_url = if let Ok(url) = std::env::var("DATABASE_URL") {
        std::path::PathBuf::from(url.replace("sqlite://", ""))
    } else {
        let mut url = dirs::data_local_dir().unwrap();
        url.push("cli-dungeon");

        if !url.exists() {
            std::fs::create_dir_all(&url).expect("failed to create database directory");
        }

        url.push("data.db");

        url
    };

    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .filename(database_url)
        .create_if_missing(true);

    let pool = sqlx::SqlitePool::connect_with(options)
        .await
        .expect("failed to open database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migration failed");

    POOL.set(pool).expect("error setting static pool");

    let mut connection = POOL.get().unwrap().acquire().await.unwrap();
    let result = sqlx::query!(
        "insert or ignore into active_character (ROWID, id, secret) values ($1, $2, $3)",
        1,
        0,
        0
    )
    .execute(&mut *connection)
    .await;

    result.unwrap();
}

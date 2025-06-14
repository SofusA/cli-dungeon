use cli_dungeon_rules::{
    Character, Encounter, Status,
    abilities::AbilityScores,
    armor::ArmorType,
    classes::LevelUpChoice,
    conditions::ActiveCondition,
    items::ItemType,
    jewelry::JewelryType,
    max_health,
    monsters::MonsterType,
    types::{Experience, Gold, HealthPoints, Level, QuestPoint},
    weapons::WeaponType,
};
use futures::future::join_all;
use serde_json::to_string;
use sqlx::types::Json;
use thiserror::Error;

pub type Pool = sqlx::Pool<sqlx::Sqlite>;

pub async fn get_pool() -> Pool {
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

    sqlx::SqlitePool::connect_with(options)
        .await
        .expect("failed to open database")
}

pub async fn init(pool: &Pool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("migration failed");
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
    equipped_jewelry: Json<Vec<JewelryType>>,
    weapon_inventory: Json<Vec<WeaponType>>,
    armor_inventory: Json<Vec<ArmorType>>,
    jewelry_inventory: Json<Vec<JewelryType>>,
    item_inventory: Json<Vec<ItemType>>,
    level_up_choices: Json<Vec<LevelUpChoice>>,
    status: Json<DbStatus>,
    encounter_id: Option<i64>,
    party_id: i64,
    quest_points: i64,
    short_rests: i64,
    active_conditions: Json<Vec<ActiveCondition>>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
enum DbStatus {
    Resting,
    Questing,
}

impl From<DbStatus> for Status {
    fn from(value: DbStatus) -> Self {
        match value {
            DbStatus::Resting => Self::Resting,
            DbStatus::Questing => Self::Questing,
        }
    }
}

impl From<CharacterRow> for Character {
    fn from(row: CharacterRow) -> Self {
        let status = match row.encounter_id {
            Some(encounter) => Status::Fighting(encounter),
            None => row.status.0.into(),
        };

        Character {
            id: row.rowid,
            name: row.name,
            player: row.player,
            gold: Gold::new(row.gold as u16),
            experience: Experience::new(row.experience as u32),
            base_ability_scores: row.base_ability_scores.0,
            current_health: HealthPoints::new(row.current_health as i16),
            equipped_weapon: row.equipped_weapon.map(|item| item.0),
            equipped_offhand: row.equipped_offhand.map(|item| item.0),
            equipped_armor: row.equipped_armor.map(|item| item.0),
            equipped_jewelry: row.equipped_jewelry.0,
            weapon_inventory: row.weapon_inventory.0,
            armor_inventory: row.armor_inventory.0,
            jewelry_inventory: row.jewelry_inventory.0,
            item_inventory: row.item_inventory.0,
            level_up_choices: row.level_up_choices.0,
            status,
            party: row.party_id,
            quest_points: QuestPoint::new(row.quest_points as u16),
            short_rests_available: row.short_rests as u16,
            active_conditions: row.active_conditions.0,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CharacterInfo {
    pub id: i64,
    secret: i64,
}

pub async fn create_player_character(
    pool: &Pool,
    name: &str,
    ability_scores: AbilityScores,
) -> CharacterInfo {
    let party_id = create_party(pool).await;

    let base_ability_scores_serialized = to_string(&ability_scores).unwrap();
    let health = max_health(&ability_scores.constitution, Level::new(0));
    let secret = rand::random_range(1..=10000);
    let weapon_inventory: Vec<WeaponType> = vec![];
    let armor_inventory: Vec<ArmorType> = vec![];
    let jewelry_inventory: Vec<JewelryType> = vec![];
    let item_inventory: Vec<ItemType> = vec![];
    let levels: Vec<LevelUpChoice> = vec![];
    let equipped_jewelry: Vec<JewelryType> = vec![];
    let active_conditions: Vec<ActiveCondition> = vec![];
    let weapon_inventory_json = to_string(&weapon_inventory).unwrap();
    let armor_inventory_json = to_string(&armor_inventory).unwrap();
    let jewelry_inventory_json = to_string(&jewelry_inventory).unwrap();
    let item_inventory_json = to_string(&item_inventory).unwrap();
    let levels_json = to_string(&levels).unwrap();
    let equipped_jewelry_json = to_string(&equipped_jewelry).unwrap();
    let status_json = to_string(&DbStatus::Resting).unwrap();
    let active_condition_json = to_string(&active_conditions).unwrap();

    let result = sqlx::query!(
        r#"
            insert into characters (secret,
            name,
            player,
            gold,
            base_ability_scores,
            current_health,
            weapon_inventory,
            armor_inventory,
            jewelry_inventory,
            item_inventory,
            level_up_choices,
            equipped_jewelry,
            status,
            party_id,
            short_rests,
            active_conditions)
            values ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        secret,
        name,
        true,
        100,
        base_ability_scores_serialized,
        *health,
        weapon_inventory_json,
        armor_inventory_json,
        jewelry_inventory_json,
        item_inventory_json,
        levels_json,
        equipped_jewelry_json,
        status_json,
        party_id,
        2,
        active_condition_json
    )
    .execute(pool)
    .await
    .unwrap();

    CharacterInfo {
        id: result.last_insert_rowid(),
        secret,
    }
}

pub async fn create_encounter(pool: &Pool, rotation: Vec<i64>) -> i64 {
    let rotation_json = to_string(&rotation).unwrap();
    let dead_characters: Vec<i64> = vec![];
    let dead_characters_json = to_string(&dead_characters).unwrap();

    let result = sqlx::query!(
        r#"
                insert into encounters (rotation, dead_characters) values ($1, $2);
            "#,
        rotation_json,
        dead_characters_json
    )
    .execute(pool)
    .await
    .unwrap();

    result.last_insert_rowid()
}

pub async fn update_encounter(pool: &Pool, encounter_id: i64, rotation: Vec<i64>, dead: Vec<i64>) {
    let rotation_json = to_string(&rotation).unwrap();
    let dead_json = to_string(&dead).unwrap();

    let result = sqlx::query!(
        r#"
            update encounters
            set rotation = $2, dead_characters = $3
            where rowid = $1"#,
        encounter_id,
        rotation_json,
        dead_json
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn create_party(pool: &Pool) -> i64 {
    let result = sqlx::query!(
        r#"
                update party_counter set value = value + 1;
                select value from party_counter;
            "#,
    )
    .fetch_one(pool)
    .await
    .unwrap();

    result.value.unwrap()
}

pub async fn set_encounter_id(pool: &Pool, character_id: &i64, encounter_id: Option<i64>) {
    let result = sqlx::query!(
        "update characters set encounter_id = $2 where rowid = $1",
        character_id,
        encounter_id
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn create_monster(pool: &Pool, monster: MonsterType, party_id: i64) -> CharacterInfo {
    let monster = monster.to_monster();

    let base_ability_scores_serialized = to_string(&monster.base_ability_scores).unwrap();

    let health = max_health(
        &monster.base_ability_scores.constitution,
        Level::new(monster.levels.len() as u16),
    );
    let secret = rand::random_range(1..=10000);
    let equipped_weapon = monster.equipped_weapon.map(|w| to_string(&w).unwrap());
    let equipped_offhand = monster.equipped_offhand.map(|w| to_string(&w).unwrap());
    let equipped_armor = monster.equipped_armor.map(|a| to_string(&a).unwrap());

    let equipped_jewelry_json = to_string(&monster.equipped_jewelry).unwrap();
    let weapon_inventory_json = to_string(&monster.weapon_inventory).unwrap();
    let armor_inventory_json = to_string(&monster.armor_inventory).unwrap();
    let jewelry_inventory_json = to_string(&monster.jewelry_inventory).unwrap();
    let item_inventory_json = to_string(&monster.item_inventory).unwrap();
    let levels_json = to_string(&monster.levels).unwrap();
    let status_json = to_string(&DbStatus::Questing).unwrap();
    let active_conditions_json = to_string(&monster.active_conditions).unwrap();

    let result = sqlx::query!(
        r#"
            insert into characters (
            secret,
            name,
            player,
            gold,
            experience,
            base_ability_scores,
            current_health,
            equipped_weapon,
            equipped_offhand,
            equipped_armor,
            equipped_jewelry,
            weapon_inventory,
            armor_inventory,
            jewelry_inventory,
            item_inventory,
            level_up_choices,
            status,
            party_id,
            active_conditions)
            values ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
        "#,
        secret,
        monster.name,
        false,
        *monster.gold,
        *monster.experience,
        base_ability_scores_serialized,
        *health,
        equipped_weapon,
        equipped_offhand,
        equipped_armor,
        equipped_jewelry_json,
        weapon_inventory_json,
        armor_inventory_json,
        jewelry_inventory_json,
        item_inventory_json,
        levels_json,
        status_json,
        party_id,
        active_conditions_json
    )
    .execute(pool)
    .await
    .unwrap();

    CharacterInfo {
        id: result.last_insert_rowid(),
        secret,
    }
}

pub async fn set_active_character(pool: &Pool, character: &CharacterInfo) {
    sqlx::query!(
        r#"
            update active_character
            set id=$1, secret = $2
            "#,
        character.id,
        character.secret
    )
    .execute(pool)
    .await
    .unwrap();
}

pub async fn get_active_character(pool: &Pool) -> Result<CharacterInfo, DatabaseError> {
    let result = sqlx::query_as!(
        CharacterInfo,
        r#"
            select id, secret from active_character limit 1
            "#
    )
    .fetch_optional(pool)
    .await;

    match result.unwrap() {
        Some(result) => Ok(result),
        None => Err(DatabaseError::NoActiveCharacter),
    }
}

pub async fn validate_player(
    pool: &Pool,
    character_info: &CharacterInfo,
) -> Result<bool, DatabaseError> {
    let character = get_character_row(pool, &character_info.id).await?;
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

    #[error("Encounter not found")]
    EncounterNotFound,
}

pub async fn get_character(pool: &Pool, id: &i64) -> Result<Character, DatabaseError> {
    let row = get_character_row(pool, id).await?;

    Ok(row.into())
}

async fn get_character_row(pool: &Pool, id: &i64) -> Result<CharacterRow, DatabaseError> {
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
            equipped_jewelry as "equipped_jewelry: Json<Vec<JewelryType>>",
            weapon_inventory as "weapon_inventory: Json<Vec<WeaponType>>",
            armor_inventory as "armor_inventory: Json<Vec<ArmorType>>",
            jewelry_inventory as "jewelry_inventory: Json<Vec<JewelryType>>",
            item_inventory as "item_inventory: Json<Vec<ItemType>>",
            level_up_choices as "level_up_choices: Json<Vec<LevelUpChoice>>",
            active_conditions as "active_conditions: Json<Vec<ActiveCondition>>",
            short_rests,
            quest_points,
            encounter_id,
            party_id,
            status as "status: Json<DbStatus>"
            from characters where rowid = $1"#,
        id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(row) => Ok(row),
        Err(_) => Err(DatabaseError::CharacterNotFound),
    }
}

#[derive(Debug, sqlx::FromRow)]
struct EncounterRow {
    rotation: Json<Vec<i64>>,
    dead_characters: Json<Vec<i64>>,
    rowid: i64,
}

pub async fn get_encounter(pool: &Pool, id: &i64) -> Result<Encounter, DatabaseError> {
    let result = sqlx::query_as!(
        EncounterRow,
        r#"
            select
            rotation as "rotation: Json<Vec<i64>>",
            dead_characters as "dead_characters: Json<Vec<i64>>",
            rowid
            from encounters where rowid = $1"#,
        id
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(row) => {
            let rotation = join_all(
                row.rotation
                    .0
                    .iter()
                    .map(async |id| get_character(pool, id).await.unwrap()),
            )
            .await;

            let dead_characters = join_all(
                row.dead_characters
                    .0
                    .iter()
                    .map(async |id| get_character(pool, id).await.unwrap()),
            )
            .await;

            Ok(Encounter {
                rotation,
                dead_characters,
                id: row.rowid,
            })
        }
        Err(_) => Err(DatabaseError::EncounterNotFound),
    }
}

pub async fn set_character_status(pool: &Pool, id: &i64, status: Status) {
    if !matches!(status, Status::Fighting(_)) {
        set_encounter_id(pool, id, None).await;
    }

    let db_status = match status {
        Status::Resting => DbStatus::Resting,
        Status::Questing => DbStatus::Questing,
        Status::Fighting(_) => DbStatus::Questing,
    };
    let db_status_json = to_string(&db_status).unwrap();

    let result = sqlx::query!(
        "update characters set status = $2 where rowid = $1",
        id,
        db_status_json
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn set_character_health(pool: &Pool, id: &i64, health: HealthPoints) {
    let result = sqlx::query!(
        "update characters set current_health = $2 where rowid = $1",
        id,
        *health
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn set_character_short_rests(pool: &Pool, id: &i64, short_rests: u16) {
    let result = sqlx::query!(
        "update characters set short_rests = $2 where rowid = $1",
        id,
        short_rests
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn set_character_quest_points(pool: &Pool, id: &i64, quest_points: QuestPoint) {
    let result = sqlx::query!(
        "update characters set quest_points = $2 where rowid = $1",
        id,
        *quest_points
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn set_character_gold(pool: &Pool, id: &i64, gold: Gold) {
    let result = sqlx::query!(
        "update characters set gold = $2 where rowid = $1",
        id,
        *gold
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn set_character_conditions(
    pool: &Pool,
    character_id: &i64,
    conditions: Vec<ActiveCondition>,
) {
    let conditions_json = to_string(&conditions).unwrap();

    let result = sqlx::query!(
        "update characters set active_conditions = $2 where rowid = $1",
        character_id,
        conditions_json
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn set_character_experience(pool: &Pool, id: &i64, experience: Experience) {
    let result = sqlx::query!(
        "update characters set experience = $2 where rowid = $1",
        id,
        *experience
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn add_level_up_choice(
    pool: &Pool,
    character_id: &i64,
    choice: LevelUpChoice,
) -> Result<(), DatabaseError> {
    let character = get_character(pool, character_id).await?;
    let mut choices = character.level_up_choices;
    choices.push(choice);

    let inventory_json = to_string(&choices).unwrap();

    let result = sqlx::query!(
        "update characters set level_up_choices = $2 where rowid = $1",
        character_id,
        inventory_json
    )
    .execute(pool)
    .await;

    result.unwrap();
    Ok(())
}

pub async fn add_weapon_to_inventory(
    pool: &Pool,
    character_id: &i64,
    weapon: WeaponType,
) -> Result<(), DatabaseError> {
    let character = get_character(pool, character_id).await?;
    let mut new_inventory = character.weapon_inventory;
    new_inventory.push(weapon);

    let inventory_json = to_string(&new_inventory).unwrap();

    let result = sqlx::query!(
        "update characters set weapon_inventory = $2 where rowid = $1",
        character_id,
        inventory_json
    )
    .execute(pool)
    .await;

    result.unwrap();
    Ok(())
}

pub async fn add_armor_to_inventory(
    pool: &Pool,
    character_id: &i64,
    armor: ArmorType,
) -> Result<(), DatabaseError> {
    let character = get_character(pool, character_id).await?;
    let mut new_inventory = character.armor_inventory;
    new_inventory.push(armor);

    let inventory_json = to_string(&new_inventory).unwrap();

    let result = sqlx::query!(
        "update characters set armor_inventory = $2 where rowid = $1",
        character_id,
        inventory_json
    )
    .execute(pool)
    .await;

    result.unwrap();
    Ok(())
}

pub async fn add_jewelry_to_inventory(
    pool: &Pool,
    character_id: &i64,
    jewelry: JewelryType,
) -> Result<(), DatabaseError> {
    let character = get_character(pool, character_id).await?;
    let mut new_inventory = character.jewelry_inventory;
    new_inventory.push(jewelry);

    let inventory_json = to_string(&new_inventory).unwrap();

    let result = sqlx::query!(
        "update characters set jewelry_inventory = $2 where rowid = $1",
        character_id,
        inventory_json
    )
    .execute(pool)
    .await;

    result.unwrap();
    Ok(())
}

pub async fn add_item_to_inventory(
    pool: &Pool,
    character_id: &i64,
    item: ItemType,
) -> Result<(), DatabaseError> {
    let character = get_character(pool, character_id).await?;
    let mut new_inventory = character.item_inventory;
    new_inventory.push(item);

    let inventory_json = to_string(&new_inventory).unwrap();

    let result = sqlx::query!(
        "update characters set item_inventory = $2 where rowid = $1",
        character_id,
        inventory_json
    )
    .execute(pool)
    .await;

    result.unwrap();
    Ok(())
}

pub async fn equip_weapon(pool: &Pool, character_id: i64, weapon: WeaponType) {
    let weapon_json = to_string(&weapon).unwrap();
    let result = sqlx::query!(
        "update characters set equipped_weapon = $2 where rowid = $1",
        character_id,
        weapon_json
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn update_equipped_jewelry(
    pool: &Pool,
    character_id: i64,
    jewelry: Vec<JewelryType>,
) -> Result<(), DatabaseError> {
    let jewelry_json = to_string(&jewelry).unwrap();
    let result = sqlx::query!(
        "update characters set equipped_jewelry = $2 where rowid = $1",
        character_id,
        jewelry_json
    )
    .execute(pool)
    .await;

    result.unwrap();

    Ok(())
}

pub async fn equip_offhand(pool: &Pool, character_id: i64, weapon: WeaponType) {
    let weapon_json = to_string(&weapon).unwrap();
    let result = sqlx::query!(
        "update characters set equipped_offhand = $2 where rowid = $1",
        character_id,
        weapon_json
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn equip_armor(pool: &Pool, character_id: i64, armor: ArmorType) {
    let armor_json = to_string(&armor).unwrap();
    let result = sqlx::query!(
        "update characters set equipped_armor = $2 where rowid = $1",
        character_id,
        armor_json
    )
    .execute(pool)
    .await;

    result.unwrap();
}

pub async fn delete_character(pool: &Pool, id: i64) {
    let result = sqlx::query!("delete from characters where rowid = $1", id)
        .execute(pool)
        .await;

    result.unwrap();
}

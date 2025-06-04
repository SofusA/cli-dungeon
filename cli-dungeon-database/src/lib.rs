use std::sync::OnceLock;

use cli_dungeon_rules::{Character, Dice};
use sqlx::types::Json;

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
pub struct CharacterRow {
    rowid: i64,
    name: String,
    secret: i64,
    attack_dice: Json<Dice>,
    hit_bonus: i64,
    max_health: i64,
    current_health: i64,
    armor_points: i64,
}

impl From<CharacterRow> for Character {
    fn from(row: CharacterRow) -> Self {
        Character {
            id: row.rowid,
            name: row.name,
            attack_dice: row.attack_dice.0,
            hit_bonus: row.hit_bonus as i16,
            max_health: row.max_health as i16,
            current_health: row.current_health as i16,
            armor_points: row.armor_points as i16,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct CharacterInfo {
    pub id: i64,
    pub secret: i64,
}

pub async fn create_character(
    name: &str,
    health: i16,
    armor_points: i16,
    attack_dice: Dice,
    hit_bonus: i16,
) -> CharacterInfo {
    let mut connection = acquire!();

    let dice = serde_json::to_string(&attack_dice).unwrap();
    let secret = rand::random_range(1..=10000);

    let result = sqlx::query!(
        r#"
            insert into characters (name, secret, attack_dice, hit_bonus, max_health, current_health, armor_points)
            values ( $1, $2, $3, $4, $5, $5, $6)
        "#,
        name,
        secret,
        dice,
        hit_bonus,
        health,
        armor_points
    )
    .execute(&mut *connection)
    .await.unwrap();

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

pub async fn get_active_character() -> Option<CharacterInfo> {
    let mut connection = acquire!();

    let result = sqlx::query_as!(
        CharacterInfo,
        r#"
        select id, secret from active_character limit 1
        "#
    )
    .fetch_optional(&mut *connection)
    .await;

    result.unwrap()
}

pub async fn validate_player(character_info: &CharacterInfo) -> bool {
    let character = get_character_row(character_info.id).await;

    character.secret == character_info.secret
}

pub async fn get_character(id: i64) -> Character {
    get_character_row(id).await.into()
}

async fn get_character_row(id: i64) -> CharacterRow {
    let mut connection = acquire!();

    let result = sqlx::query_as!(CharacterRow, "select rowid, name, secret, attack_dice as \"attack_dice: Json<Dice>\", hit_bonus, max_health, current_health, armor_points from characters where rowid = $1", id)
        .fetch_one(&mut *connection)
        .await;

    result.unwrap()
}

pub async fn set_character_health(id: i64, health: i16) {
    let mut connection = acquire!();
    let result = sqlx::query!(
        "update characters set current_health = $2 where rowid = $1",
        id,
        health
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

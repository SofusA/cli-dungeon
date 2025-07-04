use cli_dungeon_database::{CharacterInfo, Pool};
use cli_dungeon_rules::{
    Status,
    abilities::{AbilityScores, AbilityType},
    armor::ArmorType,
    character::Character,
    classes::{ClassType, LevelUpChoice},
    jewelry::JewelryType,
    types::{HealthPoints, QuestPoint},
    weapons::WeaponType,
};
use sanitizer::StringSanitizer;

use crate::GameError;

pub async fn create_character(pool: &Pool, name: String, secret: Option<i64>) -> CharacterInfo {
    let mut instance = StringSanitizer::from(name.as_str());
    instance.alphanumeric();
    let sanitized_name = instance.get();

    let ability_scores = AbilityScores::new(8, 8, 8);

    cli_dungeon_database::create_player_character(pool, &sanitized_name, secret, ability_scores)
        .await
}

pub async fn rest(pool: &Pool, character_info: &CharacterInfo) -> Result<(), GameError> {
    let character = get_character(pool, character_info).await?;

    cli_dungeon_database::set_character_status(pool, &character.id, Status::Resting).await;
    cli_dungeon_database::set_character_quest_points(pool, &character.id, QuestPoint::new(0)).await;
    cli_dungeon_database::set_character_health(pool, &character.id, character.max_health()).await;
    cli_dungeon_database::set_character_conditions(pool, &character.id, vec![]).await;
    cli_dungeon_database::set_character_short_rests(pool, &character.id, 2).await;

    Ok(())
}

pub async fn short_rest(pool: &Pool, character_info: &CharacterInfo) -> Result<(), GameError> {
    let character = get_character(pool, character_info).await?;
    let short_rests_available = character.short_rests_available;

    if short_rests_available < 1 {
        return Err(GameError::InsufficientShortRests);
    }

    let mut new_health = HealthPoints::new(*character.max_health() / 2) + character.current_health;
    if new_health > character.max_health() {
        new_health = character.max_health()
    }
    let new_short_rests = character.short_rests_available - 1;

    cli_dungeon_database::set_character_health(pool, &character.id, new_health).await;
    cli_dungeon_database::set_character_short_rests(pool, &character.id, new_short_rests).await;
    Ok(())
}

pub async fn quest(pool: &Pool, character_info: &CharacterInfo) -> Result<(), GameError> {
    let character = get_character(pool, character_info).await?;

    cli_dungeon_database::set_character_status(pool, &character.id, Status::Questing).await;
    Ok(())
}

pub async fn levelup(
    pool: &Pool,
    character_info: &CharacterInfo,
    class: String,
    ability_increment: String,
) -> Result<(), GameError> {
    let character = get_character(pool, character_info).await?;

    if character.experience_level() < character.level() {
        return Err(GameError::InsufficientExperience);
    }

    let Some(parsed_class) = ClassType::from_class_str(&class) else {
        return Err(GameError::UnknownClass);
    };

    let Some(parsed_ability) = AbilityType::from_ability_str(&ability_increment) else {
        return Err(GameError::UnknownClass);
    };

    let choice = LevelUpChoice {
        ability_increment: parsed_ability,
        class: parsed_class,
    };

    cli_dungeon_database::add_level_up_choice(pool, &character.id, choice).await?;
    rest(pool, character_info).await?;

    Ok(())
}

pub async fn get_character(
    pool: &Pool,
    character_info: &CharacterInfo,
) -> Result<Character, GameError> {
    validate_player(pool, character_info).await?;

    Ok(cli_dungeon_database::get_character(pool, &character_info.id).await?)
}

pub async fn equip_main_hand(
    pool: &Pool,
    character_info: &CharacterInfo,
    weapon: String,
) -> Result<(), GameError> {
    let Some(parsed_weapon) = WeaponType::from_weapon_str(&weapon) else {
        return Err(GameError::UnknownWeapon);
    };

    let character = get_character(pool, character_info).await?;
    let weapon_stats = parsed_weapon.to_weapon();

    let in_offhand = character
        .equipped_offhand
        .is_some_and(|weapon| weapon == parsed_weapon);
    let expected_count = if in_offhand { 2 } else { 1 };

    if **character.ability_scores().strength < **weapon_stats.strength_requirement {
        return Err(GameError::InsufficientStrength);
    }

    if character
        .weapon_inventory
        .into_iter()
        .filter(|x| *x == parsed_weapon)
        .count()
        < expected_count
    {
        return Err(GameError::WeaponNotInInventory);
    };

    if weapon_stats.two_handed {
        cli_dungeon_database::unequip_offhand(pool, &character_info.id).await;
    }

    cli_dungeon_database::equip_weapon(pool, &character_info.id, parsed_weapon).await;

    Ok(())
}

pub async fn equip_jewelry(
    pool: &Pool,
    character_info: &CharacterInfo,
    jewelry: String,
) -> Result<(), GameError> {
    let Some(parsed_jewelry) = JewelryType::from_jewelry_str(&jewelry) else {
        return Err(GameError::UnknownJewelry);
    };

    let character = get_character(pool, character_info).await?;
    let mut equipped = character.equipped_jewelry.clone();

    if equipped.len() > 3 {
        return Err(GameError::TooManyJewelriesEquipped);
    };

    let in_inventory = character
        .jewelry_inventory
        .into_iter()
        .filter(|jewelry| *jewelry == parsed_jewelry)
        .count();

    let already_equipped = character
        .equipped_jewelry
        .into_iter()
        .filter(|jewelry| *jewelry == parsed_jewelry)
        .count();

    if in_inventory < already_equipped + 1 {
        return Err(GameError::JewelryNotInInventory);
    };

    equipped.push(parsed_jewelry);

    cli_dungeon_database::update_equipped_jewelry(pool, &character_info.id, equipped).await?;

    Ok(())
}

pub async fn unequip_jewelry(
    pool: &Pool,
    character_info: &CharacterInfo,
    jewelry: String,
) -> Result<(), GameError> {
    let Some(parsed_jewelry) = JewelryType::from_jewelry_str(&jewelry) else {
        return Err(GameError::UnknownJewelry);
    };

    let character = get_character(pool, character_info).await?;
    let mut equipped = character.equipped_jewelry.clone();

    if let Some(index) = equipped.iter().position(|x| *x == parsed_jewelry) {
        equipped.remove(index);
    }

    cli_dungeon_database::update_equipped_jewelry(pool, &character_info.id, equipped).await?;

    Ok(())
}

pub async fn equip_offhand(
    pool: &Pool,
    character_info: &CharacterInfo,
    weapon: String,
) -> Result<(), GameError> {
    let Some(parsed_weapon) = WeaponType::from_weapon_str(&weapon) else {
        return Err(GameError::UnknownWeapon);
    };

    let weapon_stats = parsed_weapon.to_weapon();
    if !weapon_stats.allow_offhand {
        return Err(GameError::TwoHandedWeaponEquipped);
    }

    let character = get_character(pool, character_info).await?;

    let in_main_hand = character
        .equipped_weapon
        .is_some_and(|weapon| weapon == parsed_weapon);
    let expected_count = if in_main_hand { 2 } else { 1 };

    if **character.ability_scores().strength < **weapon_stats.strength_requirement {
        return Err(GameError::InsufficientStrength);
    }

    if character
        .weapon_inventory
        .into_iter()
        .filter(|x| *x == parsed_weapon)
        .count()
        < expected_count
    {
        return Err(GameError::WeaponNotInInventory);
    };

    if character
        .equipped_weapon
        .is_some_and(|weapon| weapon.to_weapon().two_handed)
    {
        return Err(GameError::TwoHandedWeaponEquipped);
    }

    cli_dungeon_database::equip_offhand(pool, &character_info.id, parsed_weapon).await;

    Ok(())
}

pub async fn equip_armor(
    pool: &Pool,
    character_info: &CharacterInfo,
    armor: String,
) -> Result<(), GameError> {
    let Some(parsed_armor) = ArmorType::from_armor_str(&armor) else {
        return Err(GameError::UnknownArmor);
    };

    validate_player(pool, character_info).await?;

    let armor_stats = parsed_armor.to_armor();
    let character = cli_dungeon_database::get_character(pool, &character_info.id).await?;

    let in_inventory = character
        .armor_inventory
        .iter()
        .filter(|armor| **armor == parsed_armor)
        .count();

    if in_inventory < 1 {
        return Err(GameError::ArmorNotInInventory);
    }

    if **character.ability_scores().strength < **armor_stats.strength_requirement {
        return Err(GameError::InsufficientStrength);
    }

    cli_dungeon_database::equip_armor(pool, &character_info.id, parsed_armor).await;

    Ok(())
}

pub(crate) async fn validate_player(
    pool: &Pool,
    character_info: &CharacterInfo,
) -> Result<(), GameError> {
    if !cli_dungeon_database::validate_player(pool, character_info).await? {
        return Err(GameError::Dead);
    };

    Ok(())
}
#[cfg(test)]
mod tests {
    use cli_dungeon_rules::{
        monsters::MonsterType,
        types::{Experience, HealthPoints, QuestPoint},
        weapons::WeaponType,
    };

    use crate::{
        character::{self, create_character, equip_main_hand, equip_offhand, levelup, short_rest},
        errors::GameError,
        play,
    };

    #[sqlx::test]
    async fn can_short_rest(pool: sqlx::Pool<sqlx::Sqlite>) {
        cli_dungeon_database::init(&pool).await;

        let character_info = create_character(&pool, "testington".to_string(), None).await;
        cli_dungeon_database::set_active_character(&pool, &character_info).await;

        let character = cli_dungeon_database::get_character(&pool, &character_info.id)
            .await
            .unwrap();

        let set_health = HealthPoints::new(1);
        cli_dungeon_database::set_character_health(&pool, &character.id, set_health).await;

        short_rest(&pool, &character_info).await.unwrap();

        let character = cli_dungeon_database::get_character(&pool, &character_info.id)
            .await
            .unwrap();

        assert_eq!(
            character.current_health,
            set_health + HealthPoints::new(*character.max_health() / 2)
        );

        short_rest(&pool, &character_info).await.unwrap();

        let character = cli_dungeon_database::get_character(&pool, &character_info.id)
            .await
            .unwrap();

        assert_eq!(character.current_health, character.max_health());

        let error_short_rest = short_rest(&pool, &character_info).await;
        assert_eq!(error_short_rest, Err(GameError::InsufficientShortRests));
    }

    #[sqlx::test]
    async fn questing_works(pool: sqlx::Pool<sqlx::Sqlite>) {
        cli_dungeon_database::init(&pool).await;

        let character_info = create_character(&pool, "testington".to_string(), None).await;
        cli_dungeon_database::set_active_character(&pool, &character_info).await;

        let character = cli_dungeon_database::get_character(&pool, &character_info.id)
            .await
            .unwrap();
        assert_eq!(character.quest_points, QuestPoint::new(0));

        character::quest(&pool, &character_info).await.unwrap();

        let enemy_party_id = cli_dungeon_database::create_party(&pool).await;

        let enemy = MonsterType::TestMonster;

        let enemy_id = cli_dungeon_database::create_monster(&pool, enemy, enemy_party_id)
            .await
            .id;

        let rotation = vec![character_info.id, enemy_id];

        let encounter_id = cli_dungeon_database::create_encounter(&pool, rotation.clone()).await;

        for character_id in rotation.iter() {
            cli_dungeon_database::set_encounter_id(&pool, character_id, Some(encounter_id)).await;
        }

        play(&pool, false, &character_info).await.unwrap();

        let character = cli_dungeon_database::get_character(&pool, &character_info.id)
            .await
            .unwrap();

        assert_eq!(character.quest_points, QuestPoint::new(1));

        play(&pool, false, &character_info).await.unwrap();

        let character = cli_dungeon_database::get_character(&pool, &character_info.id)
            .await
            .unwrap();

        assert_eq!(character.quest_points, QuestPoint::new(2));
    }

    #[sqlx::test]
    async fn two_handed_weapon_works(pool: sqlx::Pool<sqlx::Sqlite>) {
        cli_dungeon_database::init(&pool).await;

        let character_info = create_character(&pool, "testington".to_string(), None).await;
        cli_dungeon_database::set_active_character(&pool, &character_info).await;
        cli_dungeon_database::set_character_experience(
            &pool,
            &character_info.id,
            Experience::new(999999),
        )
        .await;

        for _ in 0..7 {
            levelup(
                &pool,
                &character_info,
                "fighter".to_string(),
                "strength".to_string(),
            )
            .await
            .unwrap();
        }

        cli_dungeon_database::add_weapon_to_inventory(
            &pool,
            &character_info.id,
            WeaponType::Dagger,
        )
        .await
        .unwrap();
        cli_dungeon_database::add_weapon_to_inventory(
            &pool,
            &character_info.id,
            WeaponType::GreatSword,
        )
        .await
        .unwrap();

        equip_offhand(&pool, &character_info, "dagger".to_string())
            .await
            .unwrap();
        equip_main_hand(&pool, &character_info, "greatsword".to_string())
            .await
            .unwrap();

        let character = cli_dungeon_database::get_character(&pool, &character_info.id)
            .await
            .unwrap();
        assert_eq!(character.equipped_offhand, None);

        let err = equip_offhand(&pool, &character_info, "dagger".to_string()).await;

        assert_eq!(err, Err(GameError::TwoHandedWeaponEquipped));
    }
}

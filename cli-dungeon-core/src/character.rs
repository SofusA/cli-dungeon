use cli_dungeon_database::CharacterInfo;
use cli_dungeon_rules::{
    Character,
    abilities::{AbilityScores, AbilityType},
    armor::ArmorType,
    classes::{ClassType, LevelUpChoice},
    types::Gold,
    weapons::WeaponType,
};
use sanitizer::StringSanitizer;

use crate::GameError;

pub async fn create_character(
    name: String,
    strength: u16,
    dexterity: u16,
    constitution: u16,
) -> Result<CharacterInfo, GameError> {
    let mut instance = StringSanitizer::from(name.as_str());
    instance.alphanumeric();
    let sanitized_name = instance.get();

    if strength + dexterity + constitution != 10 {
        return Err(GameError::AbilitySumError);
    }
    let ability_scores = AbilityScores::new(8 + strength, 8 + dexterity, 8 + constitution);

    let character_info =
        cli_dungeon_database::create_player_character(&sanitized_name, ability_scores).await;
    Ok(character_info)
}

pub async fn rest(character_info: &CharacterInfo) -> Result<(), GameError> {
    let character = get_character(character_info).await?;

    cli_dungeon_database::set_character_health(character_info.id, character.max_health()).await;
    Ok(())
}

pub async fn levelup(
    character_info: &CharacterInfo,
    class: String,
    ability_increment: String,
) -> Result<(), GameError> {
    let character = get_character(character_info).await?;

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

    cli_dungeon_database::add_level_up_choice(character.id, choice).await?;
    rest(character_info).await?;

    Ok(())
}

pub async fn get_character(character_info: &CharacterInfo) -> Result<Character, GameError> {
    if !cli_dungeon_database::validate_player(character_info).await? {
        return Err(GameError::Dead);
    };

    Ok(cli_dungeon_database::get_character(character_info.id).await?)
}

pub async fn equip_main_hand(
    character_info: &CharacterInfo,
    weapon: String,
) -> Result<(), GameError> {
    let Some(parsed_weapon) = WeaponType::from_weapon_str(&weapon) else {
        return Err(GameError::UnknownWeapon);
    };

    if !cli_dungeon_database::validate_player(character_info).await? {
        return Err(GameError::Dead);
    };

    cli_dungeon_database::equip_weapon(character_info.id, parsed_weapon).await;

    Ok(())
}

pub async fn equip_offhand(
    character_info: &CharacterInfo,
    weapon: String,
) -> Result<(), GameError> {
    let Some(parsed_weapon) = WeaponType::from_weapon_str(&weapon) else {
        return Err(GameError::UnknownWeapon);
    };

    if !cli_dungeon_database::validate_player(character_info).await? {
        return Err(GameError::Dead);
    };

    let weapon_stats = parsed_weapon.to_weapon();
    if !weapon_stats.allow_offhand {
        return Err(GameError::NotOffHandWeapon);
    }

    cli_dungeon_database::equip_offhand(character_info.id, parsed_weapon).await;

    Ok(())
}

pub async fn equip_armor(character_info: &CharacterInfo, armor: String) -> Result<(), GameError> {
    let Some(parsed_armor) = ArmorType::from_armor_str(&armor) else {
        return Err(GameError::UnknownArmor);
    };

    if !cli_dungeon_database::validate_player(character_info).await? {
        return Err(GameError::Dead);
    };

    let armor_stats = parsed_armor.to_armor();
    let character = cli_dungeon_database::get_character(character_info.id).await?;

    if **character.ability_scores().strength < **armor_stats.strength_requirement {
        return Err(GameError::InsufficientStrength);
    }

    cli_dungeon_database::equip_armor(character_info.id, parsed_armor).await;

    Ok(())
}

pub async fn buy(character_info: &CharacterInfo, item: String) -> Result<(), GameError> {
    if let Some(weapon) = WeaponType::from_weapon_str(&item) {
        return buy_weapon(character_info, weapon).await;
    };

    if let Some(armor) = ArmorType::from_armor_str(&item) {
        return buy_armor(character_info, armor).await;
    };

    Err(GameError::UnknownItem)
}

async fn buy_weapon(character_info: &CharacterInfo, weapon: WeaponType) -> Result<(), GameError> {
    if !cli_dungeon_database::validate_player(character_info).await? {
        return Err(GameError::Dead);
    };

    let stats = weapon.to_weapon();
    let character = cli_dungeon_database::get_character(character_info.id).await?;

    let new_gold = character.gold - stats.cost;
    if new_gold < Gold(0) {
        return Err(GameError::InsufficientGold);
    }

    cli_dungeon_database::set_character_gold(character_info.id, new_gold).await;
    cli_dungeon_database::add_weapon_to_inventory(character_info.id, weapon).await?;

    Ok(())
}

async fn buy_armor(character_info: &CharacterInfo, armor: ArmorType) -> Result<(), GameError> {
    if !cli_dungeon_database::validate_player(character_info).await? {
        return Err(GameError::Dead);
    };

    let stats = armor.to_armor();
    let character = cli_dungeon_database::get_character(character_info.id).await?;

    let new_gold = character.gold - stats.cost;
    if new_gold < Gold(0) {
        return Err(GameError::InsufficientGold);
    }

    cli_dungeon_database::set_character_gold(character_info.id, new_gold).await;
    cli_dungeon_database::add_armor_to_inventory(character_info.id, armor).await?;
    Ok(())
}

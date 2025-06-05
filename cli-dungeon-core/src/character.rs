use cli_dungeon_database::CharacterInfo;
use cli_dungeon_rules::{AbilityScores, ArmorType, Character, WeaponType};

use crate::GameError;

pub async fn create_character(
    name: String,
    strength: u16,
    dexterity: u16,
    constitution: u16,
) -> Result<CharacterInfo, GameError> {
    if strength + dexterity + constitution != 10 {
        return Err(GameError::AbilitySumError);
    }
    let ability_scores = AbilityScores::new(8 + strength, 8 + dexterity, 8 + constitution);

    let character_info = cli_dungeon_database::create_player_character(&name, ability_scores).await;
    Ok(character_info)
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

    if character.ability_scores().strength.0.0 < armor_stats.strength_requirement.0.0 {
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

    let new_gold = character.gold as i16 - stats.cost as i16;
    if new_gold < 0 {
        return Err(GameError::InsufficientGold);
    }

    cli_dungeon_database::set_character_gold(character_info.id, new_gold as u16).await;
    cli_dungeon_database::add_weapon_to_inventory(character_info.id, weapon).await?;

    Ok(())
}

async fn buy_armor(character_info: &CharacterInfo, armor: ArmorType) -> Result<(), GameError> {
    if !cli_dungeon_database::validate_player(character_info).await? {
        return Err(GameError::Dead);
    };

    let stats = armor.to_armor();
    let character = cli_dungeon_database::get_character(character_info.id).await?;

    let new_gold = character.gold as i16 - stats.cost as i16;
    if new_gold < 0 {
        return Err(GameError::InsufficientGold);
    }

    cli_dungeon_database::set_character_gold(character_info.id, new_gold as u16).await;
    cli_dungeon_database::add_armor_to_inventory(character_info.id, armor).await?;
    Ok(())
}

use anyhow::{Result, bail};
use cli_dungeon_database::CharacterInfo;
use cli_dungeon_rules::{AbilityScores, ArmorType, Character, WeaponType};

use crate::GameError;

pub async fn create_character(
    name: String,
    strength: u16,
    dexterity: u16,
    constitution: u16,
) -> Result<CharacterInfo> {
    if strength + dexterity + constitution != 10 {
        bail!("Ability scores must sum to 10");
    }
    let ability_scores = AbilityScores::new(8 + strength, 8 + dexterity, 8 + constitution);

    let character_info = cli_dungeon_database::create_player_character(&name, ability_scores).await;
    println!("{}", character_info.id);
    Ok(character_info)
}

pub async fn get_character(character_info: &CharacterInfo) -> Result<Character> {
    if !cli_dungeon_database::validate_player(character_info).await? {
        bail!(GameError::Dead)
    };

    Ok(cli_dungeon_database::get_character(character_info.id).await?)
}

pub async fn equip_main_hand(character_info: &CharacterInfo, weapon: String) -> Result<()> {
    let parsed_weapon = match serde_json::from_str::<WeaponType>(&weapon) {
        Ok(weapon) => weapon,
        Err(_) => bail!(GameError::UnknownWeapon),
    };

    if !cli_dungeon_database::validate_player(character_info).await? {
        bail!(GameError::Dead)
    };

    cli_dungeon_database::equip_weapon(character_info.id, parsed_weapon).await;

    Ok(())
}

pub async fn equip_offhand(character_info: &CharacterInfo, weapon: String) -> Result<()> {
    let parsed_weapon = match serde_json::from_str::<WeaponType>(&weapon) {
        Ok(weapon) => weapon,
        Err(_) => bail!(GameError::UnknownWeapon),
    };

    if !cli_dungeon_database::validate_player(character_info).await? {
        bail!(GameError::Dead)
    };

    let weapon_stats = parsed_weapon.to_weapon();
    if !weapon_stats.allow_offhand {
        bail!(GameError::NotOffHandWeapon);
    }

    cli_dungeon_database::equip_offhand(character_info.id, parsed_weapon).await;

    Ok(())
}

pub async fn equip_armor(character_info: &CharacterInfo, armor: String) -> Result<()> {
    let parsed_armor = match serde_json::from_str::<ArmorType>(&armor) {
        Ok(armor) => armor,
        Err(_) => bail!(GameError::UnknownWeapon),
    };

    if !cli_dungeon_database::validate_player(character_info).await? {
        bail!(GameError::Dead)
    };

    let armor_stats = parsed_armor.to_armor();
    let character = cli_dungeon_database::get_character(character_info.id).await?;

    if character.ability_scores().strength.0.0 < armor_stats.strength_requirement.0.0 {
        bail!(GameError::InsufficientStrength);
    }

    cli_dungeon_database::equip_armor(character_info.id, parsed_armor).await;

    Ok(())
}

pub async fn buy(character_info: &CharacterInfo, item: String) -> Result<()> {
    if let Ok(weapon) = serde_json::from_str::<WeaponType>(&item) {
        buy_weapon(character_info, weapon).await?
    };

    if let Ok(armor) = serde_json::from_str::<ArmorType>(&item) {
        buy_armor(character_info, armor).await?
    };

    bail!(GameError::UnknownItem)
}

async fn buy_weapon(character_info: &CharacterInfo, weapon: WeaponType) -> Result<()> {
    if !cli_dungeon_database::validate_player(character_info).await? {
        bail!(GameError::Dead)
    };

    let stats = weapon.to_weapon();
    let character = cli_dungeon_database::get_character(character_info.id).await?;

    let new_gold = character.gold as i16 - stats.cost as i16;
    if new_gold < 0 {
        bail!(GameError::InsufficientGold);
    }

    cli_dungeon_database::set_character_gold(character_info.id, new_gold as u16).await;

    Ok(())
}
async fn buy_armor(character_info: &CharacterInfo, armor: ArmorType) -> Result<()> {
    if !cli_dungeon_database::validate_player(character_info).await? {
        bail!(GameError::Dead)
    };

    let stats = armor.to_armor();
    let character = cli_dungeon_database::get_character(character_info.id).await?;

    let new_gold = character.gold as i16 - stats.cost as i16;
    if new_gold < 0 {
        bail!(GameError::InsufficientGold);
    }

    cli_dungeon_database::set_character_gold(character_info.id, new_gold as u16).await;
    Ok(())
}

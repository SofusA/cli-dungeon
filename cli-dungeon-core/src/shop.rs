use cli_dungeon_database::{CharacterInfo, Pool};
use cli_dungeon_rules::{
    armor::ArmorType, items::ItemType, jewelry::JewelryType, types::Gold, weapons::WeaponType,
};

use crate::{errors::GameError, validate_player};

pub struct Shop {
    pub weapons: Vec<WeaponType>,
    pub armor: Vec<ArmorType>,
    pub items: Vec<ItemType>,
}
pub fn available_in_shop() -> Shop {
    Shop {
        weapons: vec![
            WeaponType::Dagger,
            WeaponType::Shortsword,
            WeaponType::Longsword,
            WeaponType::Shield,
        ],
        armor: vec![ArmorType::Leather, ArmorType::BreastPlate],
        items: vec![ItemType::PotionOfHealing],
    }
}

pub async fn buy(
    pool: &Pool,
    character_info: &CharacterInfo,
    item: String,
) -> Result<(), GameError> {
    validate_player(pool, character_info).await?;
    let character = cli_dungeon_database::get_character(pool, &character_info.id).await?;

    if let Some(weapon) = WeaponType::from_weapon_str(&item) {
        if !available_in_shop().weapons.contains(&weapon) {
            return Err(GameError::NotInShop);
        }

        let stats = weapon.to_weapon();

        if character.gold < stats.cost {
            return Err(GameError::InsufficientGold);
        }

        let new_gold = character.gold - stats.cost;

        cli_dungeon_database::set_character_gold(pool, &character_info.id, new_gold).await;
        cli_dungeon_database::add_weapon_to_inventory(pool, &character_info.id, weapon).await?;
        return Ok(());
    };

    if let Some(armor) = ArmorType::from_armor_str(&item) {
        if !available_in_shop().armor.contains(&armor) {
            return Err(GameError::NotInShop);
        }

        let stats = armor.to_armor();

        if character.gold < stats.cost {
            return Err(GameError::InsufficientGold);
        }

        let new_gold = character.gold - stats.cost;

        cli_dungeon_database::set_character_gold(pool, &character_info.id, new_gold).await;
        cli_dungeon_database::add_armor_to_inventory(pool, &character_info.id, armor).await?;
        return Ok(());
    };

    if let Some(item) = ItemType::from_item_str(&item) {
        if !available_in_shop().items.contains(&item) {
            return Err(GameError::NotInShop);
        }

        let stats = item.to_item();

        if character.gold < stats.cost {
            return Err(GameError::InsufficientGold);
        }

        let new_gold = character.gold - stats.cost;

        cli_dungeon_database::set_character_gold(pool, &character_info.id, new_gold).await;
        cli_dungeon_database::add_item_to_inventory(pool, &character_info.id, item).await?;
        return Ok(());
    };

    Err(GameError::UnknownItem)
}

pub async fn sell(
    pool: &Pool,
    character_info: &CharacterInfo,
    item: String,
) -> Result<Gold, GameError> {
    if let Some(weapon) = WeaponType::from_weapon_str(&item) {
        return sell_weapon(pool, character_info, weapon).await;
    };

    if let Some(armor) = ArmorType::from_armor_str(&item) {
        return sell_armor(pool, character_info, armor).await;
    };

    if let Some(item) = ItemType::from_item_str(&item) {
        return sell_item(pool, character_info, item).await;
    };

    if let Some(jewelry) = JewelryType::from_jewelry_str(&item) {
        return sell_jewelry(pool, character_info, jewelry).await;
    };

    Err(GameError::UnknownItem)
}

async fn sell_weapon(
    pool: &Pool,
    character_info: &CharacterInfo,
    weapon: WeaponType,
) -> Result<Gold, GameError> {
    validate_player(pool, character_info).await?;

    let stats = weapon.to_weapon();
    let character = cli_dungeon_database::get_character(pool, &character_info.id).await?;

    let item_count = character
        .weapon_inventory
        .iter()
        .filter(|&owned_weapon| owned_weapon == &weapon)
        .count();

    if item_count < 1 {
        return Err(GameError::WeaponNotInInventory);
    }

    let new_gold = character.gold + stats.cost.sell_value();

    let equipped = character
        .equipped_weapon
        .is_some_and(|equipped| equipped == weapon);

    let equipped_offhand = character
        .equipped_offhand
        .is_some_and(|equipped| equipped == weapon);

    let minimum_count = match (equipped, equipped_offhand) {
        (true, true) => 2,
        (true, false) | (false, true) => 1,
        (false, false) => 0,
    };

    if item_count <= minimum_count && equipped {
        cli_dungeon_database::unequip_weapon(pool, &character_info.id).await;
    } else if item_count <= minimum_count && equipped_offhand {
        cli_dungeon_database::unequip_offhand(pool, &character_info.id).await;
    }

    cli_dungeon_database::set_character_gold(pool, &character_info.id, new_gold).await;
    cli_dungeon_database::remove_weapon_from_inventory(pool, &character_info.id, weapon).await?;

    Ok(new_gold)
}

async fn sell_armor(
    pool: &Pool,
    character_info: &CharacterInfo,
    armor: ArmorType,
) -> Result<Gold, GameError> {
    validate_player(pool, character_info).await?;

    let stats = armor.to_armor();
    let character = cli_dungeon_database::get_character(pool, &character_info.id).await?;

    let item_count = character
        .armor_inventory
        .iter()
        .filter(|&owned_armor| owned_armor == &armor)
        .count();

    if item_count < 1 {
        return Err(GameError::ArmorNotInInventory);
    }

    let new_gold = character.gold + stats.cost.sell_value();

    if item_count == 1 {
        cli_dungeon_database::unequip_armor(pool, &character_info.id).await;
    };

    cli_dungeon_database::set_character_gold(pool, &character_info.id, new_gold).await;
    cli_dungeon_database::remove_armor_from_inventory(pool, &character_info.id, armor).await?;

    Ok(new_gold)
}

async fn sell_item(
    pool: &Pool,
    character_info: &CharacterInfo,
    item: ItemType,
) -> Result<Gold, GameError> {
    validate_player(pool, character_info).await?;

    let stats = item.to_item();
    let character = cli_dungeon_database::get_character(pool, &character_info.id).await?;

    let item_count = character
        .item_inventory
        .iter()
        .filter(|&owned| owned == &item)
        .count();

    if item_count < 1 {
        return Err(GameError::ItemNotInInventory);
    }

    let new_gold = character.gold + stats.cost.sell_value();

    cli_dungeon_database::set_character_gold(pool, &character_info.id, new_gold).await;
    cli_dungeon_database::remove_item_from_inventory(pool, &character_info.id, item).await?;

    Ok(new_gold)
}

async fn sell_jewelry(
    pool: &Pool,
    character_info: &CharacterInfo,
    jewelry: JewelryType,
) -> Result<Gold, GameError> {
    validate_player(pool, character_info).await?;

    let stats = jewelry.to_jewelry();
    let character = cli_dungeon_database::get_character(pool, &character_info.id).await?;

    let item_count = character
        .jewelry_inventory
        .iter()
        .filter(|&owned| owned == &jewelry)
        .count();

    if item_count < 1 {
        return Err(GameError::JewelryNotInInventory);
    }

    let new_gold = character.gold + stats.cost.sell_value();

    if item_count == 1 {
        let mut new_equipped = character.jewelry_inventory;
        if let Some(pos) = new_equipped.iter().position(|w| w == &jewelry) {
            new_equipped.remove(pos);
        }

        cli_dungeon_database::update_equipped_jewelry(pool, &character_info.id, new_equipped)
            .await?;
    };

    cli_dungeon_database::set_character_gold(pool, &character_info.id, new_gold).await;
    cli_dungeon_database::remove_jewelry_from_inventory(pool, &character_info.id, jewelry).await?;

    Ok(new_gold)
}

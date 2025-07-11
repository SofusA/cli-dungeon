use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cli_dungeon_core::play;
use cli_dungeon_database::{CharacterInfo, Pool};
use cli_dungeon_rules::{
    character::Character,
    classes::ClassType,
    types::{Experience, HealthPoints, QuestPoint},
};
use color_print::{cformat, cprintln};
use config::Config;
use encounter::{display_turn_outcome, handle_encounter};

mod encounter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create new Character
    CreateCharacter,

    /// Character options
    Character {
        #[clap(subcommand)]
        command: CharacterCommands,
    },

    /// Shop available items
    Shop {
        #[clap(subcommand)]
        command: ShopCommands,
    },

    // Play the game
    Play {
        #[arg(short, long, default_value_t = false)]
        /// Force a battle
        force: bool,
    },
}

#[derive(Subcommand)]
enum CharacterCommands {
    Status,
    Equip {
        #[arg(short, long)]
        main_hand: Option<String>,

        #[arg(short, long)]
        offhand: Option<String>,

        #[arg(short, long)]
        armor: Option<String>,

        #[arg(short, long)]
        jewelry: Option<String>,
    },
    Unequip {
        #[arg(short, long)]
        jewelry: Option<String>,
    },
    Rest {
        #[clap(subcommand)]
        command: RestCommands,
    },
    Quest,
    LevelUp {
        #[clap(subcommand)]
        command: LevelUpCommands,
    },
    Actions,
}

#[derive(Subcommand)]
enum LevelUpCommands {
    List,
    Level {
        /// Class to level up
        #[arg(short, long)]
        class: String,

        /// Ability score to increment
        #[arg(short, long)]
        ability_increment: String,
    },
}

#[derive(Subcommand)]
enum ShopCommands {
    List,
    Buy {
        #[arg(short, long)]
        item: String,
    },
    Sell {
        #[arg(short, long)]
        item: String,
    },
}

#[derive(Subcommand)]
enum RestCommands {
    Long,
    Short,
}

pub fn config_path() -> PathBuf {
    let mut config = dirs::config_dir().unwrap();
    config.push("cli-dungeon");

    std::fs::create_dir_all(&config).unwrap();

    config
}

async fn create_character(pool: &Pool) -> CharacterInfo {
    let settings = Config::builder()
        .add_source(config::File::new(
            &config_path().into_os_string().into_string().unwrap(),
            config::FileFormat::Toml,
        ))
        .build();

    let (name, secret) = settings
        .map(|settings| {
            (
                settings.get_string("name").ok(),
                settings.get_int("secret").ok(),
            )
        })
        .unwrap_or((None, None));

    let name = name.unwrap_or(whoami::username());

    encounter::ensure_default_script();

    let character_info = cli_dungeon_core::character::create_character(pool, name, secret).await;
    cli_dungeon_database::set_active_character(pool, &character_info).await;

    character_info
}

async fn get_character(pool: &Pool) -> CharacterInfo {
    if let Ok(character_info) = cli_dungeon_database::get_active_character(pool).await {
        return character_info;
    }
    create_character(pool).await
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let pool = cli_dungeon_database::get_pool().await;
    cli_dungeon_database::init(&pool).await;

    match args.command {
        Commands::CreateCharacter => {
            create_character(&pool).await;
        }
        Commands::Character { command } => match command {
            CharacterCommands::Rest { command } => {
                let character_info = get_character(&pool).await;

                match command {
                    RestCommands::Long => {
                        cli_dungeon_core::character::rest(&pool, &character_info).await?;
                    }
                    RestCommands::Short => {
                        cli_dungeon_core::character::short_rest(&pool, &character_info).await?;
                    }
                }
                let character =
                    cli_dungeon_core::character::get_character(&pool, &character_info).await?;
                cprintln!(
                    "<white>Health:</> {}",
                    health_bar(character.current_health, character.max_health())
                );
            }
            CharacterCommands::Quest => {
                let character_info = get_character(&pool).await;
                cli_dungeon_core::character::quest(&pool, &character_info).await?;
            }
            CharacterCommands::LevelUp { command } => match command {
                LevelUpCommands::List => {
                    cprintln!(
                        "<cyan>{}: General fighting class</>",
                        ClassType::Fighter.to_name()
                    );
                }
                LevelUpCommands::Level {
                    class,
                    ability_increment,
                } => {
                    let character_info = get_character(&pool).await;
                    cli_dungeon_core::character::levelup(
                        &pool,
                        &character_info,
                        class,
                        ability_increment,
                    )
                    .await?;
                }
            },
            CharacterCommands::Status => {
                let character_info = get_character(&pool).await;
                let character =
                    cli_dungeon_core::character::get_character(&pool, &character_info).await?;

                cprintln!("<red>Level: {}</>", character.level());
                cprintln!("<blue>Name: {}</>", character.name);

                print_experience_bar(&character);
                cprintln!("<white>Quest:</> {}", quest_bar(character.quest_points));

                cprintln!("<yellow>Gold: {}</>", character.gold);
                cprintln!(
                    "<white>Health:</> {}",
                    health_bar(character.current_health, character.max_health())
                );
                cprintln!("Armor points: {}", character.armor_points());
                cprintln!("Strength: {}", **character.ability_scores().strength);
                cprintln!("Dexterity: {}", **character.ability_scores().dexterity);
                cprintln!(
                    "Constitution: {}",
                    **character.ability_scores().constitution
                );
                cprintln!(
                    "Weapon: {}",
                    character
                        .equipped_weapon
                        .map(|weapon| weapon.to_weapon().name)
                        .unwrap_or("Unequipped".to_string())
                );
                cprintln!(
                    "Offhand: {}",
                    character
                        .equipped_offhand
                        .map(|weapon| weapon.to_weapon().name)
                        .unwrap_or("Unequipped".to_string())
                );
                cprintln!(
                    "Armor: {}",
                    character
                        .equipped_armor
                        .map(|armor| armor.to_armor().name)
                        .unwrap_or("Unequipped".to_string())
                );

                let combined_inventory: Vec<String> = character
                    .weapon_inventory
                    .iter()
                    .map(|weapon| weapon.to_weapon().name)
                    .chain(
                        character
                            .armor_inventory
                            .iter()
                            .map(|armor| armor.to_armor().name),
                    )
                    .chain(
                        character
                            .jewelry_inventory
                            .iter()
                            .map(|jewelry| jewelry.to_jewelry().name),
                    )
                    .chain(
                        character
                            .item_inventory
                            .iter()
                            .map(|item| item.to_item().name),
                    )
                    .collect();

                cprintln!("Inventory: {}", combined_inventory.join(" "));
            }
            CharacterCommands::Equip {
                main_hand,
                offhand,
                armor,
                jewelry,
            } => {
                let character_info = get_character(&pool).await;
                if let Some(main_hand) = main_hand {
                    cli_dungeon_core::character::equip_main_hand(&pool, &character_info, main_hand)
                        .await?;
                }
                if let Some(offhand) = offhand {
                    cli_dungeon_core::character::equip_offhand(&pool, &character_info, offhand)
                        .await?;
                }
                if let Some(armor) = armor {
                    cli_dungeon_core::character::equip_armor(&pool, &character_info, armor).await?;
                }
                if let Some(jewelry) = jewelry {
                    cli_dungeon_core::character::equip_jewelry(&pool, &character_info, jewelry)
                        .await?;
                }
            }
            CharacterCommands::Unequip { jewelry } => {
                let character_info = get_character(&pool).await;
                if let Some(jewelry) = jewelry {
                    cli_dungeon_core::character::unequip_jewelry(&pool, &character_info, jewelry)
                        .await?;
                }
            }
            CharacterCommands::Actions => {
                let character_info = get_character(&pool).await;
                let character = cli_dungeon_core::character::get_character(&pool, &character_info)
                    .await
                    .unwrap();

                let actions = character.available_actions();
                let bonus_actions = character.available_bonus_actions();

                let formatted_actions: Vec<_> = actions
                    .into_iter()
                    .map(|action| {
                        format!(
                            "{}{}",
                            action.name,
                            if action.requires_target {
                                " <TARGET>"
                            } else {
                                ""
                            }
                        )
                    })
                    .collect();

                let formatted_bonus_actions: Vec<_> = bonus_actions
                    .into_iter()
                    .map(|action| {
                        format!(
                            "{}{}",
                            action.name,
                            if action.requires_target {
                                " <TARGET>"
                            } else {
                                ""
                            }
                        )
                    })
                    .collect();

                cprintln!("<blue>Available actions</>");
                cprintln!("{}", formatted_actions.join(", "));
                cprintln!("<yellow>Available bonus actions</>");
                cprintln!("{}", formatted_bonus_actions.join(", "));
            }
        },
        Commands::Shop { command } => match command {
            ShopCommands::List => {
                let shop = cli_dungeon_core::shop::available_in_shop();

                cprintln!("<blue>Available in shop</>");
                cprintln!(
                    "Weapons: {}",
                    shop.weapons
                        .iter()
                        .map(|weapon| {
                            let weapon = weapon.to_weapon();
                            cformat!("<blue>{}: </><yellow>{}</>", weapon.name, weapon.cost)
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                cprintln!(
                    "Armor: {}",
                    shop.armor
                        .iter()
                        .map(|armor| {
                            let armor = armor.to_armor();
                            cformat!("<blue>{}: </><yellow>{}</>", armor.name, armor.cost)
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                );
                cprintln!(
                    "item: {}",
                    shop.items
                        .iter()
                        .map(|item| {
                            let item = item.to_item();
                            cformat!("<blue>{}: </><yellow>{}</>", item.name, item.cost)
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
            ShopCommands::Buy { item } => {
                let character_info = get_character(&pool).await;
                cli_dungeon_core::shop::buy(&pool, &character_info, item).await?;
            }
            ShopCommands::Sell { item } => {
                let character_info = get_character(&pool).await;
                cli_dungeon_core::shop::sell(&pool, &character_info, item).await?;
            }
        },
        Commands::Play { force } => {
            let character_info = get_character(&pool).await;
            if force {
                cli_dungeon_core::character::quest(&pool, &character_info).await?;
            }

            match play(&pool, force, &character_info).await? {
                cli_dungeon_core::PlayOutcome::NothingNew(status) => match status {
                    cli_dungeon_rules::Status::Resting => (),
                    cli_dungeon_rules::Status::Questing => (),
                    cli_dungeon_rules::Status::Fighting(_) => {
                        handle_encounter(&pool, &character_info).await;
                    }
                },
                cli_dungeon_core::PlayOutcome::NewFight((participants, outcome)) => {
                    cprintln!("<blue>New encounter: {}</>", participants.join(", "));
                    display_turn_outcome(outcome);
                    handle_encounter(&pool, &character_info).await;
                }
                cli_dungeon_core::PlayOutcome::CompletedQuest(loot) => {
                    let combined_loot: Vec<String> = loot
                        .weapons
                        .iter()
                        .map(|weapon| weapon.to_weapon().name)
                        .chain(loot.armor.iter().map(|armor| armor.to_armor().name))
                        .chain(loot.jewelry.iter().map(|jewelry| jewelry.to_jewelry().name))
                        .chain(loot.items.iter().map(|item| item.to_item().name))
                        .collect();

                    cprintln!("<blue>Quest completed!</>");
                    cprintln!("Loot: {}", combined_loot.join(" "));
                }
            }
        }
    }

    Ok(())
}

pub fn health_bar(current_health: HealthPoints, max_health: HealthPoints) -> String {
    let total_slots = 10;
    let filled_slots =
        ((*current_health as f32 / *max_health as f32) * total_slots as f32).round() as usize;
    let empty_slots = total_slots - filled_slots;

    let filled = "■".repeat(filled_slots);
    let empty = "─".repeat(empty_slots);

    cformat!("[<red>{}</>{}]", filled, empty)
}

fn quest_bar(current_quest: QuestPoint) -> String {
    let total_slots = 10;

    let mut filled_slots = ((*current_quest as f32 / 100f32) * total_slots as f32).round() as usize;

    if filled_slots > 10 {
        filled_slots = 10
    };

    let empty_slots = total_slots - filled_slots;

    let filled = "■".repeat(filled_slots);
    let empty = "─".repeat(empty_slots);

    cformat!("[<cyan>{}</>{}]", filled, empty)
}

fn experience_bar(current_xp: Experience, needed_xp: Experience) -> String {
    let total_slots = 10;
    let filled_slots =
        ((*current_xp as f32 / *needed_xp as f32) * total_slots as f32).round() as usize;
    let empty_slots = total_slots - filled_slots;

    let filled = "■".repeat(filled_slots);
    let empty = "─".repeat(empty_slots);

    cformat!("[<blue>{}</>{}]", filled, empty)
}

pub fn print_experience_bar(character: &Character) {
    if character.level() < character.experience_level() {
        cprintln!("<red>Can level up!</>");
    } else if let Some(next_xp) = character.experience_for_next_level() {
        cprintln!(
            "<white>Experience:</> {}",
            experience_bar(character.experience, next_xp)
        );
    } else {
        cprintln!("<green>Max level reached!</>");
    }
}

#[cfg(test)]
mod tests {
    use cli_dungeon_core::{character::get_character, errors::GameError};
    use cli_dungeon_rules::{
        armor::ArmorType,
        character::experience_gain,
        items::ItemType,
        jewelry::JewelryType,
        monsters::MonsterType,
        types::{ArmorPoints, Constitution, Dexterity, Gold, Level, Strength},
        weapons::WeaponType,
    };

    use crate::handle_encounter;

    #[sqlx::test]
    async fn can_die(pool: sqlx::Pool<sqlx::Sqlite>) {
        cli_dungeon_database::init(&pool).await;

        // Create

        let character_info =
            cli_dungeon_core::character::create_character(&pool, "testington".to_string(), None)
                .await;
        cli_dungeon_database::set_active_character(&pool, &character_info).await;
        cli_dungeon_core::character::quest(&pool, &character_info)
            .await
            .unwrap();

        loop {
            let character = cli_dungeon_database::get_character(&pool, &character_info.id)
                .await
                .unwrap();

            if !character.is_alive() {
                break;
            }

            let enemy = MonsterType::Wolf;
            let enemy_party_id = cli_dungeon_database::create_party(&pool).await;
            let enemy_id = cli_dungeon_database::create_monster(&pool, enemy, enemy_party_id)
                .await
                .id;

            let rotation = vec![character_info.id, enemy_id];

            let encounter_id =
                cli_dungeon_database::create_encounter(&pool, rotation.clone()).await;

            for character_id in rotation.iter() {
                cli_dungeon_database::set_encounter_id(&pool, character_id, Some(encounter_id))
                    .await;
            }

            handle_encounter(&pool, &character_info).await;
        }

        let character = cli_dungeon_database::get_character(&pool, &character_info.id)
            .await
            .unwrap();

        assert!(!character.is_alive());
    }

    #[sqlx::test]
    async fn it_works(pool: sqlx::Pool<sqlx::Sqlite>) {
        cli_dungeon_database::init(&pool).await;

        // Create
        let character_info =
            cli_dungeon_core::character::create_character(&pool, "testington".to_string(), None)
                .await;
        cli_dungeon_database::set_active_character(&pool, &character_info).await;

        let starting_character = cli_dungeon_database::get_character(&pool, &character_info.id)
            .await
            .unwrap();

        assert_eq!(starting_character.experience_level(), Level::new(0));

        let starting_gold = Gold::new(500);
        cli_dungeon_database::set_character_gold(&pool, &starting_character.id, starting_gold)
            .await;

        // Shop
        let main_hand = "shortsword".to_string();
        let offhand = "dagger".to_string();
        let armor = "leather".to_string();
        cli_dungeon_core::shop::buy(&pool, &character_info, main_hand.clone())
            .await
            .unwrap();
        cli_dungeon_core::shop::buy(&pool, &character_info, offhand.clone())
            .await
            .unwrap();
        cli_dungeon_core::shop::buy(&pool, &character_info, armor.clone())
            .await
            .unwrap();

        // Equip
        cli_dungeon_core::character::equip_main_hand(&pool, &character_info, main_hand.clone())
            .await
            .unwrap();
        cli_dungeon_core::character::equip_offhand(&pool, &character_info, offhand.clone())
            .await
            .unwrap();
        cli_dungeon_core::character::equip_armor(&pool, &character_info, armor.clone())
            .await
            .unwrap();

        // Start quest
        cli_dungeon_core::character::quest(&pool, &character_info)
            .await
            .unwrap();

        let enemy_party_id = cli_dungeon_database::create_party(&pool).await;

        let enemy_1 = MonsterType::TestMonsterWithDagger;
        let enemy_2 = MonsterType::TestMonsterWithLeatherArmor;
        let enemy_3 = MonsterType::TestMonsterWithRingOfProtectionAndStone;

        let enemy_1_id = cli_dungeon_database::create_monster(&pool, enemy_1, enemy_party_id)
            .await
            .id;
        let enemy_2_id = cli_dungeon_database::create_monster(&pool, enemy_2, enemy_party_id)
            .await
            .id;
        let enemy_3_id = cli_dungeon_database::create_monster(&pool, enemy_3, enemy_party_id)
            .await
            .id;

        let enemy_1 = cli_dungeon_database::get_character(&pool, &enemy_1_id)
            .await
            .unwrap();
        let enemy_2 = cli_dungeon_database::get_character(&pool, &enemy_2_id)
            .await
            .unwrap();
        let enemy_3 = cli_dungeon_database::get_character(&pool, &enemy_3_id)
            .await
            .unwrap();

        let rotation = vec![character_info.id, enemy_1_id, enemy_2_id, enemy_3_id];

        let encounter_id = cli_dungeon_database::create_encounter(&pool, rotation.clone()).await;

        for character_id in rotation.iter() {
            cli_dungeon_database::set_encounter_id(&pool, character_id, Some(encounter_id)).await;
        }

        handle_encounter(&pool, &character_info).await;

        let updated_character = get_character(&pool, &character_info).await.unwrap();
        let expected_gold = starting_gold
            - WeaponType::Dagger.to_weapon().cost
            - WeaponType::Shortsword.to_weapon().cost
            - ArmorType::Leather.to_armor().cost
            + enemy_1.gold
            + enemy_2.gold
            + enemy_3.gold;

        assert_eq!(updated_character.gold, expected_gold);

        assert_eq!(
            updated_character.experience,
            experience_gain(Level::new(0))
                + experience_gain(Level::new(0))
                + experience_gain(Level::new(0))
        );

        assert_eq!(
            updated_character.weapon_inventory,
            vec![
                WeaponType::Shortsword,
                WeaponType::Dagger,
                WeaponType::Dagger
            ]
        );

        assert_eq!(
            updated_character.armor_inventory,
            vec![ArmorType::Leather, ArmorType::Leather,]
        );

        assert_eq!(
            updated_character.jewelry_inventory,
            vec![JewelryType::RingOfProtection]
        );
        assert_eq!(updated_character.item_inventory, vec![ItemType::Stone]);

        for _ in 0..50 {
            cli_dungeon_core::character::rest(&pool, &character_info)
                .await
                .unwrap();
            cli_dungeon_core::character::quest(&pool, &character_info)
                .await
                .unwrap();

            let enemy = cli_dungeon_database::create_monster(
                &pool,
                MonsterType::TestMonster,
                enemy_party_id,
            )
            .await
            .id;
            let rotation = vec![character_info.id, enemy];
            let encounter_id =
                cli_dungeon_database::create_encounter(&pool, rotation.clone()).await;

            for character_id in rotation.iter() {
                cli_dungeon_database::set_encounter_id(&pool, character_id, Some(encounter_id))
                    .await;
            }

            handle_encounter(&pool, &character_info).await;
        }

        cli_dungeon_core::character::levelup(
            &pool,
            &character_info,
            "fighter".to_string(),
            "dexterity".to_string(),
        )
        .await
        .unwrap();

        let updated_character = get_character(&pool, &character_info).await.unwrap();
        assert_eq!(updated_character.experience_level(), Level::new(1));
        assert_eq!(updated_character.level(), Level::new(1));
        assert_eq!(
            updated_character.ability_scores().strength,
            Strength::new(8)
        );
        assert_eq!(
            updated_character.ability_scores().dexterity,
            Dexterity::new(8 + 1)
        );
        assert_eq!(
            updated_character.ability_scores().constitution,
            Constitution::new(8)
        );

        cli_dungeon_core::character::equip_jewelry(
            &pool,
            &character_info,
            "ring of protection".to_string(),
        )
        .await
        .unwrap();
        let updated_character = get_character(&pool, &character_info).await.unwrap();

        assert_eq!(
            updated_character.equipped_jewelry,
            vec![JewelryType::RingOfProtection]
        );

        assert_eq!(updated_character.armor_points(), ArmorPoints::new(12));

        cli_dungeon_core::character::unequip_jewelry(
            &pool,
            &character_info,
            "ring of protection".to_string(),
        )
        .await
        .unwrap();
        let updated_character = get_character(&pool, &character_info).await.unwrap();

        assert_eq!(updated_character.armor_points(), ArmorPoints::new(11));

        // Sell

        cli_dungeon_core::character::equip_jewelry(
            &pool,
            &character_info,
            "ring of protection".to_string(),
        )
        .await
        .unwrap();

        let updated_character = get_character(&pool, &character_info).await.unwrap();
        let before_sell_gold = updated_character.gold;

        cli_dungeon_core::shop::sell(&pool, &character_info, main_hand.clone())
            .await
            .unwrap();
        cli_dungeon_core::shop::sell(&pool, &character_info, offhand.clone())
            .await
            .unwrap();
        cli_dungeon_core::shop::sell(&pool, &character_info, offhand.clone())
            .await
            .unwrap();
        cli_dungeon_core::shop::sell(&pool, &character_info, armor.clone())
            .await
            .unwrap();
        cli_dungeon_core::shop::sell(&pool, &character_info, armor.clone())
            .await
            .unwrap();
        cli_dungeon_core::shop::sell(&pool, &character_info, "ring of protection".to_string())
            .await
            .unwrap();

        let after_character = get_character(&pool, &character_info).await.unwrap();
        let after_sell_gold = after_character.gold;
        assert_eq!(
            after_sell_gold,
            before_sell_gold
                + Gold::new(*WeaponType::Shortsword.to_weapon().cost / 2)
                + Gold::new(*WeaponType::Dagger.to_weapon().cost / 2)
                + Gold::new(*WeaponType::Dagger.to_weapon().cost / 2)
                + Gold::new(*ArmorType::Leather.to_armor().cost / 2)
                + Gold::new(*ArmorType::Leather.to_armor().cost / 2)
                + Gold::new(*JewelryType::RingOfProtection.to_jewelry().cost / 2)
        );

        assert_eq!(after_character.equipped_weapon, None);
        assert_eq!(after_character.equipped_offhand, None);
        assert_eq!(after_character.equipped_armor, None);
        assert_eq!(after_character.equipped_jewelry, vec![]);

        let error_sell =
            cli_dungeon_core::shop::sell(&pool, &character_info, offhand.clone()).await;
        assert_eq!(error_sell, Err(GameError::WeaponNotInInventory));
    }
}

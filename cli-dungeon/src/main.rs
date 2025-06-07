use anyhow::Result;
use clap::{Parser, Subcommand};
use cli_dungeon_core::play;
use cli_dungeon_rules::classes::ClassType;
use color_print::{cformat, cprint, cprintln};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create new Character
    CreateCharacter {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        strength: u16,
        #[arg(short, long)]
        dexterity: u16,
        #[arg(short, long)]
        constitution: u16,
    },

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
        off_hand: Option<String>,

        #[arg(short, long)]
        armor: Option<String>,
    },
    Rest,
    LevelUp {
        #[clap(subcommand)]
        command: LevelUpCommands,
    },
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::CreateCharacter {
            name,
            strength,
            dexterity,
            constitution,
        } => {
            let character_info = cli_dungeon_core::character::create_character(
                name,
                strength,
                dexterity,
                constitution,
            )
            .await?;
            cli_dungeon_database::set_active_character(character_info).await;
        }
        Commands::Character { command } => match command {
            CharacterCommands::Rest => {
                let character_info = cli_dungeon_database::get_active_character().await?;
                cli_dungeon_core::character::rest(&character_info).await?;
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
                    let character_info = cli_dungeon_database::get_active_character().await?;
                    cli_dungeon_core::character::levelup(&character_info, class, ability_increment)
                        .await?;
                }
            },
            CharacterCommands::Status => {
                let character_info = cli_dungeon_database::get_active_character().await?;
                let character = cli_dungeon_core::character::get_character(&character_info).await?;

                cprintln!("<red>Level: {}</>", character.level());
                cprintln!("<blue>Name: {}</>", character.name);
                if character.level() < character.experience_level() {
                    cprintln!("<red>Can level up!</>");
                }
                cprintln!("<yellow>Gold: {}</>", character.gold);
                cprintln!("<yellow>Current health: {}</>", character.current_health);
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
                cprintln!(
                    "Inventory: {} {}",
                    character
                        .weapon_inventory
                        .iter()
                        .map(|weapon| weapon.to_weapon().name)
                        .collect::<Vec<_>>()
                        .join(" "),
                    character
                        .armor_inventory
                        .iter()
                        .map(|armor| armor.to_armor().name)
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
            CharacterCommands::Equip {
                main_hand,
                off_hand,
                armor,
            } => {
                let character_info = cli_dungeon_database::get_active_character().await?;
                if let Some(main_hand) = main_hand {
                    cli_dungeon_core::character::equip_main_hand(&character_info, main_hand)
                        .await?;
                }
                if let Some(off_hand) = off_hand {
                    cli_dungeon_core::character::equip_offhand(&character_info, off_hand).await?;
                }
                if let Some(armor) = armor {
                    cli_dungeon_core::character::equip_armor(&character_info, armor).await?;
                }
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
            }
            ShopCommands::Buy { item } => {
                let character_info = cli_dungeon_database::get_active_character().await?;
                cli_dungeon_core::character::buy(&character_info, item).await?;
            }
        },
        Commands::Play { force } => {
            let character = cli_dungeon_database::get_active_character().await?;

            if let Some(outcome) = play(force, character).await? {
                println!("New encounter!");

                for outcome in outcome {
                    match outcome {
                        cli_dungeon_core::TurnOutcome::Miss(character_name) => {
                            cprintln!("<yellow>{} missed</>", character_name)
                        }
                        cli_dungeon_core::TurnOutcome::Attack(attack) => {
                            println!("{} attacked {}", attack.attacker_name, attack.attacked_name)
                        }
                        cli_dungeon_core::TurnOutcome::Hit(hit) => {
                            if hit.critical_hit {
                                cprint!("<red>Critical hit!</> ");
                            }
                            println!("{} took {} damage", hit.character_name, hit.damage);
                        }
                        cli_dungeon_core::TurnOutcome::Death(character_name) => {
                            cprintln!("<red>{} died</>", character_name)
                        }
                    }
                }
            };
        }
    }

    Ok(())
}

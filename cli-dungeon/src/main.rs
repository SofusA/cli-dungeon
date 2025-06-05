use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use cli_dungeon_core::play;
use cli_dungeon_rules::AbilityScores;
use color_print::{cprint, cprintln};

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

    // Play the game
    Play {
        #[arg(short, long, default_value_t = false)]
        /// Force a battle
        force: bool,
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
            if strength + dexterity + constitution != 10 {
                bail!("Ability scores must sum to 10");
            }
            let ability_scores = AbilityScores::new(8 + strength, 8 + dexterity, 8 + constitution);

            let character_info =
                cli_dungeon_database::create_character(&name, ability_scores).await;
            cli_dungeon_database::set_active_character(character_info).await;
        }
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

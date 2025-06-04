use clap::{Parser, Subcommand};
use cli_dungeon_core::play;
use cli_dungeon_rules::Dice;
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
    },

    // Play the game
    Play {
        #[arg(short, long, default_value_t = false)]
        /// Force a battle
        force: bool,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Commands::CreateCharacter { name } => {
            let character_info =
                cli_dungeon_database::create_character(&name, 24, 12, Dice::D8, 2).await;
            cli_dungeon_database::set_active_character(character_info).await;
        }
        Commands::Play { force } => {
            let Some(character) = cli_dungeon_database::get_active_character().await else {
                println!("No character. Create one with 'cli-dungeon create'");
                return;
            };

            if let Some(outcome) = play(force, character).await {
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
}

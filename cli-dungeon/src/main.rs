use cli_dungeon_core::play;
use color_print::{cprint, cprintln};

#[tokio::main]
async fn main() {
    if let Some(outcome) = play(true).await {
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

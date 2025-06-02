use cli_dungeon_rules::{Dice, Hitable, roll};

pub async fn play() {
    if roll(&Dice::D4) == 4 {
        encountor().await;
    }
}

async fn encountor() {
    println!("New encounter!");

    let player_id = cli_dungeon_database::create_character("Player", 24, 12, Dice::D8, 2).await;
    let monster_id = cli_dungeon_database::create_character("Spider", 24, 11, Dice::D6, 3).await;

    let player_initiative = (player_id, roll(&Dice::D20));
    let monster_initiative = (monster_id, roll(&Dice::D20));

    let mut rotation = [player_initiative, monster_initiative];
    rotation.sort_by_key(|initiative| initiative.1);
    rotation.reverse();

    battle(rotation.to_vec()).await;

    cli_dungeon_database::delete_character(player_id).await;
    cli_dungeon_database::delete_character(monster_id).await;
}

async fn battle(rotation: Vec<(i64, i16)>) {
    loop {
        for character_id in rotation.iter().map(|initiative| initiative.0) {
            let character = cli_dungeon_database::get_character(character_id).await;

            let other_character_id = rotation
                .iter()
                .filter(|character| character.0 != character_id)
                .map(|character| character.0)
                .next()
                .unwrap();

            let mut other_character = cli_dungeon_database::get_character(other_character_id).await;

            let outcome = other_character.attacked(&character.hit_bonus, &character.attack_dice);
            match outcome {
                Some(outcome) => {
                    if outcome.critical_hit {
                        print!("Critical hit! ");
                    }
                    println!(
                        "{} took {} damage from {}'s attack",
                        other_character.name, outcome.damage, character.name
                    );

                    cli_dungeon_database::set_character_health(
                        other_character_id,
                        other_character.current_health,
                    )
                    .await;

                    if !other_character.is_alive() {
                        println!("{} died", other_character.name);
                        return;
                    }
                }
                None => println!("{} missed", character.name),
            }
        }
    }
}

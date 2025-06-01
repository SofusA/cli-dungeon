use cli_dungeon_rules::{CanBeAttacked, Character, Dice, Monster, roll};

pub fn play() {
    let mut player = Character::new(1, "Player", 24, 12, Dice::D8, 2);
    let mut monster = Monster::new(2, "Monster", 24, 11, Dice::D6, 3);

    println!("New encounter!");

    let player_initiative = (*player.id(), roll(&Dice::D20));
    let monster_initiative = (*monster.id(), roll(&Dice::D20));

    let mut rotation = [player_initiative, monster_initiative];
    rotation.sort_by_key(|initiative| initiative.1);
    rotation.reverse();

    while monster.is_alive() && player.is_alive() {
        for turn in rotation {
            let is_players_turn = &turn.0 == player.id();

            match is_players_turn {
                true => {
                    let outcome = monster.attacked(player.hit_bonus(), player.attack_dice());
                    if let Some(outcome) = outcome {
                        println!(
                            "Monster was hit! Roll: {}, damage: {}",
                            outcome.roll, outcome.damage
                        );
                    }
                    if !monster.is_alive() {
                        println!("Monster died. Player win!");
                        break;
                    }
                }
                false => {
                    let outcome = player.attacked(monster.hit_bonus(), monster.attack_dice());
                    if let Some(outcome) = outcome {
                        println!(
                            "Player was hit! Roll: {}, damage: {}",
                            outcome.roll, outcome.damage
                        );
                    }
                    if !player.is_alive() {
                        println!("Player died. Player loses!");
                        break;
                    }
                }
            }
        }
    }
}

use anyhow::Result;
use clap::{Parser, Subcommand};
use cli_dungeon_core::{Action, BonusAction, TurnOutcome, play, take_turn};
use cli_dungeon_database::{CharacterInfo, Pool};
use cli_dungeon_rules::{Encounter, classes::ClassType, items::ActionType};
use color_print::{cformat, cprint, cprintln};
use rhai::{CustomType, Dynamic, Engine, ImmutableString, Map, Scope, TypeBuilder};

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
        strength: i16,
        #[arg(short, long)]
        dexterity: i16,
        #[arg(short, long)]
        constitution: i16,
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
    // TODO: Sell
}

#[derive(Subcommand)]
enum RestCommands {
    Long,
    Short,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let pool = cli_dungeon_database::get_pool().await;
    cli_dungeon_database::init(&pool).await;

    match args.command {
        Commands::CreateCharacter {
            name,
            strength,
            dexterity,
            constitution,
        } => {
            let character_info = cli_dungeon_core::character::create_character(
                &pool,
                name,
                strength,
                dexterity,
                constitution,
            )
            .await?;
            cli_dungeon_database::set_active_character(&pool, &character_info).await;
        }
        Commands::Character { command } => match command {
            CharacterCommands::Rest { command } => match command {
                RestCommands::Long => {
                    let character_info = cli_dungeon_database::get_active_character(&pool).await?;
                    cli_dungeon_core::character::rest(&pool, &character_info).await?;
                }
                RestCommands::Short => {
                    let character_info = cli_dungeon_database::get_active_character(&pool).await?;
                    cli_dungeon_core::character::short_rest(&pool, &character_info).await?;
                }
            },
            CharacterCommands::Quest => {
                let character_info = cli_dungeon_database::get_active_character(&pool).await?;
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
                    let character_info = cli_dungeon_database::get_active_character(&pool).await?;
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
                let character_info = cli_dungeon_database::get_active_character(&pool).await?;
                let character =
                    cli_dungeon_core::character::get_character(&pool, &character_info).await?;

                cprintln!("<red>Level: {}</>", character.level());
                cprintln!("<blue>Name: {}</>", character.name);
                if character.level() < character.experience_level() {
                    cprintln!("<red>Can level up!</>");
                }
                cprintln!("<yellow>Gold: {}</>", character.gold);
                cprintln!(
                    "<yellow>Health: {}/{}</>",
                    character.current_health,
                    character.max_health()
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
                cprintln!(
                    "Inventory: {} {} {} {}",
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
                        .join(" "),
                    character
                        .jewelry_inventory
                        .iter()
                        .map(|jewelry| jewelry.to_jewelry().name)
                        .collect::<Vec<_>>()
                        .join(" "),
                    character
                        .item_inventory
                        .iter()
                        .map(|item| item.to_item().name)
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            }
            CharacterCommands::Equip {
                main_hand,
                off_hand,
                armor,
                jewelry,
            } => {
                let character_info = cli_dungeon_database::get_active_character(&pool).await?;
                if let Some(main_hand) = main_hand {
                    cli_dungeon_core::character::equip_main_hand(&pool, &character_info, main_hand)
                        .await?;
                }
                if let Some(off_hand) = off_hand {
                    cli_dungeon_core::character::equip_offhand(&pool, &character_info, off_hand)
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
                let character_info = cli_dungeon_database::get_active_character(&pool).await?;
                if let Some(jewelry) = jewelry {
                    cli_dungeon_core::character::unequip_jewelry(&pool, &character_info, jewelry)
                        .await?;
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
                let character_info = cli_dungeon_database::get_active_character(&pool).await?;
                cli_dungeon_core::character::buy(&pool, &character_info, item).await?;
            }
        },
        Commands::Play { force } => {
            let character_info = cli_dungeon_database::get_active_character(&pool).await?;

            match play(&pool, force, &character_info).await? {
                cli_dungeon_core::PlayOutcome::NothingNew(status) => match status {
                    cli_dungeon_rules::Status::Resting => (),
                    cli_dungeon_rules::Status::Questing => (),
                    cli_dungeon_rules::Status::Fighting(_) => {
                        handle_encounter(&pool, &character_info).await;
                    }
                },
                cli_dungeon_core::PlayOutcome::NewFight(outcome) => {
                    display_turn_outcome(outcome);
                    handle_encounter(&pool, &character_info).await;
                }
            }
        }
    }

    Ok(())
}

fn display_turn_outcome(outcome: Vec<TurnOutcome>) {
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
            cli_dungeon_core::TurnOutcome::StartTurn(character_name) => {
                cprintln!("<green>It is {}'s turn!</>", character_name)
            }
        }
    }
}

#[derive(Clone, CustomType)]
struct EncounterCharacter {
    id: i64,
    name: ImmutableString,
    health: i16,
    max_health: i16,
}

impl From<EncounterCharacter> for Dynamic {
    fn from(val: EncounterCharacter) -> Self {
        let mut map = Map::new();
        map.insert("id".into(), Dynamic::from(val.id));
        map.insert("name".into(), Dynamic::from(val.name));
        map.insert("health".into(), Dynamic::from(val.health));
        map.insert("max_health".into(), Dynamic::from(val.max_health));
        Dynamic::from_map(map)
    }
}

#[derive(Clone, CustomType)]
struct EncounterState {
    player: EncounterCharacter,
    enemies: Vec<Dynamic>,
    available_actions: Vec<Dynamic>,
    available_bonus_actions: Vec<Dynamic>,
}

#[derive(Debug, Clone, CustomType)]
#[rhai_type(extra = Self::build_extra)]
struct EncounterAction {
    action: ImmutableString,
    action_target: Option<i64>,
    bonus_action: ImmutableString,
    bonus_action_target: Option<i64>,
}

impl EncounterAction {
    fn new(
        action: ImmutableString,
        action_target: i64,
        bonus_action: ImmutableString,
        bonus_action_target: i64,
    ) -> EncounterAction {
        EncounterAction {
            action,
            action_target: Some(action_target),
            bonus_action,
            bonus_action_target: Some(bonus_action_target),
        }
    }
    fn new_action_no_target_no_bonus_action(action: ImmutableString) -> EncounterAction {
        EncounterAction {
            action,
            action_target: None,
            bonus_action: Default::default(),
            bonus_action_target: None,
        }
    }
    fn new_no_bonus_action(action: ImmutableString, action_target: i64) -> EncounterAction {
        EncounterAction {
            action,
            action_target: Some(action_target),
            bonus_action: Default::default(),
            bonus_action_target: None,
        }
    }
    fn new_bonus_action(
        bonus_action: ImmutableString,
        bonus_action_target: i64,
    ) -> EncounterAction {
        EncounterAction {
            action: Default::default(),
            action_target: None,
            bonus_action,
            bonus_action_target: Some(bonus_action_target),
        }
    }
    fn new_bonus_action_no_target(bonus_action: ImmutableString) -> EncounterAction {
        EncounterAction {
            action: Default::default(),
            action_target: None,
            bonus_action,
            bonus_action_target: None,
        }
    }

    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder.with_fn("react", EncounterAction::new);
        builder.with_fn("react", EncounterAction::new_bonus_action);
        builder.with_fn("react", EncounterAction::new_no_bonus_action);
        builder.with_fn(
            "react",
            EncounterAction::new_action_no_target_no_bonus_action,
        );
        builder.with_fn("react", EncounterAction::new_bonus_action_no_target);
    }

    fn to_action(&self) -> Option<Action> {
        match self.action.as_str() {
            "attack" => self.action_target.map(Action::Attack),
            _ => None,
        }
    }

    fn to_bonus_action(&self) -> Option<BonusAction> {
        match self.bonus_action.as_str() {
            "attack" => self.bonus_action_target.map(BonusAction::OffHandAttack),
            _ => None,
        }
    }
}

impl EncounterState {
    fn from_encounter(encounter: Encounter, player_id: i64) -> Option<Self> {
        let player = encounter
            .rotation
            .iter()
            .find(|character| character.id == player_id)?;

        let mut actions: Vec<_> = player
            .item_inventory
            .iter()
            .map(|item| item.to_item())
            .filter(|item| matches!(item.action, ActionType::Action(_)))
            .map(|item| item.name)
            .map(|x| x.into())
            .collect();
        actions.push("attack".into());

        let mut bonus_actions: Vec<_> = player
            .item_inventory
            .iter()
            .map(|item| item.to_item())
            .filter(|item| matches!(item.action, ActionType::BonusAction(_)))
            .map(|item| item.name)
            .map(|x| x.into())
            .collect();
        bonus_actions.push("attack".into());

        let player = EncounterCharacter {
            id: player.id,
            name: player.name.clone().into(),
            health: *player.current_health,
            max_health: *player.max_health(),
        };

        let enemies = encounter
            .rotation
            .into_iter()
            .filter(|character| character.id != player_id)
            .map(|character| EncounterCharacter {
                id: character.id,
                name: character.name.clone().into(),
                health: *character.current_health,
                max_health: *character.max_health(),
            })
            .map(|x| x.into())
            .collect();

        Some(Self {
            player,
            enemies,
            available_actions: actions,
            available_bonus_actions: bonus_actions,
        })
    }
}

async fn handle_encounter(pool: &Pool, character_info: &CharacterInfo) {
    let script_path = {
        let mut config = dirs::config_dir().unwrap();
        config.push("cli-dungeon");
        config.push("encounter.rhai");
        config
    };

    let mut engine = Engine::new();
    engine
        .build_type::<EncounterState>()
        .build_type::<EncounterAction>()
        .build_type::<EncounterCharacter>();

    let ast = engine.compile_file(script_path).unwrap();

    loop {
        let Ok(encounter) = cli_dungeon_core::get_encounter(pool, character_info).await else {
            return;
        };

        let Some(encounter) = EncounterState::from_encounter(encounter.clone(), character_info.id)
        else {
            return;
        };

        let mut scope = Scope::new();
        scope.push("state", encounter);

        let result: EncounterAction = engine.eval_ast_with_scope(&mut scope, &ast).unwrap();
        let action = result.to_action();

        let bonus_action = result.to_bonus_action();

        let outcome = take_turn(pool, character_info, action, bonus_action)
            .await
            .unwrap();
        display_turn_outcome(outcome);
    }
}

#[cfg(test)]
mod tests {
    use cli_dungeon_core::character::get_character;
    use cli_dungeon_rules::{
        armor::ArmorType,
        experience_gain,
        items::ItemType,
        jewelry::JewelryType,
        monsters::MonsterType,
        types::{ArmorPoints, Constitution, Dexterity, Level, Strength},
        weapons::WeaponType,
    };

    use crate::handle_encounter;

    #[sqlx::test]
    async fn it_works(pool: sqlx::Pool<sqlx::Sqlite>) {
        cli_dungeon_database::init(&pool).await;

        // Create
        let starting_str_bonus = 0;
        let starting_dex_bonus = 6;
        let starting_con_bonus = 4;

        let character_info = cli_dungeon_core::character::create_character(
            &pool,
            "testington".to_string(),
            starting_str_bonus,
            starting_dex_bonus,
            starting_con_bonus,
        )
        .await
        .unwrap();
        cli_dungeon_database::set_active_character(&pool, &character_info).await;

        let starting_character = cli_dungeon_database::get_character(&pool, &character_info.id)
            .await
            .unwrap();

        assert_eq!(starting_character.experience_level(), Level::new(0));

        let starting_gold = starting_character.gold;

        // Shop
        let main_hand = "shortsword".to_string();
        let off_hand = "dagger".to_string();
        let armor = "leather".to_string();
        cli_dungeon_core::character::buy(&pool, &character_info, main_hand.clone())
            .await
            .unwrap();
        cli_dungeon_core::character::buy(&pool, &character_info, off_hand.clone())
            .await
            .unwrap();
        cli_dungeon_core::character::buy(&pool, &character_info, armor.clone())
            .await
            .unwrap();

        // Equip
        cli_dungeon_core::character::equip_main_hand(&pool, &character_info, main_hand)
            .await
            .unwrap();
        cli_dungeon_core::character::equip_offhand(&pool, &character_info, off_hand)
            .await
            .unwrap();
        cli_dungeon_core::character::equip_armor(&pool, &character_info, armor)
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
            + enemy_1.to_monster().gold
            + enemy_2.to_monster().gold
            + enemy_3.to_monster().gold;

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

        for _ in 0..2 {
            cli_dungeon_core::character::rest(&pool, &character_info)
                .await
                .unwrap();
            cli_dungeon_core::character::quest(&pool, &character_info)
                .await
                .unwrap();

            let enemy = cli_dungeon_database::create_monster(
                &pool,
                MonsterType::TestMonsterWithDagger,
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
            Strength::new(8 + starting_str_bonus)
        );
        assert_eq!(
            updated_character.ability_scores().dexterity,
            Dexterity::new(8 + starting_dex_bonus + 1)
        );
        assert_eq!(
            updated_character.ability_scores().constitution,
            Constitution::new(8 + starting_con_bonus)
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

        assert_eq!(updated_character.armor_points(), ArmorPoints::new(14));

        cli_dungeon_core::character::unequip_jewelry(
            &pool,
            &character_info,
            "ring of protection".to_string(),
        )
        .await
        .unwrap();
        let updated_character = get_character(&pool, &character_info).await.unwrap();

        assert_eq!(updated_character.armor_points(), ArmorPoints::new(13));

        // TODO: Assert qusting points
        // TODO: Assert conditions
        // TODO: Assert short rests
    }
}

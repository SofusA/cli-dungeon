use std::io::Write;

use cli_dungeon_core::turn::{Action, BonusAction, TurnOutcome, take_turn};
use cli_dungeon_database::{CharacterInfo, Pool};
use cli_dungeon_rules::{Encounter, character::Character, items::ActionType};
use color_print::{cprint, cprintln};
use rhai::{CustomType, Dynamic, Engine, ImmutableString, Map, Scope, TypeBuilder};

use crate::{config_path, health_bar, print_experience_bar};

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
    fn new_action_with_target_bonus_no_target(
        action: ImmutableString,
        action_target: i64,
        bonus_action: ImmutableString,
    ) -> EncounterAction {
        EncounterAction {
            action,
            action_target: Some(action_target),
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
        builder.with_fn(
            "react",
            EncounterAction::new_action_with_target_bonus_no_target,
        );
    }

    fn to_action(&self, character: &Character) -> Option<Action> {
        character
            .available_actions()
            .into_iter()
            .find(|action| action.name.to_lowercase() == self.action.to_lowercase())
            .and_then(|action| match action.action {
                cli_dungeon_rules::character::AvailableAction::Attack => {
                    self.action_target.map(Action::Attack)
                }
                cli_dungeon_rules::character::AvailableAction::Item(item) => {
                    match action.requires_target {
                        true => self
                            .action_target
                            .map(|target| Action::ItemWithTarget(item, target)),
                        false => Some(Action::Item(item)),
                    }
                }
            })
    }

    fn to_bonus_action(&self, character: &Character) -> Option<BonusAction> {
        character
            .available_bonus_actions()
            .into_iter()
            .find(|action| action.name.to_lowercase() == self.bonus_action.to_lowercase())
            .and_then(|action| match action.action {
                cli_dungeon_rules::character::AvailableAction::Attack => {
                    self.bonus_action_target.map(BonusAction::OffhandAttack)
                }
                cli_dungeon_rules::character::AvailableAction::Item(item) => {
                    match action.requires_target {
                        true => self
                            .action_target
                            .map(|target| BonusAction::ItemWithTarget(item, target)),
                        false => Some(BonusAction::Item(item)),
                    }
                }
            })
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

fn default_encounter_script<'a>() -> &'a str {
    r#"// print(state.available_actions); // List available actions
// print(state.available_bonus_actions); // List available bonus actions

let enemies = state.enemies;

// Sort enemies by health
enemies.sort(|b, a| a.health - b.health);

// Attack with both action and bonus action on lowest health
let target = state.enemies[0];
react("attack", target.id, "attack", target.id)
    "#
}

fn run_script_get_action(
    active_character_id: i64,
    encounter: Encounter,
    ast: &rhai::AST,
    engine: &rhai::Engine,
) -> Option<(Option<Action>, Option<BonusAction>)> {
    let character = encounter
        .rotation
        .iter()
        .find(|character| character.id == active_character_id)
        .unwrap();

    let encounter = EncounterState::from_encounter(encounter.clone(), active_character_id)?;

    let mut scope = Scope::new();
    scope.push("state", encounter);

    let result: EncounterAction = engine.eval_ast_with_scope(&mut scope, ast).unwrap();
    let action = result.to_action(character);

    let bonus_action = result.to_bonus_action(character);

    Some((action, bonus_action))
}

fn build_engine() -> Engine {
    let mut engine = Engine::new();
    engine
        .build_type::<EncounterState>()
        .build_type::<EncounterAction>()
        .build_type::<EncounterCharacter>();

    engine
}

pub fn ensure_default_script() -> std::path::PathBuf {
    let script_path = {
        let mut config = config_path();
        config.push("encounter.rhai");
        config
    };

    if !script_path.exists() {
        std::fs::create_dir_all(script_path.parent().unwrap()).unwrap();

        let mut file = std::fs::File::create(&script_path).unwrap();
        write!(file, "{}", default_encounter_script()).unwrap();
    }

    script_path
}

pub(crate) async fn handle_encounter(pool: &Pool, character_info: &CharacterInfo) {
    let script_path = ensure_default_script();

    let engine = build_engine();
    let ast = engine.compile_file(script_path).unwrap();

    loop {
        let Ok(encounter) = cli_dungeon_core::get_encounter(pool, character_info).await else {
            break;
        };

        let result = run_script_get_action(character_info.id, encounter, &ast, &engine);

        let Some(result) = result else {
            return;
        };

        let outcome = take_turn(pool, character_info, result.0, result.1)
            .await
            .unwrap();

        display_turn_outcome(outcome);
    }

    if let Ok(character) = cli_dungeon_core::character::get_character(pool, character_info).await {
        if character.level() < character.experience_level() {
            cprintln!("<red>Can level up!</>");
        }
        cprintln!(
            "<white>Health:</> {}",
            health_bar(character.current_health, character.max_health())
        );
        print_experience_bar(&character);
    };
}

pub fn display_turn_outcome(outcome: Vec<TurnOutcome>) {
    for outcome in outcome {
        match outcome {
            TurnOutcome::Attack(attack) => {
                cprint!(
                    "{} attacked {}: ",
                    attack.attacker_name,
                    attack.attacked_name
                )
            }
            TurnOutcome::Miss(character_name) => cprintln!("<white>{} missed</>", character_name),
            TurnOutcome::Hit(hit) => {
                if hit.critical_hit {
                    cprint!("<red>Critical hit!</> ");
                }
                cprintln!("<yellow>{} damage</>", hit.damage);
            }
            TurnOutcome::Death(character_name) => cprintln!("<red>{} died</>", character_name),
            TurnOutcome::ConditionSet((character_name, condition)) => {
                cprintln!("<yellow>{} got {} condition</>", character_name, condition)
            }
            TurnOutcome::Healed((character_name, health)) => cprintln!(
                "<red>{} healed by {} health points</>",
                character_name,
                health
            ),
            TurnOutcome::GoldReceived((_character_name, gold)) => {
                cprintln!("<green><yellow>+{} gold</></>", gold)
            }
            TurnOutcome::LootReceived((character_name, loot)) => {
                let combined_loot: Vec<String> = loot
                    .weapons
                    .iter()
                    .map(|weapon| weapon.to_weapon().name)
                    .chain(loot.armor.iter().map(|armor| armor.to_armor().name))
                    .chain(loot.jewelry.iter().map(|jewelry| jewelry.to_jewelry().name))
                    .chain(loot.items.iter().map(|item| item.to_item().name))
                    .collect();
                if !combined_loot.is_empty() {
                    cprintln!(
                        "<green>{} received loot: {}</>",
                        character_name,
                        combined_loot.join(" ")
                    );
                }
            }
            TurnOutcome::UsedItem((character, item)) => {
                cprintln!("<white>{} used {}</>", character, item.to_item().name)
            }
            TurnOutcome::UsedItemOn((character, item, target)) => {
                cprintln!(
                    "<white>{} used {} on {}</>",
                    character,
                    item.to_item().name,
                    target
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use cli_dungeon_core::turn::{Action, BonusAction};
    use cli_dungeon_rules::{
        Encounter, Status,
        abilities::AbilityScores,
        character::{Character, CharacterType},
        items::ItemType,
        monsters::MonsterType,
        types::{Constitution, Dexterity, Experience, Gold, HealthPoints, QuestPoint, Strength},
    };

    use crate::encounter::{build_engine, default_encounter_script, run_script_get_action};

    fn setup() -> (Encounter, Character) {
        let scroll_of_weaken = ItemType::ScrollOfWeaken;
        let healing_potion = ItemType::PotionOfHealing;
        let character = Character {
            id: 1,
            name: "Testington".to_string(),
            character_type: CharacterType::Player,
            current_health: HealthPoints::new(10),
            base_ability_scores: AbilityScores {
                strength: Strength::new(8),
                dexterity: Dexterity::new(8),
                constitution: Constitution::new(8),
            },
            gold: Gold::new(0),
            experience: Experience::new(0),
            equipped_weapon: None,
            equipped_offhand: None,
            equipped_armor: None,
            equipped_jewelry: vec![],
            weapon_inventory: vec![],
            armor_inventory: vec![],
            jewelry_inventory: vec![],
            item_inventory: vec![scroll_of_weaken, healing_potion],
            level_up_choices: vec![],
            status: Status::Questing,
            party: 1,
            quest_points: QuestPoint::new(0),
            short_rests_available: 2,
            active_conditions: vec![],
        };

        let monster = Character {
            id: 2,
            name: "monster".to_string(),
            character_type: CharacterType::Monster(MonsterType::Wolf),
            current_health: HealthPoints::new(10),
            base_ability_scores: AbilityScores {
                strength: Strength::new(8),
                dexterity: Dexterity::new(8),
                constitution: Constitution::new(8),
            },
            gold: Gold::new(0),
            experience: Experience::new(0),
            equipped_weapon: None,
            equipped_offhand: None,
            equipped_armor: None,
            equipped_jewelry: vec![],
            weapon_inventory: vec![],
            armor_inventory: vec![],
            jewelry_inventory: vec![],
            item_inventory: vec![scroll_of_weaken, healing_potion],
            level_up_choices: vec![],
            status: Status::Questing,
            party: 1,
            quest_points: QuestPoint::new(0),
            short_rests_available: 2,
            active_conditions: vec![],
        };

        let encounter = Encounter {
            id: 1,
            rotation: vec![character.clone(), monster],
            dead_characters: vec![],
        };

        (encounter, character)
    }

    #[test]
    fn default_script_can_run() {
        let engine = build_engine();
        let (encounter, character) = setup();

        let script = default_encounter_script();

        let ast = engine.compile(script).unwrap();

        let result = run_script_get_action(character.id, encounter, &ast, &engine).unwrap();

        assert!(result.0.is_some());
        assert!(result.1.is_some());
    }

    #[test]
    fn scripts_can_use_items() {
        let engine = build_engine();
        let (encounter, character) = setup();

        let script = r#"
            react("scrollofweaken", 2, "PotionOfHealing");
        "#;

        let ast = engine.compile(script).unwrap();

        let result = run_script_get_action(character.id, encounter, &ast, &engine).unwrap();

        assert_eq!(
            result.0.unwrap(),
            Action::ItemWithTarget(ItemType::ScrollOfWeaken, 2)
        );
        assert_eq!(
            result.1.unwrap(),
            BonusAction::Item(ItemType::PotionOfHealing)
        );
    }

    #[test]
    fn scripts_can_use_attacks() {
        let engine = build_engine();
        let (encounter, character) = setup();

        let script = r#"
            react("attack", 2, "attack", 2);
        "#;

        let ast = engine.compile(script).unwrap();

        let result = run_script_get_action(character.id, encounter, &ast, &engine).unwrap();

        assert_eq!(result.0.unwrap(), Action::Attack(2));
        assert_eq!(result.1.unwrap(), BonusAction::OffhandAttack(2));
    }
}

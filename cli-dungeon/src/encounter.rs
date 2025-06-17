use std::io::Write;

use cli_dungeon_core::turn::{Action, BonusAction, TurnOutcome, take_turn};
use cli_dungeon_database::{CharacterInfo, Pool};
use cli_dungeon_rules::{Encounter, character::Character, items::ActionType};
use color_print::{cprint, cprintln};
use rhai::{CustomType, Dynamic, Engine, ImmutableString, Map, Scope, TypeBuilder};

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

    fn to_action(&self, character: &Character) -> Option<Action> {
        character
            .available_actions()
            .into_iter()
            .find(|action| action.name == self.action)
            .and_then(|action| match action.action {
                cli_dungeon_rules::character::AvailableAction::Attack => {
                    self.action_target.map(Action::Attack)
                }
                cli_dungeon_rules::character::AvailableAction::Item(item_action) => {
                    match action.requires_target {
                        true => self
                            .action_target
                            .map(|target| Action::ItemWithTarget(item_action, target)),
                        false => Some(Action::Item(item_action)),
                    }
                }
            })
    }

    fn to_bonus_action(&self, character: &Character) -> Option<BonusAction> {
        character
            .available_bonus_actions()
            .into_iter()
            .find(|action| action.name == self.action)
            .and_then(|action| match action.action {
                cli_dungeon_rules::character::AvailableAction::Attack => {
                    self.action_target.map(BonusAction::OffhandAttack)
                }
                cli_dungeon_rules::character::AvailableAction::Item(item_action) => {
                    match action.requires_target {
                        true => self
                            .action_target
                            .map(|target| BonusAction::ItemWithTarget(item_action, target)),
                        false => Some(BonusAction::Item(item_action)),
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
    r#"
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

pub(crate) async fn handle_encounter(pool: &Pool, character_info: &CharacterInfo) {
    let script_path = {
        let mut config = dirs::config_dir().unwrap();
        config.push("cli-dungeon");
        config.push("encounter.rhai");
        config
    };

    let engine = build_engine();

    if !script_path.exists() {
        let mut file = std::fs::File::create(&script_path).unwrap();
        write!(file, "{}", default_encounter_script()).unwrap();
    }

    let ast = engine.compile_file(script_path).unwrap();

    loop {
        let Ok(encounter) = cli_dungeon_core::get_encounter(pool, character_info).await else {
            return;
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
}

pub fn display_turn_outcome(outcome: Vec<TurnOutcome>) {
    for outcome in outcome {
        match outcome {
            TurnOutcome::Miss(character_name) => {
                cprintln!("<yellow>{} missed</>", character_name)
            }
            TurnOutcome::Attack(attack) => {
                println!("{} attacked {}", attack.attacker_name, attack.attacked_name)
            }
            TurnOutcome::Hit(hit) => {
                if hit.critical_hit {
                    cprint!("<red>Critical hit!</> ");
                }
                println!("{} took {} damage", hit.character_name, hit.damage);
            }
            TurnOutcome::Death(character_name) => {
                cprintln!("<red>{} died</>", character_name)
            }
            TurnOutcome::StartTurn(character_name) => {
                cprintln!("<green>It is {}'s turn!</>", character_name)
            }
            TurnOutcome::ConditionSet((character_name, condition)) => {
                cprintln!("<yellow>{} got {} condition</>", character_name, condition)
            }
            TurnOutcome::Healed((character_name, health)) => {
                cprintln!(
                    "<red>{} healed by {} health points</>",
                    character_name,
                    health
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use cli_dungeon_rules::{
        Encounter, Status,
        abilities::AbilityScores,
        character::Character,
        items::ItemType,
        types::{Constitution, Dexterity, Experience, Gold, HealthPoints, QuestPoint, Strength},
    };

    use crate::encounter::{build_engine, default_encounter_script, run_script_get_action};

    fn setup() -> (i64, Encounter) {
        let scroll_of_weaken = ItemType::ScrollOfWeaken;
        let healing_potion = ItemType::MinorHealingPotion;
        let character = Character {
            id: 1,
            name: "Testington".to_string(),
            player: true,
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
            player: false,
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

        let character_id = character.id;

        let encounter = Encounter {
            id: 1,
            rotation: vec![character, monster],
            dead_characters: vec![],
        };

        (character_id, encounter)
    }

    #[test]
    fn default_script_can_run() {
        let engine = build_engine();
        let (character_id, encounter) = setup();

        let script = default_encounter_script();

        let ast = engine.compile(script).unwrap();

        let result = run_script_get_action(character_id, encounter, &ast, &engine).unwrap();

        assert!(result.0.is_some());
        assert!(result.1.is_some());
    }

    // TODO: Test items
    // Scroll of weaken
    // Health potion
    // TODO: Test action attack
    // TODO: Test bonus action attack
}

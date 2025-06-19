use std::collections::HashSet;

use cli_dungeon_database::{CharacterInfo, Pool};
use cli_dungeon_rules::{
    AttackStats, Encounter, Hit, Status,
    armor::ArmorType,
    character::{Character, CharacterWeapon, experience_gain},
    items::{ItemAction, ItemType},
    jewelry::JewelryType,
    spells::SpellAction,
    types::{Experience, Gold, HealthPoints, Level, Turn},
    weapons::WeaponType,
};
use rand::seq::{IndexedRandom, IteratorRandom};

use crate::{errors::GameError, validate_player};

pub(crate) async fn advance_turn(pool: &Pool, character: &Character) {
    let new_conditions: Vec<_> = character
        .active_conditions
        .clone()
        .into_iter()
        .filter_map(|mut condition| {
            if let Some(duration) = &mut condition.duration {
                if *duration == Turn::new(0) {
                    return None;
                }

                *duration -= Turn::new(1);
            }
            Some(condition)
        })
        .collect();

    cli_dungeon_database::set_character_conditions(pool, &character.id, new_conditions).await;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    Attack(i64),
    Item(ItemAction),
    ItemWithTarget(ItemAction, i64),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BonusAction {
    OffhandAttack(i64),
    Item(ItemAction),
    ItemWithTarget(ItemAction, i64),
}

pub(crate) async fn monster_take_turn(
    pool: &Pool,
    monster: &Character,
    encounter: &Encounter,
) -> Vec<TurnOutcome> {
    let target = encounter
        .rotation
        .iter()
        .filter(|character| character.party != monster.party)
        .map(|character| character.id)
        .choose(&mut rand::rng());

    let action = target.map(Action::Attack);
    let bonus_action = target.map(BonusAction::OffhandAttack);

    character_take_turn(pool, monster, encounter, action, bonus_action).await
}

async fn handle_attack(
    pool: &Pool,
    active_character: &Character,
    attack_stats: &AttackStats,
    target: i64,
    rotation: &mut Vec<Character>,
    dead_list: &mut Vec<Character>,
) -> Vec<TurnOutcome> {
    let mut outcome_list = vec![];

    if !rotation
        .iter()
        .map(|character| character.id)
        .collect::<Vec<_>>()
        .contains(&target)
    {
        outcome_list.push(TurnOutcome::Miss(active_character.name.clone()));
        return outcome_list;
    }
    let mut target = cli_dungeon_database::get_character(pool, &target)
        .await
        .unwrap();
    let action_outcome = target.attacked(attack_stats);

    match action_outcome {
        Some(outcome) => {
            outcome_list.push(TurnOutcome::Hit(outcome));
            cli_dungeon_database::set_character_health(pool, &target.id, target.current_health)
                .await;

            if !target.is_alive() {
                outcome_list.push(TurnOutcome::Death(target.name.clone()));

                rotation.retain(|character| character.id != target.id);
                let same_party: Vec<_> = rotation
                    .iter()
                    .filter(|character| character.party == active_character.party)
                    .collect();

                let experience_gained = Experience::new(
                    *experience_gain(Level::new(target.level_up_choices.len() as u16))
                        / same_party.len() as u32,
                );

                dead_list.push(target);

                for character_info in same_party {
                    let character = cli_dungeon_database::get_character(pool, &character_info.id)
                        .await
                        .unwrap();
                    let new_xp = character.experience + experience_gained;
                    cli_dungeon_database::set_character_experience(
                        pool,
                        &character_info.id,
                        new_xp,
                    )
                    .await;
                }
            }
        }
        None => {
            outcome_list.push(TurnOutcome::Miss(active_character.name.clone()));
        }
    }

    outcome_list
}

async fn character_take_turn(
    pool: &Pool,
    active_character: &Character,
    encounter: &Encounter,
    action: Option<Action>,
    bonus_action: Option<BonusAction>,
) -> Vec<TurnOutcome> {
    let mut outcome_list = vec![];
    let mut new_dead_list = encounter.dead_characters.clone();
    let mut new_rotation = encounter.rotation.clone();

    outcome_list.push(TurnOutcome::StartTurn(active_character.name.clone()));

    if let Some(action) = action {
        match action {
            Action::Attack(target) => {
                let mut outcome = handle_attack(
                    pool,
                    active_character,
                    &active_character.attack_stats(CharacterWeapon::Mainhand),
                    target,
                    &mut new_rotation,
                    &mut new_dead_list,
                )
                .await;
                outcome_list.append(&mut outcome);
            }
            Action::Item(item) => match item {
                cli_dungeon_rules::items::ItemAction::Spell(spell_type) => match spell_type {
                    SpellAction::Condition(active_condition) => {
                        cli_dungeon_database::set_character_conditions(
                            pool,
                            &active_character.id,
                            vec![active_condition],
                        )
                        .await;
                        outcome_list.push(TurnOutcome::ConditionSet((
                            active_condition.condition_type.to_condition().name,
                            active_character.name.clone(),
                        )));
                    }
                    SpellAction::Projectile(_) => (),
                },
                cli_dungeon_rules::items::ItemAction::Healing(health_stats) => {
                    let health_roll = health_stats.roll();

                    let new_health = active_character.current_health + health_roll;
                    cli_dungeon_database::set_character_health(
                        pool,
                        &active_character.id,
                        new_health,
                    )
                    .await;
                    outcome_list.push(TurnOutcome::Healed((
                        active_character.name.clone(),
                        health_roll,
                    )));
                }
                cli_dungeon_rules::items::ItemAction::Projectile(_) => (),
            },
            Action::ItemWithTarget(item, target) => {
                let target = encounter
                    .rotation
                    .iter()
                    .find(|character| character.id == target)
                    .unwrap();

                match item {
                    cli_dungeon_rules::items::ItemAction::Spell(spell_type) => match spell_type {
                        SpellAction::Condition(active_condition) => {
                            cli_dungeon_database::set_character_conditions(
                                pool,
                                &target.id,
                                vec![active_condition],
                            )
                            .await;
                            outcome_list.push(TurnOutcome::ConditionSet((
                                active_condition.condition_type.to_condition().name,
                                target.name.clone(),
                            )));
                        }
                        SpellAction::Projectile(attack_stats) => {
                            let attack = active_character.spell_stats(attack_stats);
                            let mut outcome = handle_attack(
                                pool,
                                active_character,
                                &attack,
                                target.id,
                                &mut new_rotation,
                                &mut new_dead_list,
                            )
                            .await;
                            outcome_list.append(&mut outcome);
                        }
                    },
                    cli_dungeon_rules::items::ItemAction::Healing(health_stats) => {
                        let health_roll = health_stats.roll();
                        let new_health = target.current_health + health_roll;
                        cli_dungeon_database::set_character_health(pool, &target.id, new_health)
                            .await;
                        outcome_list.push(TurnOutcome::Healed((target.name.clone(), health_roll)));
                    }
                    cli_dungeon_rules::items::ItemAction::Projectile(projectile_attack_stats) => {
                        let attack = active_character
                            .attack_stats(CharacterWeapon::Thrown(projectile_attack_stats));
                        let mut outcome = handle_attack(
                            pool,
                            active_character,
                            &attack,
                            target.id,
                            &mut new_rotation,
                            &mut new_dead_list,
                        )
                        .await;
                        outcome_list.append(&mut outcome);
                    }
                }
            }
        }
    }

    if let Some(action) = bonus_action {
        match action {
            BonusAction::OffhandAttack(target) => {
                let mut outcome = handle_attack(
                    pool,
                    active_character,
                    &active_character.attack_stats(CharacterWeapon::Offhand),
                    target,
                    &mut new_rotation,
                    &mut new_dead_list,
                )
                .await;
                outcome_list.append(&mut outcome);
            }
            BonusAction::Item(item) => match item {
                cli_dungeon_rules::items::ItemAction::Spell(spell_type) => match spell_type {
                    SpellAction::Condition(active_condition) => {
                        cli_dungeon_database::set_character_conditions(
                            pool,
                            &active_character.id,
                            vec![active_condition],
                        )
                        .await;
                        outcome_list.push(TurnOutcome::ConditionSet((
                            active_condition.condition_type.to_condition().name,
                            active_character.name.clone(),
                        )));
                    }
                    SpellAction::Projectile(_) => (),
                },
                cli_dungeon_rules::items::ItemAction::Healing(health_stats) => {
                    let health_roll = health_stats.roll();
                    let new_health = active_character.current_health + health_roll;
                    cli_dungeon_database::set_character_health(
                        pool,
                        &active_character.id,
                        new_health,
                    )
                    .await;
                    outcome_list.push(TurnOutcome::Healed((
                        active_character.name.clone(),
                        health_roll,
                    )));
                }
                cli_dungeon_rules::items::ItemAction::Projectile(_) => (),
            },
            BonusAction::ItemWithTarget(item, target) => {
                let target = encounter
                    .rotation
                    .iter()
                    .find(|character| character.id == target)
                    .unwrap();

                match item {
                    cli_dungeon_rules::items::ItemAction::Spell(spell_type) => match spell_type {
                        SpellAction::Condition(active_condition) => {
                            cli_dungeon_database::set_character_conditions(
                                pool,
                                &target.id,
                                vec![active_condition],
                            )
                            .await;
                            outcome_list.push(TurnOutcome::ConditionSet((
                                active_condition.condition_type.to_condition().name,
                                target.name.clone(),
                            )));
                        }
                        SpellAction::Projectile(attack_stats) => {
                            let attack = active_character.spell_stats(attack_stats);
                            let mut outcome = handle_attack(
                                pool,
                                active_character,
                                &attack,
                                target.id,
                                &mut new_rotation,
                                &mut new_dead_list,
                            )
                            .await;
                            outcome_list.append(&mut outcome);
                        }
                    },
                    cli_dungeon_rules::items::ItemAction::Healing(health_stats) => {
                        let health_roll = health_stats.roll();
                        let new_health = target.current_health + health_roll;
                        cli_dungeon_database::set_character_health(pool, &target.id, new_health)
                            .await;
                        outcome_list.push(TurnOutcome::Healed((target.name.clone(), health_roll)));
                    }
                    cli_dungeon_rules::items::ItemAction::Projectile(projectile_attack_stats) => {
                        let attack = active_character
                            .attack_stats(CharacterWeapon::Thrown(projectile_attack_stats));
                        let mut outcome = handle_attack(
                            pool,
                            active_character,
                            &attack,
                            target.id,
                            &mut new_rotation,
                            &mut new_dead_list,
                        )
                        .await;
                        outcome_list.append(&mut outcome);
                    }
                }
            }
        }
    }

    let parties_left = {
        let unique_party_ids: HashSet<i64> = new_rotation.iter().map(|p| p.party).collect();
        unique_party_ids.len()
    };

    if parties_left == 1 {
        let total_gold: u16 = new_dead_list.iter().map(|character| *character.gold).sum();
        let split_gold = total_gold / new_rotation.len() as u16;

        let weapon_loot: Vec<WeaponType> = new_dead_list
            .iter()
            .flat_map(|character| character.weapon_inventory.clone())
            .collect();
        let armor_loot: Vec<ArmorType> = new_dead_list
            .iter()
            .flat_map(|character| character.armor_inventory.clone())
            .collect();
        let jewelry_loot: Vec<JewelryType> = new_dead_list
            .iter()
            .flat_map(|character| character.jewelry_inventory.clone())
            .collect();
        let item_loot: Vec<ItemType> = new_dead_list
            .iter()
            .flat_map(|character| character.item_inventory.clone())
            .collect();

        for character in new_rotation.iter() {
            let new_gold = character.gold + Gold::new(split_gold);
            cli_dungeon_database::set_character_gold(pool, &character.id, new_gold).await;
            cli_dungeon_database::set_character_status(pool, &character.id, Status::Questing).await;
        }

        for weapon in weapon_loot {
            let recipient = new_rotation.choose(&mut rand::rng()).unwrap();

            cli_dungeon_database::add_weapon_to_inventory(pool, &recipient.id, weapon)
                .await
                .unwrap();
        }
        for armor in armor_loot {
            let recipient = new_rotation.choose(&mut rand::rng()).unwrap();

            cli_dungeon_database::add_armor_to_inventory(pool, &recipient.id, armor)
                .await
                .unwrap();
        }
        for jewelry in jewelry_loot {
            let recipient = new_rotation.choose(&mut rand::rng()).unwrap();

            cli_dungeon_database::add_jewelry_to_inventory(pool, &recipient.id, jewelry)
                .await
                .unwrap();
        }
        for item in item_loot {
            let recipient = new_rotation.choose(&mut rand::rng()).unwrap();

            cli_dungeon_database::add_item_to_inventory(pool, &recipient.id, item)
                .await
                .unwrap();
        }
    }

    if let Some(first) = new_rotation.first().cloned() {
        new_rotation.remove(0);
        new_rotation.push(first);
    }

    cli_dungeon_database::update_encounter(
        pool,
        encounter.id,
        new_rotation.iter().map(|character| character.id).collect(),
        new_dead_list.iter().map(|character| character.id).collect(),
    )
    .await;

    advance_turn(pool, active_character).await;

    outcome_list
}

pub async fn take_turn(
    pool: &Pool,
    character_info: &CharacterInfo,
    action: Option<Action>,
    bonus_action: Option<BonusAction>,
) -> Result<Vec<TurnOutcome>, GameError> {
    let mut outcome = vec![];

    validate_player(pool, character_info).await?;

    let active_character = cli_dungeon_database::get_character(pool, &character_info.id)
        .await
        .unwrap();

    let Status::Fighting(encounter_id) = active_character.status else {
        return Err(GameError::NotFighting);
    };

    let encounter = cli_dungeon_database::get_encounter(pool, &encounter_id)
        .await
        .unwrap();

    if encounter.rotation.first().unwrap().id != active_character.id {
        return Err(GameError::NotPlayerTurn);
    }

    outcome.append(
        &mut character_take_turn(pool, &active_character, &encounter, action, bonus_action).await,
    );

    loop {
        let encounter = cli_dungeon_database::get_encounter(pool, &encounter_id)
            .await
            .unwrap();

        match encounter.rotation.first() {
            Some(turn) => match &turn.character_type {
                cli_dungeon_rules::character::CharacterType::Player => break,
                cli_dungeon_rules::character::CharacterType::Monster(_) => {
                    outcome.append(&mut monster_take_turn(pool, turn, &encounter).await);
                }
            },
            None => break,
        }
    }

    Ok(outcome)
}

#[derive(Debug, Clone)]
pub enum TurnOutcome {
    Miss(String),
    Attack(Attack),
    Hit(Hit),
    Death(String),
    StartTurn(String),
    ConditionSet((String, String)),
    Healed((String, HealthPoints)),
}

#[derive(Debug, Clone)]
pub struct Attack {
    pub attacker_name: String,
    pub attacked_name: String,
}

#[cfg(test)]
mod tests {
    use cli_dungeon_rules::{
        abilities::AbilityType,
        armor::ArmorType,
        classes::{ClassType, LevelUpChoice},
        conditions::{ActiveCondition, ConditionType},
        items::HealingStats,
        monsters::MonsterType,
        spells::SpellType,
        types::{HealthPoints, Turn},
        weapons::WeaponType,
    };
    use rand::seq::SliceRandom;

    use crate::{
        advance_turn,
        character::create_character,
        turn::{Action, BonusAction, character_take_turn},
    };

    #[sqlx::test]
    async fn can_skip_turn(pool: sqlx::Pool<sqlx::Sqlite>) {
        cli_dungeon_database::init(&pool).await;
        let monster_1 = MonsterType::TestMonsterWithDagger;
        let monster_2 = MonsterType::TestMonsterWithLeatherArmor;

        let party_1 = cli_dungeon_database::create_party(&pool).await;
        let party_2 = cli_dungeon_database::create_party(&pool).await;

        let monster_1 = cli_dungeon_database::create_monster(&pool, monster_1, party_1)
            .await
            .id;
        let monster_2 = cli_dungeon_database::create_monster(&pool, monster_2, party_2)
            .await
            .id;

        let rotation = vec![monster_1, monster_2];

        let encounter = cli_dungeon_database::create_encounter(&pool, rotation).await;
        let encounter = cli_dungeon_database::get_encounter(&pool, &encounter)
            .await
            .unwrap();

        let monster_1 = cli_dungeon_database::get_character(&pool, &monster_1)
            .await
            .unwrap();
        let monster_2 = cli_dungeon_database::get_character(&pool, &monster_2)
            .await
            .unwrap();

        let character_turn = encounter.rotation.first().unwrap();
        assert_eq!(character_turn.id, monster_1.id);

        character_take_turn(&pool, &monster_1, &encounter, None, None).await;

        let encounter = cli_dungeon_database::get_encounter(&pool, &encounter.id)
            .await
            .unwrap();
        let character_turn = encounter.rotation.first().unwrap();
        assert_eq!(character_turn.id, monster_2.id);
    }

    #[sqlx::test]
    async fn can_heal(pool: sqlx::Pool<sqlx::Sqlite>) {
        cli_dungeon_database::init(&pool).await;
        let monster_1 = MonsterType::TestMonsterWithDagger;
        let monster_2 = MonsterType::TestMonsterWithLeatherArmor;

        let party_1 = cli_dungeon_database::create_party(&pool).await;
        let party_2 = cli_dungeon_database::create_party(&pool).await;

        let monster_1 = cli_dungeon_database::create_monster(&pool, monster_1, party_1)
            .await
            .id;
        let monster_2 = cli_dungeon_database::create_monster(&pool, monster_2, party_2)
            .await
            .id;

        cli_dungeon_database::set_character_health(&pool, &monster_1, HealthPoints::new(1)).await;

        let rotation = vec![monster_1, monster_2];

        let encounter = cli_dungeon_database::create_encounter(&pool, rotation).await;
        let encounter = cli_dungeon_database::get_encounter(&pool, &encounter)
            .await
            .unwrap();

        let monster_1 = cli_dungeon_database::get_character(&pool, &monster_1)
            .await
            .unwrap();

        character_take_turn(
            &pool,
            &monster_1,
            &encounter,
            None,
            Some(crate::turn::BonusAction::Item(
                cli_dungeon_rules::items::ItemAction::Healing(HealingStats {
                    dice: vec![],
                    bonus: HealthPoints::new(1),
                }),
            )),
        )
        .await;

        let monster_1 = cli_dungeon_database::get_character(&pool, &monster_1.id)
            .await
            .unwrap();

        assert_eq!(monster_1.current_health, HealthPoints::new(2));
    }

    #[sqlx::test]
    async fn can_set_conditions(pool: sqlx::Pool<sqlx::Sqlite>) {
        cli_dungeon_database::init(&pool).await;
        let monster_1 = MonsterType::TestMonsterWithDagger;
        let monster_2 = MonsterType::TestMonsterWithLeatherArmor;

        let party_1 = cli_dungeon_database::create_party(&pool).await;
        let party_2 = cli_dungeon_database::create_party(&pool).await;

        let monster_1 = cli_dungeon_database::create_monster(&pool, monster_1, party_1)
            .await
            .id;
        let monster_2 = cli_dungeon_database::create_monster(&pool, monster_2, party_2)
            .await
            .id;

        cli_dungeon_database::set_character_health(&pool, &monster_1, HealthPoints::new(1)).await;

        let rotation = vec![monster_1, monster_2];

        let encounter = cli_dungeon_database::create_encounter(&pool, rotation).await;
        let encounter = cli_dungeon_database::get_encounter(&pool, &encounter)
            .await
            .unwrap();

        let monster_1 = cli_dungeon_database::get_character(&pool, &monster_1)
            .await
            .unwrap();

        character_take_turn(
            &pool,
            &monster_1,
            &encounter,
            None,
            Some(crate::turn::BonusAction::ItemWithTarget(
                cli_dungeon_rules::items::ItemAction::Spell(SpellType::Weaken.spell_action()),
                monster_2,
            )),
        )
        .await;

        let monster_2 = cli_dungeon_database::get_character(&pool, &monster_2)
            .await
            .unwrap();

        assert_eq!(
            monster_2.active_conditions,
            vec![SpellType::Weaken.active_condition().unwrap()]
        );

        advance_turn(&pool, &monster_2).await;

        let monster_2 = cli_dungeon_database::get_character(&pool, &monster_2.id)
            .await
            .unwrap();

        assert_eq!(
            monster_2.active_conditions,
            vec![ActiveCondition {
                duration: Some(Turn::new(1)),
                condition_type: ConditionType::Weaken
            }]
        );

        advance_turn(&pool, &monster_2).await;
        let monster_2 = cli_dungeon_database::get_character(&pool, &monster_2.id)
            .await
            .unwrap();

        advance_turn(&pool, &monster_2).await;
        let monster_2 = cli_dungeon_database::get_character(&pool, &monster_2.id)
            .await
            .unwrap();

        assert_eq!(monster_2.active_conditions, vec![]);
    }

    #[sqlx::test]
    async fn str_and_dex_builds_are_equal(pool: sqlx::Pool<sqlx::Sqlite>) {
        cli_dungeon_database::init(&pool).await;
        let rounds = 100;
        let mut dex_wins = 0;
        let mut str_wins = 0;

        for _ in 0..rounds {
            let dex = create_character(&pool, "dex".to_string(), 0, 6, 4)
                .await
                .unwrap();

            cli_dungeon_database::equip_armor(&pool, &dex.id, ArmorType::Leather).await;
            cli_dungeon_database::equip_weapon(&pool, &dex.id, WeaponType::Shortsword).await;
            cli_dungeon_database::equip_offhand(&pool, &dex.id, WeaponType::Shortsword).await;

            let str = create_character(&pool, "str".to_string(), 6, 0, 4)
                .await
                .unwrap();

            cli_dungeon_database::equip_armor(&pool, &str.id, ArmorType::Chainmail).await;
            cli_dungeon_database::equip_weapon(&pool, &str.id, WeaponType::Longsword).await;
            cli_dungeon_database::equip_offhand(&pool, &str.id, WeaponType::Shield).await;

            let mut rotation = vec![dex.id, str.id];
            let encounter_id =
                cli_dungeon_database::create_encounter(&pool, rotation.clone()).await;

            for character_id in rotation.iter() {
                cli_dungeon_database::set_encounter_id(&pool, character_id, Some(encounter_id))
                    .await;
            }

            rotation.shuffle(&mut rand::rng());

            loop {
                let encounter = cli_dungeon_database::get_encounter(&pool, &encounter_id)
                    .await
                    .unwrap();

                if !encounter.dead_characters.is_empty() {
                    let dead = encounter.dead_characters.first().unwrap();
                    if dead.name == "str" {
                        dex_wins += 1;
                    }

                    if dead.name == "dex" {
                        str_wins += 1;
                    }
                    break;
                }

                let first = encounter.rotation.first().unwrap();
                let last = encounter.rotation.last().unwrap();

                character_take_turn(
                    &pool,
                    first,
                    &encounter,
                    Some(Action::Attack(last.id)),
                    Some(BonusAction::OffhandAttack(last.id)),
                )
                .await;
            }
        }

        println!("dex wins: {dex_wins}");
        println!("str wins: {str_wins}");

        let win_dif: i32 = dex_wins - str_wins;
        let win_dif = win_dif.abs();

        assert!(win_dif < 20);
    }

    #[sqlx::test]
    async fn high_level_str_and_dex_builds_are_equal(pool: sqlx::Pool<sqlx::Sqlite>) {
        cli_dungeon_database::init(&pool).await;
        let rounds = 100;
        let mut dex_wins = 0;
        let mut str_wins = 0;

        for _ in 0..rounds {
            let dex = create_character(&pool, "dex".to_string(), 0, 6, 4)
                .await
                .unwrap();

            cli_dungeon_database::equip_armor(&pool, &dex.id, ArmorType::Leather).await;
            cli_dungeon_database::equip_weapon(&pool, &dex.id, WeaponType::Shortsword).await;
            cli_dungeon_database::equip_offhand(&pool, &dex.id, WeaponType::Shortsword).await;
            for _ in 0..2 {
                cli_dungeon_database::add_level_up_choice(
                    &pool,
                    &dex.id,
                    LevelUpChoice {
                        ability_increment: AbilityType::Dexterity,
                        class: ClassType::Fighter,
                    },
                )
                .await
                .unwrap();
            }

            let str = create_character(&pool, "str".to_string(), 6, 0, 4)
                .await
                .unwrap();

            cli_dungeon_database::equip_armor(&pool, &str.id, ArmorType::Splint).await;
            cli_dungeon_database::equip_weapon(&pool, &str.id, WeaponType::Longsword).await;
            cli_dungeon_database::equip_offhand(&pool, &str.id, WeaponType::Shield).await;
            for _ in 0..2 {
                cli_dungeon_database::add_level_up_choice(
                    &pool,
                    &str.id,
                    LevelUpChoice {
                        ability_increment: AbilityType::Strength,
                        class: ClassType::Fighter,
                    },
                )
                .await
                .unwrap();
            }

            let mut rotation = vec![dex.id, str.id];
            let encounter_id =
                cli_dungeon_database::create_encounter(&pool, rotation.clone()).await;

            for character_id in rotation.iter() {
                cli_dungeon_database::set_encounter_id(&pool, character_id, Some(encounter_id))
                    .await;
            }

            rotation.shuffle(&mut rand::rng());

            loop {
                let encounter = cli_dungeon_database::get_encounter(&pool, &encounter_id)
                    .await
                    .unwrap();

                if !encounter.dead_characters.is_empty() {
                    let dead = encounter.dead_characters.first().unwrap();
                    if dead.name == "str" {
                        dex_wins += 1;
                    }

                    if dead.name == "dex" {
                        str_wins += 1;
                    }
                    break;
                }

                let first = encounter.rotation.first().unwrap();
                let last = encounter.rotation.last().unwrap();

                character_take_turn(
                    &pool,
                    first,
                    &encounter,
                    Some(Action::Attack(last.id)),
                    Some(BonusAction::OffhandAttack(last.id)),
                )
                .await;
            }
        }

        println!("dex wins: {dex_wins}");
        println!("str wins: {str_wins}");

        let win_dif: i32 = dex_wins - str_wins;
        let win_dif = win_dif.abs();

        assert!(win_dif < 20);
    }
}

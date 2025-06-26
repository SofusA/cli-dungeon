use crate::{
    AttackStats, Dice, Hit, Status,
    abilities::{AbilityScaling, AbilityScores, AbilityType},
    armor::ArmorType,
    classes::LevelUpChoice,
    conditions::ActiveCondition,
    items::{ActionType, ItemAction, ItemType},
    jewelry::JewelryType,
    monsters::MonsterType,
    roll,
    types::{
        AbilityScoreBonus, ArmorPoints, Constitution, Dexterity, Experience, Gold, HealthPoints,
        Level, QuestPoint, Strength,
    },
    weapons::{WeaponAttackStats, WeaponType},
};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum CharacterType {
    Player,
    Monster(MonsterType),
}

#[derive(Clone)]
pub struct Character {
    pub id: i64,
    pub name: String,
    pub character_type: CharacterType,
    pub current_health: HealthPoints,
    pub base_ability_scores: AbilityScores,
    pub gold: Gold,
    pub experience: Experience,
    pub equipped_weapon: Option<WeaponType>,
    pub equipped_offhand: Option<WeaponType>,
    pub equipped_armor: Option<ArmorType>,
    pub equipped_jewelry: Vec<JewelryType>,
    pub weapon_inventory: Vec<WeaponType>,
    pub armor_inventory: Vec<ArmorType>,
    pub jewelry_inventory: Vec<JewelryType>,
    pub item_inventory: Vec<ItemType>,
    pub level_up_choices: Vec<LevelUpChoice>,
    pub status: Status,
    pub party: i64,
    pub quest_points: QuestPoint,
    pub short_rests_available: u16,
    pub active_conditions: Vec<ActiveCondition>,
}

const EXPERIENCE_THRESHOLDS: [u32; 12] = [
    100, 600, 2000, 6500, 8500, 14000, 23000, 34000, 48000, 64000, 85000, 100000,
];

pub fn max_health(constitution: &Constitution, level: Level) -> HealthPoints {
    let health = 12 + 6 * *level as i16 + *constitution.ability_score_bonus() * 4;
    HealthPoints::new(health)
}

fn attack(attack_stats: &AttackStats) -> HealthPoints {
    let damage =
        attack_stats.attack_dice.iter().map(roll).sum::<i16>() + *attack_stats.attack_bonus;
    HealthPoints::new(damage)
}

pub fn experience_gain(level: Level) -> Experience {
    let level = *level as usize;
    match level {
        0 => Experience::new(10),
        1 => Experience::new(30),
        2 => Experience::new(50),
        3 => Experience::new(100),
        4 => Experience::new(300),
        5 => Experience::new(500),
        6 => Experience::new(750),
        7 => Experience::new(1000),
        8 => Experience::new(1500),
        9 => Experience::new(2000),
        10 => Experience::new(2750),
        11 => Experience::new(3500),
        12 => Experience::new(5000),
        _ => Experience::new(0),
    }
}

pub enum CharacterWeapon {
    Mainhand,
    Offhand,
    Thrown(WeaponAttackStats),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AvailableAction {
    Attack,
    Item(ItemType),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AvailableActionDefinition {
    pub name: String,
    pub action: AvailableAction,
    pub requires_target: bool,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AvailableBonusActionDefinition {
    pub name: String,
    pub action: AvailableAction,
    pub requires_target: bool,
}

impl Character {
    pub fn ability_scores(&self) -> AbilityScores {
        let base_ability_scores = &self.base_ability_scores;

        let strength_level_bonus = Strength::new(
            self.level_up_choices
                .iter()
                .filter(|choice| choice.ability_increment == AbilityType::Strength)
                .count() as i16,
        );

        let strength_condition_bonus = Strength::new(
            self.active_conditions
                .iter()
                .flat_map(|condition| condition.condition_type.to_condition().strength_bonus)
                .map(|strength| **strength)
                .sum(),
        );

        let strength_jewelry_bonus = Strength::new(
            self.equipped_jewelry
                .iter()
                .map(|j| j.to_jewelry())
                .map(|j| **j.strength_bonus)
                .sum(),
        );

        let dexterity_level_bonus = Dexterity::new(
            self.level_up_choices
                .iter()
                .filter(|choice| choice.ability_increment == AbilityType::Dexterity)
                .count() as i16,
        );

        let dexterity_condition_bonus = Dexterity::new(
            self.active_conditions
                .iter()
                .flat_map(|condition| condition.condition_type.to_condition().dexterity_bonus)
                .map(|dexterity| **dexterity)
                .sum(),
        );

        let dexterity_jewelry_bonus = Dexterity::new(
            self.equipped_jewelry
                .iter()
                .map(|j| j.to_jewelry())
                .map(|j| **j.dexterity_bonus)
                .sum(),
        );

        let constitution_level_bonus = Constitution::new(
            self.level_up_choices
                .iter()
                .filter(|choice| choice.ability_increment == AbilityType::Constitution)
                .count() as i16,
        );

        let constitution_condition_bonus = Constitution::new(
            self.active_conditions
                .iter()
                .flat_map(|condition| condition.condition_type.to_condition().constitution_bonus)
                .map(|constitution| **constitution)
                .sum(),
        );

        let constitution_jewelry_bonus = Constitution::new(
            self.equipped_jewelry
                .iter()
                .map(|j| j.to_jewelry())
                .map(|j| **j.constitution_bonus)
                .sum(),
        );

        AbilityScores {
            strength: base_ability_scores.strength
                + strength_level_bonus
                + strength_condition_bonus
                + strength_jewelry_bonus,
            dexterity: base_ability_scores.dexterity
                + dexterity_level_bonus
                + dexterity_condition_bonus
                + dexterity_jewelry_bonus,
            constitution: base_ability_scores.constitution
                + constitution_level_bonus
                + constitution_condition_bonus
                + constitution_jewelry_bonus,
        }
    }

    pub fn healing_potion(&self) -> Option<ItemType> {
        self.item_inventory
            .iter()
            .find(|x| matches!(x.item_action(), ItemAction::Healing(_)))
            .copied()
    }

    pub fn available_actions(&self) -> Vec<AvailableActionDefinition> {
        let mut actions: Vec<_> = self
            .item_inventory
            .iter()
            .flat_map(|item| match item.to_item().action {
                ActionType::Action(item_action) => Some((item, item_action)),
                ActionType::BonusAction(_) => None,
            })
            .map(|item| match item.1 {
                ItemAction::Spell(_) => (item.0, item.1, true),
                ItemAction::Projectile(_) => (item.0, item.1, true),
                ItemAction::Healing(_) => (item.0, item.1, false),
            })
            .map(|item| AvailableActionDefinition {
                name: item.0.to_item().name,
                action: AvailableAction::Item(*item.0),
                requires_target: item.2,
            })
            .collect();

        actions.push(AvailableActionDefinition {
            name: "attack".to_string(),
            action: AvailableAction::Attack,
            requires_target: true,
        });

        actions
    }

    pub fn available_bonus_actions(&self) -> Vec<AvailableBonusActionDefinition> {
        let mut actions: Vec<_> = self
            .item_inventory
            .iter()
            .flat_map(|item| match item.to_item().action {
                ActionType::BonusAction(item_action) => Some((item, item_action)),
                ActionType::Action(_) => None,
            })
            .map(|item| match item.1 {
                ItemAction::Spell(_) => (item.0, item.1, true),
                ItemAction::Projectile(_) => (item.0, item.1, true),
                ItemAction::Healing(_) => (item.0, item.1, false),
            })
            .map(|item| AvailableBonusActionDefinition {
                name: item.0.to_item().name,
                action: AvailableAction::Item(*item.0),
                requires_target: item.2,
            })
            .collect();

        if self.can_attack_with_offhand() {
            actions.push(AvailableBonusActionDefinition {
                name: "attack".to_string(),
                action: AvailableAction::Attack,
                requires_target: true,
            });
        }

        actions
    }

    pub fn can_attack_with_offhand(&self) -> bool {
        self.equipped_offhand
            .is_some_and(|offhand| !offhand.to_weapon().attack_stats.attack_dices.is_empty())
            || self.equipped_offhand.is_none()
    }

    pub fn experience_level(&self) -> Level {
        for (level, &threshold) in EXPERIENCE_THRESHOLDS.iter().enumerate() {
            if *self.experience < threshold {
                return Level::new(level as u16);
            }
        }
        Level::new(EXPERIENCE_THRESHOLDS.len() as u16)
    }

    pub fn experience_for_next_level(&self) -> Option<Experience> {
        EXPERIENCE_THRESHOLDS
            .get(*self.level() as usize)
            .copied()
            .map(Experience::new)
    }

    pub fn level(&self) -> Level {
        Level::new(self.level_up_choices.len() as u16)
    }

    pub fn max_health(&self) -> HealthPoints {
        max_health(&self.ability_scores().constitution, self.level())
    }

    pub fn spell_stats(&self, spell_stats: WeaponAttackStats) -> AttackStats {
        let dex = &self.ability_scores().dexterity;
        let str = &self.ability_scores().strength;

        let ability_bonus = match spell_stats.primary_ability {
            AbilityScaling::Strength => str.ability_score_bonus(),
            AbilityScaling::Dexterity => dex.ability_score_bonus(),
            AbilityScaling::Versatile => match **dex < **str {
                true => str.ability_score_bonus(),
                false => dex.ability_score_bonus(),
            },
            AbilityScaling::Wisdom => AbilityScoreBonus::new(0),
        };

        let attack_dice = spell_stats.attack_dices;

        AttackStats {
            attack_dice,
            attack_bonus: ability_bonus + spell_stats.attack_bonus,
            hit_bonus: ability_bonus,
        }
    }

    pub fn attack_stats(&self, weapon_type: CharacterWeapon) -> AttackStats {
        let dex = &self.ability_scores().dexterity;
        let str = &self.ability_scores().strength;

        let weapon = match &weapon_type {
            CharacterWeapon::Mainhand => self
                .equipped_weapon
                .map(|weapon| weapon.to_weapon())
                .filter(|weapon| &weapon.strength_requirement < str)
                .map(|weapon| weapon.attack_stats),
            CharacterWeapon::Offhand => self
                .equipped_offhand
                .map(|weapon| weapon.to_weapon())
                .filter(|weapon| &weapon.strength_requirement < str)
                .map(|weapon| weapon.attack_stats),
            CharacterWeapon::Thrown(weapon_attack_stats) => Some(weapon_attack_stats).cloned(),
        };

        let Some(weapon) = weapon else {
            let ability_bonus = match **dex < **str {
                true => str.ability_score_bonus(),
                false => dex.ability_score_bonus(),
            };
            return AttackStats {
                attack_dice: vec![Dice::D4],
                attack_bonus: ability_bonus,
                hit_bonus: ability_bonus,
            };
        };

        let ability_bonus = match weapon_type {
            CharacterWeapon::Mainhand => match weapon.primary_ability {
                AbilityScaling::Strength => str.ability_score_bonus(),
                AbilityScaling::Dexterity => dex.ability_score_bonus(),
                AbilityScaling::Versatile => match **dex < **str {
                    true => str.ability_score_bonus(),
                    false => dex.ability_score_bonus(),
                },
                AbilityScaling::Wisdom => AbilityScoreBonus::new(0),
            },
            CharacterWeapon::Offhand => AbilityScoreBonus::new(0),
            CharacterWeapon::Thrown(_) => str.ability_score_bonus(),
        };

        let attack_dice = if self.equipped_offhand.is_none() {
            weapon.versatile_attack_dices.unwrap_or(weapon.attack_dices)
        } else {
            weapon.attack_dices
        };

        AttackStats {
            attack_dice,
            attack_bonus: ability_bonus + weapon.attack_bonus,
            hit_bonus: ability_bonus,
        }
    }

    pub fn is_alive(&self) -> bool {
        *self.current_health > 0
    }

    pub fn armor_points(&self) -> ArmorPoints {
        let armor = self.equipped_armor.as_ref().map(|armor| armor.to_armor());
        let base_armor = ArmorPoints::new(10);
        let dexterity_ability_score_bonus = self.ability_scores().dexterity.ability_score_bonus();

        let dexterity_bonus: ArmorPoints =
            match armor.as_ref().map(|armor| armor.max_dexterity_bonus) {
                Some(max_bonus) => {
                    if *dexterity_ability_score_bonus > *max_bonus {
                        max_bonus.into()
                    } else {
                        dexterity_ability_score_bonus.into()
                    }
                }
                None => dexterity_ability_score_bonus.into(),
            };

        let armor_bonus = armor
            .filter(|x| x.strength_requirement <= self.ability_scores().strength)
            .map(|armor| armor.armor_bonus)
            .unwrap_or(ArmorPoints::new(0));
        let main_hand_bonus = self
            .equipped_weapon
            .map(|weapon| weapon.to_weapon().armor_bonus)
            .unwrap_or(ArmorPoints::new(0));
        let offhand_bonus = self
            .equipped_offhand
            .map(|weapon| weapon.to_weapon().armor_bonus)
            .unwrap_or(ArmorPoints::new(0));

        let jewelry_bonus = ArmorPoints::new(
            self.equipped_jewelry
                .iter()
                .map(|x| x.to_jewelry().armor_bonus)
                .map(|x| *x)
                .sum(),
        );

        base_armor + armor_bonus + dexterity_bonus + main_hand_bonus + offhand_bonus + jewelry_bonus
    }

    pub fn attacked(&mut self, attack_stats: &AttackStats) -> Option<Hit> {
        let dice_roll = roll(&Dice::D20);
        let hit = dice_roll + *attack_stats.hit_bonus;
        let critical_hit = dice_roll == 20;
        let critical_miss = dice_roll == 1;

        if critical_miss {
            return None;
        }

        if hit > *self.armor_points() || critical_hit {
            let mut damage = match critical_hit {
                true => attack(attack_stats) + attack(attack_stats),
                false => attack(attack_stats),
            };

            if *damage < 0 {
                damage = HealthPoints::new(0);
            }

            self.current_health -= damage;

            let outcome = Hit {
                damage,
                critical_hit,
                character_name: self.name.clone(),
            };

            return Some(outcome);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Dice, Status,
        abilities::AbilityScores,
        armor::ArmorType,
        character::{
            AvailableAction, AvailableActionDefinition, AvailableBonusActionDefinition, Character,
            CharacterType, CharacterWeapon,
        },
        items::ItemType,
        spells::SpellType,
        types::{
            AbilityScoreBonus, ArmorPoints, Constitution, Dexterity, Experience, Gold,
            HealthPoints, QuestPoint, Strength,
        },
        weapons::WeaponType,
    };

    #[test]
    fn correct_conditions() {
        let strength = Strength::new(8);
        let character = Character {
            id: 1,
            name: "Testington".to_string(),
            character_type: CharacterType::Player,
            current_health: HealthPoints::new(10),
            base_ability_scores: AbilityScores {
                strength,
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
            item_inventory: vec![],
            level_up_choices: vec![],
            status: Status::Questing,
            party: 1,
            quest_points: QuestPoint::new(0),
            short_rests_available: 2,
            active_conditions: vec![SpellType::Weaken.active_condition().unwrap()],
        };

        assert_eq!(
            character.ability_scores().strength,
            strength
                + SpellType::Weaken
                    .active_condition()
                    .unwrap()
                    .condition_type
                    .to_condition()
                    .strength_bonus
                    .unwrap()
        )
    }

    #[test]
    fn correct_available_actions() {
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

        let mut actual_actions = character.available_actions();
        actual_actions.sort_by(|a, b| b.name.cmp(&a.name));

        assert_eq!(
            actual_actions,
            vec![
                AvailableActionDefinition {
                    name: "attack".to_string(),
                    action: AvailableAction::Attack,
                    requires_target: true
                },
                AvailableActionDefinition {
                    name: scroll_of_weaken.to_item().name,
                    action: AvailableAction::Item(scroll_of_weaken),
                    requires_target: true
                },
            ]
        );

        let mut actual_bonus_actions = character.available_bonus_actions();
        actual_bonus_actions.sort_by(|a, b| b.name.cmp(&a.name));

        assert_eq!(
            actual_bonus_actions,
            vec![
                AvailableBonusActionDefinition {
                    name: "attack".to_string(),
                    action: AvailableAction::Attack,
                    requires_target: true
                },
                AvailableBonusActionDefinition {
                    name: healing_potion.to_item().name,
                    action: AvailableAction::Item(healing_potion),
                    requires_target: false
                },
            ]
        )
    }

    #[test]
    fn correct_armor_bonus_without_armor() {
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
            item_inventory: vec![],
            level_up_choices: vec![],
            status: Status::Questing,
            party: 1,
            quest_points: QuestPoint::new(0),
            short_rests_available: 2,
            active_conditions: vec![],
        };

        assert_eq!(
            character.ability_scores().dexterity.ability_score_bonus(),
            AbilityScoreBonus::new(-1)
        );
        assert_eq!(
            character.armor_points(),
            ArmorPoints::new(10)
                + ArmorPoints::new(*character.ability_scores().dexterity.ability_score_bonus())
        );
        assert_eq!(character.armor_points(), ArmorPoints::new(9));
    }

    #[test]
    fn correct_armor_bonus_with_armor() {
        let character = Character {
            id: 1,
            name: "Testington".to_string(),
            character_type: CharacterType::Player,
            current_health: HealthPoints::new(10),
            base_ability_scores: AbilityScores {
                strength: Strength::new(8),
                dexterity: Dexterity::new(12),
                constitution: Constitution::new(8),
            },
            gold: Gold::new(0),
            experience: Experience::new(0),
            equipped_weapon: None,
            equipped_offhand: Some(WeaponType::Shield),
            equipped_armor: Some(ArmorType::Leather),
            equipped_jewelry: vec![],
            weapon_inventory: vec![],
            armor_inventory: vec![],
            jewelry_inventory: vec![],
            item_inventory: vec![],
            level_up_choices: vec![],
            status: Status::Questing,
            party: 1,
            quest_points: QuestPoint::new(0),
            short_rests_available: 2,
            active_conditions: vec![],
        };

        assert_eq!(
            character.ability_scores().dexterity.ability_score_bonus(),
            AbilityScoreBonus::new(1)
        );
        assert_eq!(
            character.armor_points(),
            ArmorPoints::new(10)
                + ArmorType::Leather.to_armor().armor_bonus
                + ArmorPoints::new(*character.ability_scores().dexterity.ability_score_bonus())
                + WeaponType::Shield.to_weapon().armor_bonus
        );
        assert_eq!(character.armor_points(), ArmorPoints::new(15));
    }

    #[test]
    fn correct_armor_bonus_with_armor_with_insufficient_strength() {
        let character = Character {
            id: 1,
            name: "Testington".to_string(),
            character_type: CharacterType::Player,
            current_health: HealthPoints::new(10),
            base_ability_scores: AbilityScores {
                strength: Strength::new(8),
                dexterity: Dexterity::new(12),
                constitution: Constitution::new(8),
            },
            gold: Gold::new(0),
            experience: Experience::new(0),
            equipped_weapon: None,
            equipped_offhand: None,
            equipped_armor: Some(ArmorType::Splint),
            equipped_jewelry: vec![],
            weapon_inventory: vec![],
            armor_inventory: vec![],
            jewelry_inventory: vec![],
            item_inventory: vec![],
            level_up_choices: vec![],
            status: Status::Questing,
            party: 1,
            quest_points: QuestPoint::new(0),
            short_rests_available: 2,
            active_conditions: vec![],
        };

        assert_eq!(character.armor_points(), ArmorPoints::new(10));
    }

    #[test]
    fn correct_armor_bonus_with_medium_armor_with_insufficient_strength() {
        let character = Character {
            id: 1,
            name: "Testington".to_string(),
            character_type: CharacterType::Player,
            current_health: HealthPoints::new(10),
            base_ability_scores: AbilityScores {
                strength: Strength::new(8),
                dexterity: Dexterity::new(12),
                constitution: Constitution::new(8),
            },
            gold: Gold::new(0),
            experience: Experience::new(0),
            equipped_weapon: None,
            equipped_offhand: None,
            equipped_armor: Some(ArmorType::BreastPlate),
            equipped_jewelry: vec![],
            weapon_inventory: vec![],
            armor_inventory: vec![],
            jewelry_inventory: vec![],
            item_inventory: vec![],
            level_up_choices: vec![],
            status: Status::Questing,
            party: 1,
            quest_points: QuestPoint::new(0),
            short_rests_available: 2,
            active_conditions: vec![],
        };

        assert_eq!(character.armor_points(), ArmorPoints::new(11));
    }

    #[test]
    fn correct_attack_dice_with_versatile_weapon() {
        let character = Character {
            id: 1,
            name: "Testington".to_string(),
            character_type: CharacterType::Player,
            current_health: HealthPoints::new(10),
            base_ability_scores: AbilityScores {
                strength: Strength::new(14),
                dexterity: Dexterity::new(12),
                constitution: Constitution::new(8),
            },
            gold: Gold::new(0),
            experience: Experience::new(0),
            equipped_weapon: Some(WeaponType::Longsword),
            equipped_offhand: None,
            equipped_armor: None,
            equipped_jewelry: vec![],
            weapon_inventory: vec![],
            armor_inventory: vec![],
            jewelry_inventory: vec![],
            item_inventory: vec![],
            level_up_choices: vec![],
            status: Status::Questing,
            party: 1,
            quest_points: QuestPoint::new(0),
            short_rests_available: 2,
            active_conditions: vec![],
        };

        assert_eq!(
            character
                .attack_stats(CharacterWeapon::Mainhand)
                .attack_dice,
            WeaponType::Longsword
                .to_weapon()
                .attack_stats
                .versatile_attack_dices
                .unwrap()
        );
    }

    #[test]
    fn correct_attack_dice_with_versatile_weapon_with_equipped() {
        let character = Character {
            id: 1,
            name: "Testington".to_string(),
            character_type: CharacterType::Player,
            current_health: HealthPoints::new(10),
            base_ability_scores: AbilityScores {
                strength: Strength::new(14),
                dexterity: Dexterity::new(12),
                constitution: Constitution::new(8),
            },
            gold: Gold::new(0),
            experience: Experience::new(0),
            equipped_weapon: Some(WeaponType::Longsword),
            equipped_offhand: Some(WeaponType::Dagger),
            equipped_armor: None,
            equipped_jewelry: vec![],
            weapon_inventory: vec![],
            armor_inventory: vec![],
            jewelry_inventory: vec![],
            item_inventory: vec![],
            level_up_choices: vec![],
            status: Status::Questing,
            party: 1,
            quest_points: QuestPoint::new(0),
            short_rests_available: 2,
            active_conditions: vec![],
        };

        assert_eq!(
            character
                .attack_stats(CharacterWeapon::Mainhand)
                .attack_dice,
            WeaponType::Longsword.to_weapon().attack_stats.attack_dices
        );
    }

    #[test]
    fn correct_attack_dice_when_insufficient_strength() {
        let character = Character {
            id: 1,
            name: "Testington".to_string(),
            character_type: CharacterType::Player,
            current_health: HealthPoints::new(10),
            base_ability_scores: AbilityScores {
                strength: Strength::new(8),
                dexterity: Dexterity::new(12),
                constitution: Constitution::new(8),
            },
            gold: Gold::new(0),
            experience: Experience::new(0),
            equipped_weapon: Some(WeaponType::Longsword),
            equipped_offhand: Some(WeaponType::Dagger),
            equipped_armor: None,
            equipped_jewelry: vec![],
            weapon_inventory: vec![],
            armor_inventory: vec![],
            jewelry_inventory: vec![],
            item_inventory: vec![],
            level_up_choices: vec![],
            status: Status::Questing,
            party: 1,
            quest_points: QuestPoint::new(0),
            short_rests_available: 2,
            active_conditions: vec![],
        };

        assert_eq!(
            character
                .attack_stats(CharacterWeapon::Mainhand)
                .attack_dice,
            vec![Dice::D4]
        );
    }
}

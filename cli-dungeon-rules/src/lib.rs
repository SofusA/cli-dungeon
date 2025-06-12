use abilities::{AbilityScaling, AbilityScores, AbilityType};
use armor::ArmorType;
use classes::LevelUpChoice;
use conditions::ActiveCondition;
use items::ItemType;
use jewelry::JewelryType;
use types::{
    AbilityScoreBonus, ArmorPoints, Constitution, Dexterity, Experience, Gold, HealthPoints, Level,
    QuestPoint, Strength,
};
use weapons::WeaponType;

pub mod abilities;
pub mod armor;
pub mod classes;
pub mod conditions;
pub mod items;
pub mod jewelry;
pub mod monsters;
pub mod spells;
pub mod types;
pub mod weapons;

pub fn roll(dice: &Dice) -> i16 {
    let max = match dice {
        Dice::D4 => 4,
        Dice::D6 => 6,
        Dice::D8 => 8,
        Dice::D10 => 10,
        Dice::D20 => 20,
    };

    rand::random_range(1..=max)
}

pub fn roll_success(dice: &Dice) -> bool {
    let max = match dice {
        Dice::D4 => 4,
        Dice::D6 => 6,
        Dice::D8 => 8,
        Dice::D10 => 10,
        Dice::D20 => 20,
    };

    rand::random_range(1..=max) == max
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Dice {
    D4,
    D6,
    D8,
    D10,
    D20,
}

pub struct AttackStats {
    pub attack_dice: Vec<Dice>,
    pub attack_bonus: AbilityScoreBonus,
    pub hit_bonus: AbilityScoreBonus,
}

#[derive(Default, Clone)]
pub enum Status {
    #[default]
    Resting,
    Questing,
    Fighting(i64),
}

pub struct Encounter {
    pub id: i64,
    pub rotation: Vec<Character>,
    pub dead_characters: Vec<Character>,
}

#[derive(Clone)]
pub struct Character {
    pub id: i64,
    pub name: String,
    pub player: bool,
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

pub fn max_health(constitution: &Constitution, level: Level) -> HealthPoints {
    let health = 12 + 6 * *level as i16 + *constitution.ability_score_bonus();
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
        1 => Experience::new(100),
        2 => Experience::new(150),
        3 => Experience::new(250),
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

impl Character {
    pub fn ability_scores(&self) -> AbilityScores {
        let base_ability_scores = &self.base_ability_scores;

        let strength_level_bonus = Strength::new(
            self.level_up_choices
                .iter()
                .filter(|choice| choice.ability_increment == AbilityType::Strength)
                .count() as i16,
        );

        let dexterity_level_bonus = Dexterity::new(
            self.level_up_choices
                .iter()
                .filter(|choice| choice.ability_increment == AbilityType::Dexterity)
                .count() as i16,
        );

        let constitution_level_bonus = Constitution::new(
            self.level_up_choices
                .iter()
                .filter(|choice| choice.ability_increment == AbilityType::Constitution)
                .count() as i16,
        );

        AbilityScores {
            strength: base_ability_scores.strength + strength_level_bonus,
            dexterity: base_ability_scores.dexterity + dexterity_level_bonus,
            constitution: base_ability_scores.constitution + constitution_level_bonus,
        }
    }

    pub fn experience_level(&self) -> Level {
        let thresholds = [
            30, 300, 900,
            // 2700, 6500, 14000, 23000, 34000, 48000, 64000, 85000, 100000,
        ];

        for (level, &threshold) in thresholds.iter().enumerate() {
            if *self.experience < threshold {
                return Level::new(level as u16);
            }
        }

        Level::new(thresholds.len() as u16)
    }

    pub fn level(&self) -> Level {
        Level::new(self.level_up_choices.len() as u16)
    }

    pub fn max_health(&self) -> HealthPoints {
        max_health(&self.ability_scores().constitution, self.level())
    }

    pub fn attack_stats(&self) -> AttackStats {
        let dex = &self.ability_scores().dexterity;
        let str = &self.ability_scores().strength;

        let Some(weapon) = &self.equipped_weapon else {
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

        let ability_bonus = match weapon.to_weapon().attack_stats.primary_ability {
            AbilityScaling::Strength => str.ability_score_bonus(),
            AbilityScaling::Dexterity => dex.ability_score_bonus(),
            AbilityScaling::Either => match **dex < **str {
                true => str.ability_score_bonus(),
                false => dex.ability_score_bonus(),
            },
        };

        let attack_dice = weapon.to_weapon().attack_stats.attack_dices;

        AttackStats {
            attack_dice,
            attack_bonus: ability_bonus + weapon.to_weapon().attack_stats.attack_bonus,
            hit_bonus: ability_bonus,
        }
    }

    pub fn off_hand_attack_stats(&self) -> AttackStats {
        let dex = &self.ability_scores().dexterity;
        let str = &self.ability_scores().strength;

        let Some(weapon) = &self.equipped_offhand else {
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

        let ability_bonus = match weapon.to_weapon().attack_stats.primary_ability {
            AbilityScaling::Strength => str.ability_score_bonus(),
            AbilityScaling::Dexterity => dex.ability_score_bonus(),
            AbilityScaling::Either => match **dex < **str {
                true => str.ability_score_bonus(),
                false => dex.ability_score_bonus(),
            },
        };

        let attack_dice = weapon.to_weapon().attack_stats.attack_dices;

        AttackStats {
            attack_dice,
            attack_bonus: ability_bonus + weapon.to_weapon().attack_stats.attack_bonus,
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
            .map(|armor| armor.armor_bonus)
            .unwrap_or(ArmorPoints::new(10));
        let main_hand_bonus = self
            .equipped_weapon
            .as_ref()
            .map(|weapon| weapon.to_weapon().armor_bonus)
            .unwrap_or(ArmorPoints::new(0));
        let off_hand_bonus = self
            .equipped_offhand
            .as_ref()
            .map(|weapon| weapon.to_weapon().armor_bonus)
            .unwrap_or(ArmorPoints::new(0));

        let jewelry_bonus = ArmorPoints::new(
            self.equipped_jewelry
                .iter()
                .map(|x| x.to_jewelry().armor_bonus)
                .map(|x| *x)
                .sum(),
        );

        base_armor
            + armor_bonus
            + dexterity_bonus
            + main_hand_bonus
            + off_hand_bonus
            + jewelry_bonus
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

#[derive(Debug, Clone)]
pub struct Hit {
    pub damage: HealthPoints,
    pub critical_hit: bool,
    pub character_name: String,
}

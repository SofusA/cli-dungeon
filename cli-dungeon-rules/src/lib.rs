use abilities::{
    AbilityScaling, AbilityScore, AbilityScoreBonus, AbilityScores, AbilityType, Constitution,
    Dexterity, Strength,
};
use armor::{ArmorPoints, ArmorType};
use classes::LevelUpChoice;
use types::{Experience, Gold, HealthPoints, Level};
use weapons::WeaponType;

pub mod abilities;
pub mod armor;
pub mod classes;
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
    pub weapon_inventory: Vec<WeaponType>,
    pub armor_inventory: Vec<ArmorType>,
    pub level_up_choices: Vec<LevelUpChoice>,
}

pub fn max_health(constitution: &Constitution, level: Level) -> HealthPoints {
    let health = 12 + 6 * *level as i16 + *constitution.ability_score_bonus();
    HealthPoints(health)
}

fn attack(attack_stats: &AttackStats) -> HealthPoints {
    let damage =
        attack_stats.attack_dice.iter().map(roll).sum::<i16>() + *attack_stats.attack_bonus;
    HealthPoints(damage)
}

pub fn experience_gain(levels: Vec<LevelUpChoice>) -> Experience {
    let level = levels.len();

    match level {
        0 => Experience(0),
        1 => Experience(100),
        2 => Experience(150),
        3 => Experience(250),
        4 => Experience(300),
        5 => Experience(500),
        6 => Experience(750),
        7 => Experience(1000),
        8 => Experience(1500),
        9 => Experience(2000),
        10 => Experience(2750),
        11 => Experience(3500),
        12 => Experience(5000),
        _ => Experience(0),
    }
}

impl Character {
    pub fn ability_scores(&self) -> AbilityScores {
        let base_ability_scores = &self.base_ability_scores;

        let strength_level_bonus = Strength(AbilityScore(
            self.level_up_choices
                .iter()
                .filter(|choice| choice.ability_increment == AbilityType::Strength)
                .count() as u16,
        ));

        let dexterity_level_bonus = Dexterity(AbilityScore(
            self.level_up_choices
                .iter()
                .filter(|choice| choice.ability_increment == AbilityType::Dexterity)
                .count() as u16,
        ));

        let constitution_level_bonus = Constitution(AbilityScore(
            self.level_up_choices
                .iter()
                .filter(|choice| choice.ability_increment == AbilityType::Constitution)
                .count() as u16,
        ));

        AbilityScores {
            strength: base_ability_scores.strength + strength_level_bonus,
            dexterity: base_ability_scores.dexterity + dexterity_level_bonus,
            constitution: base_ability_scores.constitution + constitution_level_bonus,
        }
    }

    pub fn experience_level(&self) -> Level {
        let thresholds = [
            0, 300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000, 85000, 100000,
        ];

        for (level, &threshold) in thresholds.iter().enumerate() {
            if *self.experience < threshold {
                return Level(level as u16);
            }
        }

        Level(thresholds.len() as u16)
    }

    pub fn level(&self) -> Level {
        Level(self.level_up_choices.len() as u16)
    }

    pub fn max_health(&self) -> HealthPoints {
        max_health(&self.ability_scores().constitution, self.level())
    }

    pub fn attack_stats(&self) -> AttackStats {
        let dex = &self.ability_scores().dexterity;
        let str = &self.ability_scores().strength;

        let Some(weapon) = &self.equipped_weapon else {
            let ability_bonus = match dex.0.0 < str.0.0 {
                true => str.ability_score_bonus(),
                false => dex.ability_score_bonus(),
            };
            return AttackStats {
                attack_dice: vec![Dice::D4],
                attack_bonus: ability_bonus,
                hit_bonus: ability_bonus,
            };
        };

        let mut offhand_attack_dice = self
            .equipped_offhand
            .as_ref()
            .map(|offhand| offhand.to_weapon().attack_dices)
            .unwrap_or_default();

        let offhand_attack_bonus = &self
            .equipped_offhand
            .as_ref()
            .map(|offhand| offhand.to_weapon().attack_bonus)
            .unwrap_or(AbilityScoreBonus(0));

        let ability_bonus = match weapon.to_weapon().primary_ability {
            AbilityScaling::Strength => str.ability_score_bonus(),
            AbilityScaling::Dexterity => dex.ability_score_bonus(),
            AbilityScaling::Either => match **dex < **str {
                true => str.ability_score_bonus(),
                false => dex.ability_score_bonus(),
            },
        };

        let mut attack_dice = weapon.to_weapon().attack_dices;
        attack_dice.append(&mut offhand_attack_dice);

        AttackStats {
            attack_dice,
            attack_bonus: ability_bonus + weapon.to_weapon().attack_bonus + *offhand_attack_bonus,
            hit_bonus: ability_bonus,
        }
    }

    pub fn is_alive(&self) -> bool {
        *self.current_health > 0
    }

    pub fn armor_points(&self) -> ArmorPoints {
        let armor = self.equipped_armor.as_ref().map(|armor| armor.to_armor());
        let base_armor = ArmorPoints(10);
        let dexterity_ability_score_bonus = self.ability_scores().dexterity.ability_score_bonus();

        let dexterity_bonus: ArmorPoints = match armor.map(|armor| armor.max_dexterity_bonus) {
            Some(max_bonus) => {
                if *dexterity_ability_score_bonus > *max_bonus {
                    max_bonus.into()
                } else {
                    dexterity_ability_score_bonus.into()
                }
            }
            None => dexterity_ability_score_bonus.into(),
        };

        let armor_bonus = self
            .equipped_armor
            .as_ref()
            .map(|armor| armor.to_armor().armor_bonus)
            .unwrap_or(ArmorPoints(0));
        let main_hand_bonus = self
            .equipped_weapon
            .as_ref()
            .map(|weapon| weapon.to_weapon().armor_bonus)
            .unwrap_or(ArmorPoints(0));
        let off_hand_bonus = self
            .equipped_offhand
            .as_ref()
            .map(|weapon| weapon.to_weapon().armor_bonus)
            .unwrap_or(ArmorPoints(0));

        base_armor + armor_bonus + dexterity_bonus + main_hand_bonus + off_hand_bonus
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
            let damage = match critical_hit {
                true => attack(attack_stats) + attack(attack_stats),
                false => attack(attack_stats),
            };

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

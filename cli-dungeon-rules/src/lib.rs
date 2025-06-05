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
    pub attack_bonus: i16,
}

pub struct Character {
    pub id: i64,
    pub name: String,
    pub current_health: i16,
    pub base_ability_scores: AbilityScores,
    pub equipped_weapon: Option<WeaponType>,
}

pub struct Weapon {
    pub name: String,
    pub primary_ability: Ability,
    pub hit_bonus: i16,
    pub attack_dices: Vec<Dice>,
    pub attack_bonus: i16,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum WeaponType {
    Shortsword,
    Longsword,
}

impl WeaponType {
    fn to_weapon(&self) -> Weapon {
        match self {
            WeaponType::Shortsword => Weapon {
                name: "Shortsword".to_string(),
                primary_ability: Ability::Dexterity,
                hit_bonus: 0,
                attack_dices: vec![Dice::D6],
                attack_bonus: 1,
            },
            WeaponType::Longsword => Weapon {
                name: "Longsword".to_string(),
                primary_ability: Ability::Strength,
                hit_bonus: 0,
                attack_dices: vec![Dice::D8],
                attack_bonus: 0,
            },
        }
    }
}

pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct AbilityScore(i16);
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Strength(AbilityScore);
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Dexterity(AbilityScore);
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Constitution(AbilityScore);

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AbilityScores {
    pub strength: Strength,
    pub dexterity: Dexterity,
    pub constitution: Constitution,
}

impl AbilityScores {
    pub fn new(strength: i16, dexterity: i16, constitution: i16) -> Self {
        Self {
            strength: Strength(AbilityScore(strength)),
            dexterity: Dexterity(AbilityScore(dexterity)),
            constitution: Constitution(AbilityScore(constitution)),
        }
    }
}

fn ability_score_bonus(ability_score: &AbilityScore) -> i16 {
    (ability_score.0 - 10) / 2
}

fn base_amour(dexterity: &Dexterity) -> i16 {
    10 + ability_score_bonus(&dexterity.0)
}

pub fn max_health(constitution: &Constitution, level: i16) -> i16 {
    12 + 6 * level + ability_score_bonus(&constitution.0)
}

fn attack(attack_stats: &AttackStats) -> i16 {
    attack_stats.attack_dice.iter().map(roll).sum::<i16>() + attack_stats.attack_bonus
}

impl Character {
    pub fn new(id: i64, name: &str, ability_scores: AbilityScores) -> Self {
        Self {
            id,
            name: name.to_owned(),
            current_health: max_health(&ability_scores.constitution, 0),
            base_ability_scores: ability_scores,
            equipped_weapon: None,
        }
    }

    pub fn ability_scores(&self) -> &AbilityScores {
        &self.base_ability_scores
    }

    pub fn hit_bonus(&self) -> i16 {
        let dex = &self.ability_scores().dexterity;
        let str = &self.ability_scores().strength;

        match dex.0.0 < str.0.0 {
            true => ability_score_bonus(&str.0),
            false => ability_score_bonus(&dex.0),
        }
    }

    // pub fn equip_weapon(&mut self, weapon: WeaponType) {
    //     self.equipped_weapon = Some(weapon);
    // }

    pub fn attack_stats(&self) -> AttackStats {
        let dex = &self.ability_scores().dexterity;
        let str = &self.ability_scores().strength;

        let Some(weapon) = &self.equipped_weapon else {
            let attack_bonus = match dex.0.0 < str.0.0 {
                true => ability_score_bonus(&str.0),
                false => ability_score_bonus(&dex.0),
            };
            return AttackStats {
                attack_dice: vec![Dice::D4],
                attack_bonus,
            };
        };

        AttackStats {
            attack_dice: weapon.to_weapon().attack_dices,
            attack_bonus: weapon.to_weapon().attack_bonus,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current_health > 0
    }

    pub fn armor_points(&self) -> i16 {
        base_amour(&self.base_ability_scores.dexterity)
    }

    pub fn attacked(&mut self, hit_bonus: &i16, attack_stats: &AttackStats) -> Option<Hit> {
        let dice_roll = roll(&Dice::D20);
        let hit = dice_roll + hit_bonus;
        let critical_hit = dice_roll == 20;
        let critical_miss = dice_roll == 1;

        if critical_miss {
            return None;
        }

        if hit > self.armor_points() || critical_hit {
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
    pub damage: i16,
    pub critical_hit: bool,
    pub character_name: String,
}

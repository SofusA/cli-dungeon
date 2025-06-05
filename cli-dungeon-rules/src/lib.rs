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
    pub player: bool,
    pub current_health: u16,
    pub base_ability_scores: AbilityScores,
    pub gold: u16,
    pub experience: u16,
    pub equipped_weapon: Option<WeaponType>,
    pub equipped_offhand: Option<WeaponType>,
    pub equipped_armor: Option<ArmorType>,
    pub weapon_inventory: Vec<WeaponType>,
    pub armor_inventory: Vec<ArmorType>,
}

pub struct Weapon {
    pub name: String,
    pub cost: u16,
    pub primary_ability: AbilityScaling,
    pub hit_bonus: u16,
    pub attack_dices: Vec<Dice>,
    pub attack_bonus: i16,
    pub allow_offhand: bool,
    pub armor_bonus: u16,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum WeaponType {
    Dagger,
    Shortsword,
    Longsword,
    Shield,
}

impl WeaponType {
    fn to_weapon(&self) -> Weapon {
        match self {
            WeaponType::Dagger => Weapon {
                name: "Dagger".to_string(),
                cost: 5,
                primary_ability: AbilityScaling::Dexterity,
                hit_bonus: 0,
                attack_dices: vec![Dice::D4],
                attack_bonus: 0,
                allow_offhand: true,
                armor_bonus: 0,
            },
            WeaponType::Shortsword => Weapon {
                name: "Shortsword".to_string(),
                cost: 50,
                primary_ability: AbilityScaling::Either,
                hit_bonus: 0,
                attack_dices: vec![Dice::D6],
                attack_bonus: 0,
                allow_offhand: true,
                armor_bonus: 0,
            },
            WeaponType::Longsword => Weapon {
                name: "Longsword".to_string(),
                cost: 50,
                primary_ability: AbilityScaling::Strength,
                hit_bonus: 0,
                attack_dices: vec![Dice::D8],
                attack_bonus: 0,
                allow_offhand: false,
                armor_bonus: 0,
            },
            WeaponType::Shield => Weapon {
                name: "Shield".to_string(),
                cost: 30,
                primary_ability: AbilityScaling::Strength,
                hit_bonus: 0,
                attack_dices: vec![],
                attack_bonus: 0,
                allow_offhand: true,
                armor_bonus: 2,
            },
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum ArmorType {
    Leather,
    ChainMail,
    Splint,
}

pub struct Armor {
    pub name: String,
    pub cost: u16,
    pub armor_bonus: i16,
    pub max_dexterity_bonus: u16,
    pub strength_requirement: Strength,
}

impl ArmorType {
    fn to_armor(&self) -> Armor {
        match self {
            ArmorType::Leather => Armor {
                name: "Leather".to_string(),
                cost: 30,
                armor_bonus: 1,
                max_dexterity_bonus: 6,
                strength_requirement: Strength(AbilityScore(8)),
            },
            ArmorType::ChainMail => Armor {
                name: "Chain mail".to_string(),
                cost: 150,
                armor_bonus: 4,
                max_dexterity_bonus: 4,
                strength_requirement: Strength(AbilityScore(14)),
            },
            ArmorType::Splint => Armor {
                name: "Split armor".to_string(),
                cost: 200,
                armor_bonus: 7,
                max_dexterity_bonus: 0,
                strength_requirement: Strength(AbilityScore(16)),
            },
        }
    }
}

pub enum AbilityScaling {
    Strength,
    Dexterity,
    Either,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct AbilityScore(u16);
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
    pub fn new(strength: u16, dexterity: u16, constitution: u16) -> Self {
        Self {
            strength: Strength(AbilityScore(strength)),
            dexterity: Dexterity(AbilityScore(dexterity)),
            constitution: Constitution(AbilityScore(constitution)),
        }
    }
}

fn ability_score_bonus(ability_score: &AbilityScore) -> i16 {
    let bonus = (ability_score.0 - 10) / 2;
    bonus as i16
}

pub fn max_health(constitution: &Constitution, level: u16) -> u16 {
    let health = 12 + 6 * level as i16 + ability_score_bonus(&constitution.0);
    health as u16
}

fn attack(attack_stats: &AttackStats) -> u16 {
    let damage = attack_stats.attack_dice.iter().map(roll).sum::<i16>() + attack_stats.attack_bonus;
    damage as u16
}

impl Character {
    pub fn new_player(id: i64, name: &str, ability_scores: AbilityScores) -> Self {
        Self {
            id,
            name: name.to_owned(),
            player: true,
            current_health: max_health(&ability_scores.constitution, 0),
            base_ability_scores: ability_scores,
            equipped_weapon: None,
            equipped_offhand: None,
            equipped_armor: None,
            experience: 0,
            gold: 150,
            weapon_inventory: vec![],
            armor_inventory: vec![],
        }
    }

    pub fn new(
        id: i64,
        name: &str,
        ability_scores: AbilityScores,
        gold: u16,
        weapon_inventory: Vec<WeaponType>,
        armor_inventory: Vec<ArmorType>,
    ) -> Self {
        Self {
            id,
            name: name.to_owned(),
            player: false,
            current_health: max_health(&ability_scores.constitution, 0),
            base_ability_scores: ability_scores,
            equipped_weapon: None,
            equipped_offhand: None,
            equipped_armor: None,
            experience: 0,
            gold,
            weapon_inventory,
            armor_inventory,
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

    pub fn attack_stats(&self) -> AttackStats {
        let dex = &self.ability_scores().dexterity;
        let str = &self.ability_scores().strength;
        let attack_bonus = match dex.0.0 < str.0.0 {
            true => ability_score_bonus(&str.0),
            false => ability_score_bonus(&dex.0),
        };

        let Some(weapon) = &self.equipped_weapon else {
            return AttackStats {
                attack_dice: vec![Dice::D4],
                attack_bonus,
            };
        };

        AttackStats {
            attack_dice: weapon.to_weapon().attack_dices,
            attack_bonus: attack_bonus + weapon.to_weapon().attack_bonus,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current_health > 0
    }

    pub fn armor_points(&self) -> u16 {
        let armor_bonus = self
            .equipped_armor
            .as_ref()
            .map(|armor| armor.to_armor().armor_bonus)
            .unwrap_or(0);
        let main_hand_bonus = self
            .equipped_weapon
            .as_ref()
            .map(|weapon| weapon.to_weapon().armor_bonus)
            .unwrap_or(0);
        let off_hand_bonus = self
            .equipped_weapon
            .as_ref()
            .map(|weapon| weapon.to_weapon().armor_bonus)
            .unwrap_or(0);

        let base_armor = 10 + ability_score_bonus(&self.ability_scores().dexterity.0);

        let armor = base_armor + armor_bonus + main_hand_bonus as i16 + off_hand_bonus as i16;
        armor as u16
    }

    pub fn attacked(&mut self, hit_bonus: &i16, attack_stats: &AttackStats) -> Option<Hit> {
        let dice_roll = roll(&Dice::D20);
        let hit = dice_roll + hit_bonus;
        let critical_hit = dice_roll == 20;
        let critical_miss = dice_roll == 1;

        if critical_miss {
            return None;
        }

        if hit > self.armor_points() as i16 || critical_hit {
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
    pub damage: u16,
    pub critical_hit: bool,
    pub character_name: String,
}

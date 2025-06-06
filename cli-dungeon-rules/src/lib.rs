use derive_more::Display;

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

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum ClassType {
    Monster,
    Fighter,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct LevelUpChoice {
    pub ability_increment: AbilityType,
    pub class: ClassType,
}

impl ClassType {
    pub fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn from_class_str(string: &str) -> Option<Self> {
        let string = string.to_lowercase();
        match string.as_str() {
            "fighter" => Some(Self::Fighter),
            _ => None,
        }
    }
}

fn experience_level(experience: Experience) -> Level {
    let thresholds = [
        0, 300, 900, 2700, 6500, 14000, 23000, 34000, 48000, 64000, 85000, 100000,
    ];

    for (level, &threshold) in thresholds.iter().enumerate() {
        if *experience < threshold {
            return Level(level as u16);
        }
    }

    Level(thresholds.len() as u16)
}

pub struct Character {
    pub id: i64,
    pub name: String,
    pub player: bool,
    pub current_health: Health,
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

pub struct Weapon {
    pub name: String,
    pub cost: Gold,
    pub primary_ability: AbilityScaling,
    pub hit_bonus: AbilityScoreBonus,
    pub attack_dices: Vec<Dice>,
    pub attack_bonus: AbilityScoreBonus,
    pub allow_offhand: bool,
    pub armor_bonus: ArmorPoints,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum WeaponType {
    Dagger,
    Shortsword,
    Longsword,
    Shield,
}

impl WeaponType {
    fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn from_weapon_str(string: &str) -> Option<Self> {
        let string = string.to_lowercase();
        match string.as_str() {
            "dagger" => Some(Self::Dagger),
            "shortsword" => Some(Self::Shortsword),
            "longsword" => Some(Self::Longsword),
            "shield" => Some(Self::Shield),
            _ => None,
        }
    }

    pub fn to_weapon(&self) -> Weapon {
        match self {
            WeaponType::Dagger => Weapon {
                name: self.to_name(),
                cost: Gold(5),
                primary_ability: AbilityScaling::Dexterity,
                hit_bonus: AbilityScoreBonus(0),
                attack_dices: vec![Dice::D4],
                attack_bonus: AbilityScoreBonus(0),
                allow_offhand: true,
                armor_bonus: ArmorPoints(0),
            },
            WeaponType::Shortsword => Weapon {
                name: self.to_name(),
                cost: Gold(50),
                primary_ability: AbilityScaling::Either,
                hit_bonus: AbilityScoreBonus(0),
                attack_dices: vec![Dice::D6],
                attack_bonus: AbilityScoreBonus(0),
                allow_offhand: true,
                armor_bonus: ArmorPoints(0),
            },
            WeaponType::Longsword => Weapon {
                name: self.to_name(),
                cost: Gold(50),
                primary_ability: AbilityScaling::Strength,
                hit_bonus: AbilityScoreBonus(0),
                attack_dices: vec![Dice::D8],
                attack_bonus: AbilityScoreBonus(0),
                allow_offhand: false,
                armor_bonus: ArmorPoints(0),
            },
            WeaponType::Shield => Weapon {
                name: self.to_name(),
                cost: Gold(30),
                primary_ability: AbilityScaling::Strength,
                hit_bonus: AbilityScoreBonus(0),
                attack_dices: vec![],
                attack_bonus: AbilityScoreBonus(0),
                allow_offhand: true,
                armor_bonus: ArmorPoints(2),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum ArmorType {
    Leather,
    Chainmail,
    Splint,
}

pub struct Armor {
    pub name: String,
    pub cost: Gold,
    pub armor_bonus: ArmorPoints,
    pub max_dexterity_bonus: AbilityScoreBonus,
    pub strength_requirement: Strength,
}

impl ArmorType {
    fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn from_armor_str(string: &str) -> Option<Self> {
        let string = string.to_lowercase();
        match string.as_str() {
            "leather" => Some(Self::Leather),
            "chainmail" => Some(Self::Chainmail),
            "splint" => Some(Self::Splint),
            _ => None,
        }
    }

    pub fn to_armor(&self) -> Armor {
        match self {
            ArmorType::Leather => Armor {
                name: self.to_name(),
                cost: Gold(30),
                armor_bonus: ArmorPoints(1),
                max_dexterity_bonus: AbilityScoreBonus(6),
                strength_requirement: Strength(AbilityScore(8)),
            },
            ArmorType::Chainmail => Armor {
                name: self.to_name(),
                cost: Gold(150),
                armor_bonus: ArmorPoints(4),
                max_dexterity_bonus: AbilityScoreBonus(4),
                strength_requirement: Strength(AbilityScore(14)),
            },
            ArmorType::Splint => Armor {
                name: self.to_name(),
                cost: Gold(200),
                armor_bonus: ArmorPoints(7),
                max_dexterity_bonus: AbilityScoreBonus(0),
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

#[derive(Debug, Clone, PartialEq, Eq, Copy, serde::Deserialize, serde::Serialize)]
pub enum AbilityType {
    Strength,
    Dexterity,
    Constitution,
}

impl AbilityType {
    pub fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn from_ability_str(string: &str) -> Option<Self> {
        let string = string.to_lowercase();
        match string.as_str() {
            "strength" => Some(Self::Strength),
            "dexterity" => Some(Self::Dexterity),
            "constitution" => Some(Self::Constitution),
            _ => None,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, derive_more::Deref, PartialEq, Eq)]
pub struct AbilityScore(u16);
#[derive(Debug, serde::Deserialize, serde::Serialize, derive_more::Deref)]
pub struct Strength(AbilityScore);
#[derive(Debug, serde::Deserialize, serde::Serialize, derive_more::Deref)]
pub struct Dexterity(AbilityScore);
#[derive(Debug, serde::Deserialize, serde::Serialize, derive_more::Deref)]
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
#[derive(
    Debug,
    Clone,
    Copy,
    serde::Deserialize,
    serde::Serialize,
    derive_more::Deref,
    derive_more::Add,
    PartialEq,
    Eq,
)]
pub struct AbilityScoreBonus(i16);

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, derive_more::Deref, derive_more::Add,
)]
pub struct Experience(pub u32);

#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    serde::Deserialize,
    serde::Serialize,
    derive_more::Deref,
    derive_more::Add,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct Level(pub u16);
#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    serde::Deserialize,
    serde::Serialize,
    derive_more::Deref,
    derive_more::Add,
    derive_more::Sub,
    derive_more::AddAssign,
    derive_more::SubAssign,
)]
pub struct Health(pub i16);

#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    serde::Deserialize,
    serde::Serialize,
    derive_more::Deref,
    derive_more::Add,
    derive_more::Sub,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct Gold(pub u16);

#[derive(
    Debug,
    Clone,
    Copy,
    serde::Deserialize,
    serde::Serialize,
    derive_more::Deref,
    derive_more::Add,
    Display,
)]
pub struct ArmorPoints(i16);

impl From<AbilityScoreBonus> for ArmorPoints {
    fn from(value: AbilityScoreBonus) -> Self {
        Self(*value)
    }
}

fn ability_score_bonus(ability_score: &AbilityScore) -> AbilityScoreBonus {
    AbilityScoreBonus((ability_score.0 as i16 - 10) / 2)
}

pub fn max_health(constitution: &Constitution, level: Level) -> Health {
    let health = 12 + 6 * *level as i16 + *ability_score_bonus(&constitution.0);
    Health(health as i16)
}

fn attack(attack_stats: &AttackStats) -> Health {
    let damage =
        attack_stats.attack_dice.iter().map(roll).sum::<i16>() + *attack_stats.attack_bonus;
    Health(damage)
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

        let strength_levels = self
            .level_up_choices
            .iter()
            .filter(|choice| choice.ability_increment == AbilityType::Strength)
            .count();

        let dexterity_levels = self
            .level_up_choices
            .iter()
            .filter(|choice| choice.ability_increment == AbilityType::Dexterity)
            .count();

        let constitution_levels = self
            .level_up_choices
            .iter()
            .filter(|choice| choice.ability_increment == AbilityType::Constitution)
            .count();

        AbilityScores {
            strength: Strength(AbilityScore(
                base_ability_scores.strength.0.0 + strength_levels as u16,
            )),
            dexterity: Dexterity(AbilityScore(
                base_ability_scores.dexterity.0.0 + dexterity_levels as u16,
            )),
            constitution: Constitution(AbilityScore(
                base_ability_scores.constitution.0.0 + constitution_levels as u16,
            )),
        }
    }

    pub fn experience_level(&self) -> Level {
        experience_level(self.experience)
    }

    pub fn level(&self) -> Level {
        Level(self.level_up_choices.len() as u16)
    }

    pub fn max_health(&self) -> Health {
        max_health(&self.ability_scores().constitution, self.level())
    }

    pub fn attack_stats(&self) -> AttackStats {
        let dex = &self.ability_scores().dexterity;
        let str = &self.ability_scores().strength;

        let Some(weapon) = &self.equipped_weapon else {
            let ability_bonus = match dex.0.0 < str.0.0 {
                true => ability_score_bonus(&str.0),
                false => ability_score_bonus(&dex.0),
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
            AbilityScaling::Strength => ability_score_bonus(&str.0),
            AbilityScaling::Dexterity => ability_score_bonus(&dex.0),
            AbilityScaling::Either => match dex.0.0 < str.0.0 {
                true => ability_score_bonus(&str.0),
                false => ability_score_bonus(&dex.0),
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
        let dexterity_ability_score_bonus = ability_score_bonus(&self.ability_scores().dexterity.0);

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
    pub damage: Health,
    pub critical_hit: bool,
    pub character_name: String,
}

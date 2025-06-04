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

pub trait Object {
    fn id(&self) -> &i64;
    fn name(&self) -> String;
}

pub trait Hitable {
    fn attacked(&mut self, hit_bonus: &i16, attack_dice: &Dice) -> Option<Attack>;
    fn is_alive(&self) -> bool;
}

pub trait Hit {
    fn attack_dice(&self) -> &Dice;
    fn hit_bonus(&self) -> &i16;
}

pub trait Combat: Object + Hitable + Hit {}
impl<T: Object + Hitable + Hit> Combat for T {}

pub struct Character {
    pub id: i64,
    pub name: String,
    pub attack_dice: Dice,
    pub hit_bonus: i16,
    pub max_health: i16,
    pub current_health: i16,
    pub armor_points: i16,
}

impl Character {
    pub fn new(
        id: i64,
        name: &str,
        health: i16,
        armor_points: i16,
        attack_dice: Dice,
        hit_bonus: i16,
    ) -> Self {
        Self {
            id,
            name: name.to_owned(),
            max_health: health,
            current_health: health,
            armor_points,
            attack_dice,
            hit_bonus,
        }
    }

    pub fn rest(&mut self) {
        self.current_health = self.max_health;
    }
}

impl Object for Character {
    fn id(&self) -> &i64 {
        &self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Hit for Character {
    fn attack_dice(&self) -> &Dice {
        &self.attack_dice
    }

    fn hit_bonus(&self) -> &i16 {
        &self.hit_bonus
    }
}

impl Hitable for Character {
    fn is_alive(&self) -> bool {
        self.current_health > 0
    }

    fn attacked(&mut self, hit_bonus: &i16, attack_dice: &Dice) -> Option<Attack> {
        let dice_roll = roll(&Dice::D20);
        let hit = dice_roll + hit_bonus;
        let critical_hit = dice_roll == 20;
        let critical_miss = dice_roll == 1;

        if critical_miss {
            return None;
        }

        if hit > self.armor_points || critical_hit {
            let damage = match critical_hit {
                true => roll(attack_dice) + roll(attack_dice),
                false => roll(attack_dice),
            };

            self.current_health -= damage;

            let outcome = Attack {
                roll: hit,
                damage,
                critical_hit,
            };

            return Some(outcome);
        }

        None
    }
}

pub struct Attack {
    pub roll: i16,
    pub damage: i16,
    pub critical_hit: bool,
}

use crate::{
    Dice, armor::ArmorType, items::ItemType, jewelry::JewelryType, roll_success, types::Level,
    weapons::WeaponType,
};

pub struct Loot {
    pub weapons: Vec<WeaponType>,
    pub armor: Vec<ArmorType>,
    pub items: Vec<ItemType>,
    pub jewelry: Vec<JewelryType>,
}

pub fn get_loot(level: Level) -> Loot {
    let index = *level as usize;
    let loot_chance = loot_catalogue().remove(index);
    loot_chance.into()
}

fn loot_catalogue() -> Vec<LootChances> {
    vec![
        LootChances {
            weapons: vec![(WeaponType::Dagger, Dice::D4)],
            armor: vec![(ArmorType::Leather, Dice::D4)],
            items: vec![(ItemType::Stone, Dice::D4)],
            jewelry: vec![],
        },
        LootChances {
            weapons: vec![
                (WeaponType::Dagger, Dice::D6),
                (WeaponType::Shortsword, Dice::D8),
                (WeaponType::Longsword, Dice::D8),
            ],
            armor: vec![
                (ArmorType::Leather, Dice::D8),
                (ArmorType::Chainmail, Dice::D8),
            ],
            items: vec![
                (ItemType::Stone, Dice::D4),
                (ItemType::PotionOfHealing, Dice::D4),
                (ItemType::PotionOfHealing, Dice::D4),
            ],
            jewelry: vec![
                (JewelryType::BrassRing, Dice::D8),
                (JewelryType::RingOfProtection, Dice::D20),
            ],
        },
        LootChances {
            weapons: vec![
                (WeaponType::Dagger, Dice::D6),
                (WeaponType::Shortsword, Dice::D8),
                (WeaponType::Longsword, Dice::D8),
            ],
            armor: vec![
                (ArmorType::Leather, Dice::D8),
                (ArmorType::Chainmail, Dice::D8),
                (ArmorType::Splint, Dice::D8),
            ],
            items: vec![
                (ItemType::Stone, Dice::D4),
                (ItemType::PotionOfHealing, Dice::D4),
                (ItemType::PotionOfHealing, Dice::D4),
                (ItemType::PotionOfHealing, Dice::D4),
                (ItemType::ScrollOfWeaken, Dice::D6),
            ],
            jewelry: vec![(JewelryType::RingOfProtection, Dice::D8)],
        },
    ]
}

struct LootChances {
    pub weapons: Vec<(WeaponType, Dice)>,
    pub armor: Vec<(ArmorType, Dice)>,
    pub items: Vec<(ItemType, Dice)>,
    pub jewelry: Vec<(JewelryType, Dice)>,
}

impl From<LootChances> for Loot {
    fn from(value: LootChances) -> Self {
        Self {
            weapons: roll_items(value.weapons),
            armor: roll_items(value.armor),
            items: roll_items(value.items),
            jewelry: roll_items(value.jewelry),
        }
    }
}

fn roll_items<T>(items: Vec<(T, Dice)>) -> Vec<T> {
    items
        .into_iter()
        .flat_map(|item| roll_item(item.0, item.1))
        .collect()
}

fn roll_item<T>(item: T, dice: Dice) -> Option<T> {
    if roll_success(&dice) {
        Some(item)
    } else {
        None
    }
}

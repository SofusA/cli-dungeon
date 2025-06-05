use cli_dungeon_rules::{ArmorType, WeaponType};

pub struct Shop {
    pub weapons: Vec<WeaponType>,
    pub armor: Vec<ArmorType>,
}
pub fn available_in_shop() -> Shop {
    Shop {
        weapons: vec![
            WeaponType::Dagger,
            WeaponType::Shortsword,
            WeaponType::Longsword,
            WeaponType::Shield,
        ],
        armor: vec![ArmorType::Leather, ArmorType::ChainMail],
    }
}

use crate::types::{ArmorPoints, Constitution, Dexterity, Gold, Strength};

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum JewelryType {
    BrassRing,
    RingOfProtection,
    RingOfStrength,
    RingOfDexterity,
    RingOfConstitution,
}

pub struct Jewelry {
    pub name: String,
    pub cost: Gold,
    pub armor_bonus: ArmorPoints,
    pub strength_bonus: Strength,
    pub dexterity_bonus: Dexterity,
    pub constitution_bonus: Constitution,
}

impl JewelryType {
    fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn from_jewelry_str(string: &str) -> Option<Self> {
        let string = string.to_lowercase();
        match string.as_str() {
            "brass ring" => Some(Self::BrassRing),
            "ring of protection" => Some(Self::RingOfProtection),
            _ => None,
        }
    }

    pub fn to_jewelry(&self) -> Jewelry {
        match self {
            JewelryType::BrassRing => Jewelry {
                name: self.to_name(),
                cost: Gold::new(300),
                armor_bonus: ArmorPoints::new(0),
                strength_bonus: Strength::new(0),
                dexterity_bonus: Dexterity::new(0),
                constitution_bonus: Constitution::new(0),
            },
            JewelryType::RingOfProtection => Jewelry {
                name: self.to_name(),
                cost: Gold::new(30000),
                armor_bonus: ArmorPoints::new(1),
                strength_bonus: Strength::new(0),
                dexterity_bonus: Dexterity::new(0),
                constitution_bonus: Constitution::new(0),
            },
            JewelryType::RingOfStrength => Jewelry {
                name: self.to_name(),
                cost: Gold::new(30000),
                armor_bonus: ArmorPoints::new(0),
                strength_bonus: Strength::new(1),
                dexterity_bonus: Dexterity::new(0),
                constitution_bonus: Constitution::new(0),
            },
            JewelryType::RingOfDexterity => Jewelry {
                name: self.to_name(),
                cost: Gold::new(30000),
                armor_bonus: ArmorPoints::new(0),
                strength_bonus: Strength::new(0),
                dexterity_bonus: Dexterity::new(1),
                constitution_bonus: Constitution::new(0),
            },
            JewelryType::RingOfConstitution => Jewelry {
                name: self.to_name(),
                cost: Gold::new(30000),
                armor_bonus: ArmorPoints::new(0),
                strength_bonus: Strength::new(0),
                dexterity_bonus: Dexterity::new(0),
                constitution_bonus: Constitution::new(1),
            },
        }
    }
}

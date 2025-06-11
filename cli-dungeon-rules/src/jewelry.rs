use crate::types::{ArmorPoints, Gold};

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum JewelryType {
    BrassRing,
    RingOfProtection,
}

pub struct Jewelry {
    pub name: String,
    pub cost: Gold,
    pub armor_bonus: ArmorPoints,
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
            },
            JewelryType::RingOfProtection => Jewelry {
                name: self.to_name(),
                cost: Gold::new(30000),
                armor_bonus: ArmorPoints::new(1),
            },
        }
    }
}

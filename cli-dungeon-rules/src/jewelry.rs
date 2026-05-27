use crate::{
    conditions::{Condition, ConditionType},
    normalize_name,
    types::Gold,
};

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum JewelryType {
    BrassRing,
    RingOfStrength,
    RingOfDexterity,
    RingOfConstitution,
    RingOfProtection,
    AmuletOfFocus,
    AmuletOfPower,
    AmuletOfAgility,
}

pub struct Jewelry {
    pub name: String,
    pub cost: Gold,
    pub condition: Option<Condition>,
}

impl JewelryType {
    fn to_name(self) -> String {
        match self {
            JewelryType::BrassRing => "Brass ring",
            JewelryType::RingOfStrength => "Ring of Strength",
            JewelryType::RingOfDexterity => "Ring of Dexterity",
            JewelryType::RingOfConstitution => "Ring of Constitution",
            JewelryType::RingOfProtection => "Ring of Protection",
            JewelryType::AmuletOfFocus => "Amulet of Focus",
            JewelryType::AmuletOfPower => "Amulet of Power",
            JewelryType::AmuletOfAgility => "Amulet of Agility",
        }
        .to_string()
    }

    pub fn from_jewelry_str(string: &str) -> Option<Self> {
        let normalized = normalize_name(string);

        match normalized.as_str() {
            "brassring" => Some(Self::BrassRing),
            "ringofstrength" => Some(Self::RingOfStrength),
            "ringofdexterity" => Some(Self::RingOfDexterity),
            "ringofconstitution" => Some(Self::RingOfConstitution),
            "ringofprotection" => Some(Self::RingOfProtection),
            "amuletoffocus" => Some(Self::AmuletOfFocus),
            "amuletofpower" => Some(Self::AmuletOfPower),
            "amuletofagility" => Some(Self::AmuletOfAgility),
            _ => None,
        }
    }

    pub fn to_jewelry(&self) -> Jewelry {
        match self {
            JewelryType::BrassRing => Jewelry {
                name: self.to_name(),
                cost: Gold::new(300),
                condition: None,
            },
            JewelryType::RingOfStrength => Jewelry {
                name: self.to_name(),
                cost: Gold::new(250),
                condition: Some(ConditionType::StrengthMinor.to_condition()),
            },
            JewelryType::RingOfDexterity => Jewelry {
                name: self.to_name(),
                cost: Gold::new(250),
                condition: Some(ConditionType::DexterityMinor.to_condition()),
            },
            JewelryType::RingOfConstitution => Jewelry {
                name: self.to_name(),
                cost: Gold::new(300),
                condition: Some(ConditionType::ConstitutionMinor.to_condition()),
            },
            JewelryType::RingOfProtection => Jewelry {
                name: self.to_name(),
                cost: Gold::new(350),
                condition: Some(ConditionType::ArmorMinor.to_condition()),
            },
            JewelryType::AmuletOfFocus => Jewelry {
                name: self.to_name(),
                cost: Gold::new(400),
                condition: Some(ConditionType::Focused.to_condition()),
            },
            JewelryType::AmuletOfPower => Jewelry {
                name: self.to_name(),
                cost: Gold::new(500),
                condition: Some(ConditionType::Strong.to_condition()),
            },
            JewelryType::AmuletOfAgility => Jewelry {
                name: self.to_name(),
                cost: Gold::new(500),
                condition: Some(ConditionType::Agile.to_condition()),
            },
        }
    }
}

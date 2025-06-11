use crate::types::Gold;

#[derive(
    Debug, Clone, Copy, serde::Deserialize, serde::Serialize, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum ItemType {
    Stone,
}

pub struct Item {
    pub name: String,
    pub cost: Gold,
}

impl ItemType {
    fn to_name(self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .strip_prefix("\"")
            .unwrap()
            .strip_suffix("\"")
            .unwrap()
            .to_string()
    }

    pub fn to_item(&self) -> Item {
        match self {
            ItemType::Stone => Item {
                name: self.to_name(),
                cost: Gold::new(1),
            },
        }
    }
}

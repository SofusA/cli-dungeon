use cli_dungeon_database::DatabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Character is dead")]
    Dead,

    #[error("Character is not in a fight")]
    NotFighting,

    #[error("It is not your turn!")]
    NotPlayerTurn,

    #[error("Ability scores must sum to 10")]
    AbilitySumError,

    #[error("Weapon cannot be wielded in offhand")]
    NotOffHandWeapon,

    #[error("Your character is not strong enough")]
    InsufficientStrength,

    #[error("Insufficient gold")]
    InsufficientGold,

    #[error("No short rests remaining")]
    InsufficientShortRests,

    #[error("Insufficient experience for level up")]
    InsufficientExperience,

    #[error("Unknown Item. Spelling error?")]
    UnknownItem,

    #[error("Unknown weapon. Spelling error?")]
    UnknownWeapon,

    #[error("Unknown armor. Spelling error?")]
    UnknownArmor,

    #[error("Unknown jewelry. Spelling error?")]
    UnknownJewelry,

    #[error("Unknown class. Spelling error?")]
    UnknownClass,

    #[error("Weapon not in inventory")]
    WeaponNotInInventory,
    #[error("Armor not in inventory")]
    ArmorNotInInventory,
    #[error("Weapon not in inventory")]
    JewelryNotInInventory,

    #[error("Too many jewelries equipped. Unequip one first")]
    TooManyJewelriesEquipped,

    #[error(transparent)]
    Database(#[from] DatabaseError),
}

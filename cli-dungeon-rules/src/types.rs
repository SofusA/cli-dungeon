use derive_more::{Add, AddAssign, Deref, Display, Sub, SubAssign};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize, Deref, Add)]
pub struct Experience(pub u32);

#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    serde::Deserialize,
    serde::Serialize,
    Deref,
    Add,
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
    Deref,
    Add,
    Sub,
    AddAssign,
    SubAssign,
)]
pub struct HealthPoints(pub i16);

#[derive(
    Debug,
    Clone,
    Copy,
    Display,
    serde::Deserialize,
    serde::Serialize,
    Deref,
    Add,
    Sub,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct Gold(pub u16);

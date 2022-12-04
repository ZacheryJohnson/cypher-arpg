use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Debug, Serialize, PartialEq, Eq)]
pub enum ItemClassification {
    Invalid,
    Equippable(ItemEquipSlot),
    Currency,
}

impl std::fmt::Display for ItemClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Deserialize, Debug, Serialize, PartialEq, Eq)]
pub enum ItemEquipSlot {
    // These are all WIP! Expect these to change
    Head,
    LeftArm,
    RightArm,
    Body,
    Belt,
    Legs,
    Boots,
}

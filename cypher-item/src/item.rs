pub struct _Item;

use cypher_core::stat::StatList;

/// Items that can be purchased.
pub trait Purchaseable {
    /// How much does it cost to purchase this item?
    fn purchase_price() -> Vec<_Item>;

    /// Can this item be purchased infinitely?
    fn finite() -> bool;
}

/// Items that can be sold.
pub trait Sellable {
    /// How much does this item sell for?
    fn sell_price() -> Vec<_Item>;
}

/// Items that can be equipped.
pub trait Equippable {
    /// What stats are required to equip this item?
    fn requirements() -> Option<StatList>;
}

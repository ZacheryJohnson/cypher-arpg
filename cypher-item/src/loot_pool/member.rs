use std::sync::{Arc, Mutex};

use serde::{Serialize, Serializer};

use crate::item::definition::ItemDefinition;

/// A [LootPoolMember] is a pairing of an item that can drop, in tandem with the chance that item will drop.
///
/// The lifetime `'item` is that of the [ItemDefinitionDatabase], as each [LootPoolMember] contains a reference
/// to an [ItemDefinition] within the [ItemDefinitionDatabase] instance.
#[derive(Clone, Debug, Serialize)]
pub struct LootPoolMember {
    #[serde(serialize_with = "serialize_item_def_member")]
    #[serde(rename = "item_id")]
    /// What item will be generated when selected.
    /// The affixes of the item are resolved when generating the item itself, outside of the purview of [LootPool]s.
    pub item_def: Arc<Mutex<ItemDefinition>>,

    /// Weight indicates how often this member will be chosen. A higher value = more common.
    pub weight: u64,
}

fn serialize_item_def_member<S>(
    definition: &Arc<Mutex<ItemDefinition>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u64(definition.lock().unwrap().id)
}

use std::sync::{Arc, Mutex};

use cypher_core::{
    affix::database::AffixDefinitionDatabase,
    affix_pool::database::AffixPoolDefinitionDatabase,
    data::{DataDefinition, DataDefinitionDatabase},
};

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::{Serialize, Serializer};

use crate::item::{database::ItemDefinitionDatabase, Item, ItemDefinition, ItemDefinitionCriteria};

pub mod database;
pub mod deserializer;

pub type LootPoolDefinitionId = u32;

/// A [LootPoolDefinition] is a collection of [LootPoolMember]s. When generating items from a [LootPool],
/// the item will be chosen from one of the [LootPoolMember]s.
/// Enemies may have one or more [LootPoolDefinition]s.
#[derive(Debug, Serialize)]
pub struct LootPoolDefinition {
    id: LootPoolDefinitionId,

    /// All [LootPoolMember]s that can drop as part of this [LootPoolDefinition].
    members: Vec<LootPoolMember>,
}

#[derive(Default)]
pub struct LootPoolCriteria {}

impl DataDefinition for LootPoolDefinition {
    type DefinitionTypeId = LootPoolDefinitionId;

    fn validate(&self) -> bool {
        !self.members.is_empty()
    }
}

impl LootPoolDefinition {
    pub fn generate(
        definition: Arc<Mutex<LootPoolDefinition>>,
        item_database: Arc<Mutex<ItemDefinitionDatabase>>,
        affix_database: Arc<Mutex<AffixDefinitionDatabase>>,
        affix_pool_database: Arc<Mutex<AffixPoolDefinitionDatabase>>,
        _criteria: &LootPoolCriteria,
    ) -> Item {
        let weights = definition
            .lock()
            .unwrap()
            .members
            .iter()
            .map(|member| member.weight)
            .collect::<Vec<u64>>();

        let distribution = WeightedIndex::new(weights.as_slice()).unwrap();
        let mut rng = rand::thread_rng();
        let item_id = definition.lock().unwrap().members[distribution.sample(&mut rng)]
            .item_def
            .lock()
            .unwrap()
            .id;

        let definition = item_database.lock().unwrap().definition(item_id).unwrap();

        ItemDefinition::generate(
            definition,
            affix_database,
            affix_pool_database,
            &ItemDefinitionCriteria::default(),
        )
    }
}

/// A [LootPoolMember] is a pairing of an item that can drop, in tandem with the chance that item will drop.
///
/// The lifetime `'item` is that of the [ItemDefinitionDatabase], as each [LootPoolMember] contains a reference
/// to an [ItemDefinition] within the [ItemDefinitionDatabase] instance.
#[derive(Debug, Serialize)]
pub struct LootPoolMember {
    #[serde(serialize_with = "serialize_item_def_member")]
    /// What item will be generated when selected.
    /// The affixes of the item are resolved when generating the item itself, outside of the purview of [LootPool]s.
    item_def: Arc<Mutex<ItemDefinition>>,

    /// Weight indicates how often this member will be chosen. A higher value = more common.
    weight: u64,
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

#[cfg(test)]
mod tests {
    use super::{database::LootPoolDatabase, *};
    use crate::item::database::ItemDefinitionDatabase;

    #[test]
    fn loot_pool_initialize() {
        let affix_database = Arc::new(Mutex::new(AffixDefinitionDatabase::initialize()));
        let affix_pool_database = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::initialize(
            affix_database.clone(),
        )));
        let item_database = Arc::new(Mutex::new(ItemDefinitionDatabase::initialize(
            affix_pool_database.clone(),
        )));
        let loot_pool_database = Arc::new(Mutex::new(LootPoolDatabase::initialize(
            item_database.clone(),
        )));

        assert!(loot_pool_database.lock().unwrap().validate())
    }

    #[test]
    fn loot_pool_generation() {
        let affix_database = Arc::new(Mutex::new(AffixDefinitionDatabase::initialize()));
        let affix_pool_database = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::initialize(
            affix_database.clone(),
        )));
        let item_database = Arc::new(Mutex::new(ItemDefinitionDatabase::initialize(
            affix_pool_database.clone(),
        )));
        let loot_pool_database = Arc::new(Mutex::new(LootPoolDatabase::initialize(
            item_database.clone(),
        )));

        let database = loot_pool_database.lock().unwrap();
        let definition = database.pools.get(&1).unwrap();

        for _ in 0..10 {
            let item = LootPoolDefinition::generate(
                definition.to_owned(),
                item_database.clone(),
                affix_database.clone(),
                affix_pool_database.clone(),
                &LootPoolCriteria::default(),
            );
            println!("{:?}", item);
        }
    }
}

use cypher_core::{
    affix::{database::AffixDefinitionDatabase, pool::AffixPoolDefinitionDatabase},
    data::{DataDefinition, DataDefinitionDatabase},
};

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::Serialize;

use crate::item::{database::ItemDefinitionDatabase, Item, ItemDefinition, ItemDefinitionCriteria};

pub mod database;
pub mod deserializer;

pub type LootPoolDefinitionId = u32;

/// A [LootPoolDefinition] is a collection of [LootPoolMember]s. When generating items from a [LootPool],
/// the item will be chosen from one of the [LootPoolMember]s.
/// Enemies may have one or more [LootPoolDefinition]s.
#[derive(Debug, Serialize)]
pub struct LootPoolDefinition<'db> {
    id: LootPoolDefinitionId,

    /// All [LootPoolMember]s that can drop as part of this [LootPoolDefinition].
    members: Vec<LootPoolMember<'db>>,
}

#[derive(Default)]
pub struct LootPoolCriteria {}

impl<'db> DataDefinition for LootPoolDefinition<'db> {
    type DefinitionTypeId = LootPoolDefinitionId;

    fn validate(&self) -> bool {
        !self.members.is_empty()
    }
}

impl<'db> LootPoolDefinition<'db> {
    pub fn generate(
        &self,
        item_database: &'db ItemDefinitionDatabase,
        affix_database: &'db AffixDefinitionDatabase,
        affix_pool_database: &'db AffixPoolDefinitionDatabase,
        _criteria: &LootPoolCriteria,
    ) -> Item {
        let weights = self
            .members
            .iter()
            .map(|member| member.weight)
            .collect::<Vec<u64>>();

        let distribution = WeightedIndex::new(weights.as_slice()).unwrap();
        let mut rng = rand::thread_rng();
        let item_id = self.members[distribution.sample(&mut rng)].item_def.id;

        let definition = item_database.get_definition_by_id(item_id).unwrap();

        definition.generate(
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
pub struct LootPoolMember<'db> {
    /// What item will be generated when selected.
    /// The affixes of the item are resolved when generating the item itself, outside of the purview of [LootPool]s.
    item_def: &'db ItemDefinition<'db>,

    /// Weight indicates how often this member will be chosen. A higher value = more common.
    weight: u64,
}

#[cfg(test)]
mod tests {
    use super::{database::LootPoolDatabase, *};
    use crate::item::database::ItemDefinitionDatabase;

    #[test]
    fn loot_pool_initialize() {
        let affix_database = AffixDefinitionDatabase::initialize();
        let affix_pool_database = AffixPoolDefinitionDatabase::initialize(&affix_database);
        let item_database = ItemDefinitionDatabase::initialize(&affix_pool_database);
        let loot_pool_database = LootPoolDatabase::initialize(&item_database);

        assert!(loot_pool_database.validate())
    }

    #[test]
    fn loot_pool_generation() {
        let affix_database = AffixDefinitionDatabase::initialize();
        let affix_pool_database = AffixPoolDefinitionDatabase::initialize(&affix_database);
        let item_database = ItemDefinitionDatabase::initialize(&affix_pool_database);
        let loot_pool_database = LootPoolDatabase::initialize(&item_database);

        let loot_pool = loot_pool_database.pools.get(&1).unwrap();

        for _ in 0..10 {
            let item = loot_pool.generate(
                &item_database,
                &affix_database,
                &affix_pool_database,
                &LootPoolCriteria::default(),
            );
            println!("{:?}", item);
        }
    }
}

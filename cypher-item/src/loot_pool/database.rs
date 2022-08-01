use std::collections::HashMap;

use cypher_core::data::{DataDefinition, DataDefinitionDatabase};
use serde::de::DeserializeSeed;

use crate::item::database::ItemDefinitionDatabase;

use super::{deserializer::LootPoolDatabaseDeserializer, LootPoolDefinition, LootPoolDefinitionId};

pub struct LootPoolDatabase<'db> {
    pub(crate) pools: HashMap<LootPoolDefinitionId, LootPoolDefinition<'db>>,
}

impl<'db> LootPoolDatabase<'db> {
    pub fn initialize(item_db: &'db ItemDefinitionDatabase) -> Self {
        let loot_pool_file = include_str!("../../data/loot_pool.json");

        let loot_pool_deserializer = LootPoolDatabaseDeserializer { item_db };
        let pools_database: Vec<LootPoolDefinition> = loot_pool_deserializer
            .deserialize(&mut serde_json::Deserializer::from_str(loot_pool_file))
            .unwrap();

        let pools = pools_database
            .into_iter()
            .map(|pool| (pool.id, pool))
            .collect::<HashMap<_, _>>();

        LootPoolDatabase { pools }
    }
}

impl<'db> DataDefinitionDatabase<'db, LootPoolDefinition<'db>> for LootPoolDatabase<'db> {
    fn validate(&'db self) -> bool {
        !self.pools.is_empty() && self.pools.iter().all(|(_id, pool_def)| pool_def.validate())
    }

    fn get_definition_by_id(&self, id: LootPoolDefinitionId) -> Option<&LootPoolDefinition> {
        self.pools.get(&id)
    }
}

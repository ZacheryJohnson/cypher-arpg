use std::{collections::HashMap, sync::Arc};

use cypher_core::data::{DataDefinition, DataDefinitionDatabase};
use serde::de::DeserializeSeed;

use crate::item::database::ItemDefinitionDatabase;

use super::{deserializer::LootPoolDatabaseDeserializer, LootPoolDefinition, LootPoolDefinitionId};

pub struct LootPoolDatabase {
    pub(crate) pools: HashMap<LootPoolDefinitionId, Arc<LootPoolDefinition>>,
}

impl LootPoolDatabase {
    pub fn initialize(item_db: Arc<ItemDefinitionDatabase>) -> Self {
        let loot_pool_file = include_str!("../../data/loot_pool.json");

        let loot_pool_deserializer = LootPoolDatabaseDeserializer { item_db };
        let pools_database: Vec<Arc<LootPoolDefinition>> = loot_pool_deserializer
            .deserialize(&mut serde_json::Deserializer::from_str(loot_pool_file))
            .unwrap();

        let pools = pools_database
            .into_iter()
            .map(|pool| (pool.id, pool))
            .collect::<HashMap<_, _>>();

        LootPoolDatabase { pools }
    }
}

impl DataDefinitionDatabase<LootPoolDefinition> for LootPoolDatabase {
    fn validate(&self) -> bool {
        !self.pools.is_empty() && self.pools.iter().all(|(_id, pool_def)| pool_def.validate())
    }

    fn get_definition_by_id(&self, id: LootPoolDefinitionId) -> Option<Arc<LootPoolDefinition>> {
        self.pools.get(&id).map(|arc| arc.to_owned())
    }
    fn definitions(&self) -> Vec<Arc<LootPoolDefinition>> {
        self.pools.iter().map(|(_, def)| def.to_owned()).collect()
    }
}

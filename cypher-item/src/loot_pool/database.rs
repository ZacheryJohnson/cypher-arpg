use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cypher_core::data::{DataDefinition, DataDefinitionDatabase};
use serde::de::DeserializeSeed;

use crate::item::database::ItemDefinitionDatabase;

use super::{deserializer::LootPoolDatabaseDeserializer, LootPoolDefinition, LootPoolDefinitionId};

pub struct LootPoolDefinitionDatabase {
    pub(crate) pools: HashMap<LootPoolDefinitionId, Arc<Mutex<LootPoolDefinition>>>,
}

impl LootPoolDefinitionDatabase {
    pub fn initialize(item_db: Arc<Mutex<ItemDefinitionDatabase>>) -> Self {
        let mut path = std::env::current_dir().unwrap();
        path.push("..");
        path.push("cypher-item");
        path.push("data");
        path.push("loot_pool.json");

        Self::load_from(item_db, path.to_str().unwrap())
    }

    pub fn load_from<S: Into<String>>(
        item_db: Arc<Mutex<ItemDefinitionDatabase>>,
        path: S,
    ) -> Self {
        let loot_pool_file = String::from_utf8(std::fs::read(path.into()).unwrap()).unwrap();

        let loot_pool_deserializer = LootPoolDatabaseDeserializer { item_db };
        let pools_database: Vec<LootPoolDefinition> = loot_pool_deserializer
            .deserialize(&mut serde_json::Deserializer::from_str(
                loot_pool_file.as_str(),
            ))
            .unwrap();

        let pools = pools_database
            .into_iter()
            .map(|pool| (pool.id, Arc::new(Mutex::new(pool))))
            .collect::<HashMap<_, _>>();

        LootPoolDefinitionDatabase { pools }
    }
}

impl DataDefinitionDatabase<LootPoolDefinition> for LootPoolDefinitionDatabase {
    fn validate(&self) -> bool {
        !self.pools.is_empty()
            && self
                .pools
                .iter()
                .all(|(_id, pool_def)| pool_def.lock().unwrap().validate())
    }

    fn definition(&self, id: LootPoolDefinitionId) -> Option<Arc<Mutex<LootPoolDefinition>>> {
        self.pools.get(&id).map(|arc| arc.to_owned())
    }
    fn definitions(&self) -> Vec<Arc<Mutex<LootPoolDefinition>>> {
        self.pools.iter().map(|(_, def)| def.to_owned()).collect()
    }
}

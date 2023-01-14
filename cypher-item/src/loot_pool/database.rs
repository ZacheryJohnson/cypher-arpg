use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cypher_core::data::{DataDefinition, DataDefinitionDatabase};
use serde::de::DeserializeSeed;

use crate::item::database::ItemDefinitionDatabase;

use super::{
    definition::{LootPoolDefinition, LootPoolDefinitionId},
    deserializer::LootPoolDatabaseDeserializer,
};

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

        Self::load_from(path.to_str().unwrap(), &item_db)
    }
}

impl DataDefinitionDatabase<LootPoolDefinition> for LootPoolDefinitionDatabase {
    type DataDependencies = Arc<Mutex<ItemDefinitionDatabase>>;

    fn load_from<S: Into<String>>(path: S, dependencies: &Self::DataDependencies) -> Self {
        let loot_pool_file = String::from_utf8(std::fs::read(path.into()).unwrap()).unwrap();

        let loot_pool_deserializer = LootPoolDatabaseDeserializer {
            item_db: dependencies.clone(),
        };
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

    fn write_to<S: Into<String>>(&self, path: S) {
        let definition_clones = self
            .pools
            .values()
            .map(|def| def.lock().unwrap().to_owned())
            .collect::<Vec<LootPoolDefinition>>();

        let serialized = serde_json::ser::to_string(&definition_clones)
            .expect("failed to serialize loot pool database");

        std::fs::write(path.into(), serialized).expect("failed to write serialized data to path");
    }

    fn validate(&self) -> bool {
        !self.pools.is_empty()
            && self
                .pools
                .values()
                .all(|pool_def| pool_def.lock().unwrap().validate())
    }

    fn definition(&self, id: LootPoolDefinitionId) -> Option<Arc<Mutex<LootPoolDefinition>>> {
        self.pools.get(&id).map(|arc| arc.to_owned())
    }
    fn definitions(&self) -> Vec<Arc<Mutex<LootPoolDefinition>>> {
        self.pools.values().map(|def| def.to_owned()).collect()
    }

    fn add_definition(&mut self, definition: LootPoolDefinition) {
        self.pools
            .insert(definition.id, Arc::new(Mutex::new(definition)));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use cypher_core::{
        affix::database::AffixDefinitionDatabase,
        affix_pool::database::AffixPoolDefinitionDatabase, data::DataDefinitionDatabase,
    };

    use super::LootPoolDefinitionDatabase;
    use crate::item::database::ItemDefinitionDatabase;

    #[test]
    fn loot_pool_initialize() {
        let affix_database = Arc::new(Mutex::new(AffixDefinitionDatabase::initialize()));
        let affix_pool_database = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::initialize(
            affix_database.clone(),
        )));
        let item_database = Arc::new(Mutex::new(ItemDefinitionDatabase::initialize(
            affix_database.clone(),
            affix_pool_database.clone(),
        )));
        let loot_pool_database = Arc::new(Mutex::new(LootPoolDefinitionDatabase::initialize(
            item_database.clone(),
        )));

        assert!(loot_pool_database.lock().unwrap().validate())
    }
}

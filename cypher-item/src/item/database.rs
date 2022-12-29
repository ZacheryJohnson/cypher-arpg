use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cypher_core::{
    affix_pool::database::AffixPoolDefinitionDatabase,
    data::{DataDefinition, DataDefinitionDatabase},
};
use serde::de::DeserializeSeed;

use super::{
    definition::{ItemDefinition, ItemDefinitionId},
    deserializer::ItemDefinitionDatabaseDeserializer,
};

pub struct ItemDefinitionDatabase {
    pub(super) items: HashMap<ItemDefinitionId, Arc<Mutex<ItemDefinition>>>,
}

impl ItemDefinitionDatabase {
    pub fn initialize(affix_pool_db: Arc<Mutex<AffixPoolDefinitionDatabase>>) -> Self {
        let mut path = std::env::current_dir().unwrap();
        path.push("..");
        path.push("cypher-item");
        path.push("data");
        path.push("item.json");

        Self::load_from(path.to_str().unwrap(), &affix_pool_db)
    }
}

impl DataDefinitionDatabase<ItemDefinition> for ItemDefinitionDatabase {
    type DataDependencies = Arc<Mutex<AffixPoolDefinitionDatabase>>;

    fn load_from<S: Into<String>>(path: S, dependencies: &Self::DataDependencies) -> Self {
        let item_file = String::from_utf8(std::fs::read(path.into()).unwrap()).unwrap();

        let item_def_deserializer = ItemDefinitionDatabaseDeserializer {
            affix_pool_db: dependencies.clone(),
        };
        let definitions: Vec<ItemDefinition> = item_def_deserializer
            .deserialize(&mut serde_json::Deserializer::from_str(item_file.as_str()))
            .unwrap();

        let items = definitions
            .into_iter()
            .map(|item| (item.id, Arc::new(Mutex::new(item))))
            .collect::<HashMap<_, _>>();

        ItemDefinitionDatabase { items }
    }

    fn write_to<S: Into<String>>(&self, path: S) {
        let definition_clones = self
            .items
            .values()
            .map(|def| def.lock().unwrap().to_owned())
            .collect::<Vec<ItemDefinition>>();

        let serialized = serde_json::ser::to_string(&definition_clones)
            .expect("failed to serialize item database");

        std::fs::write(path.into(), serialized).expect("failed to write serialized data to path");
    }

    fn validate(&self) -> bool {
        !self.items.is_empty()
            && self
                .items
                .values()
                .all(|item_def| item_def.lock().unwrap().validate())
    }

    fn definition(&self, id: ItemDefinitionId) -> Option<Arc<Mutex<ItemDefinition>>> {
        self.items.get(&id).map(|arc| arc.to_owned())
    }

    fn definitions(&self) -> Vec<Arc<Mutex<ItemDefinition>>> {
        self.items.values().map(|def| def.to_owned()).collect()
    }

    fn add_definition(&mut self, definition: ItemDefinition) {
        self.items
            .insert(definition.id, Arc::new(Mutex::new(definition)));
    }
}

#[cfg(test)]
mod tests {
    use cypher_core::{
        affix::database::AffixDefinitionDatabase, affix_pool::database::AffixPoolDefinitionDatabase,
    };

    use super::{ItemDefinitionDatabase, *};

    #[test]
    fn init_item_database() {
        let affix_db = Arc::new(Mutex::new(AffixDefinitionDatabase::initialize()));
        let affix_pool_db = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::initialize(
            affix_db.clone(),
        )));
        let _item_db = Arc::new(ItemDefinitionDatabase::initialize(affix_pool_db.clone()));
    }
}

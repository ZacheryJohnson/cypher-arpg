use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use cypher_core::{
    affix_pool::database::AffixPoolDefinitionDatabase,
    data::{DataDefinition, DataDefinitionDatabase},
};
use serde::de::DeserializeSeed;

use super::{deserializer::ItemDefinitionDatabaseDeserializer, ItemDefinition, ItemDefinitionId};

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

        Self::load_from(affix_pool_db, path.to_str().unwrap())
    }

    pub fn load_from<S: Into<String>>(
        affix_pool_db: Arc<Mutex<AffixPoolDefinitionDatabase>>,
        path: S,
    ) -> Self {
        let item_file = String::from_utf8(std::fs::read(path.into()).unwrap()).unwrap();

        let item_def_deserializer = ItemDefinitionDatabaseDeserializer { affix_pool_db };
        let definitions: Vec<ItemDefinition> = item_def_deserializer
            .deserialize(&mut serde_json::Deserializer::from_str(item_file.as_str()))
            .unwrap();

        let items = definitions
            .into_iter()
            .map(|item| (item.id, Arc::new(Mutex::new(item))))
            .collect::<HashMap<_, _>>();

        ItemDefinitionDatabase { items }
    }
}

impl DataDefinitionDatabase<ItemDefinition> for ItemDefinitionDatabase {
    fn validate(&self) -> bool {
        !self.items.is_empty()
            && self
                .items
                .iter()
                .all(|(_id, item_def)| item_def.lock().unwrap().validate())
    }

    fn definition(&self, id: ItemDefinitionId) -> Option<Arc<Mutex<ItemDefinition>>> {
        self.items.get(&id).map(|arc| arc.to_owned())
    }

    fn definitions(&self) -> Vec<Arc<Mutex<ItemDefinition>>> {
        self.items.iter().map(|(_, def)| def.to_owned()).collect()
    }
}

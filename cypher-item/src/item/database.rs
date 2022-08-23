use std::{collections::HashMap, sync::Arc};

use cypher_core::{
    affix::pool::AffixPoolDefinitionDatabase,
    data::{DataDefinition, DataDefinitionDatabase},
};
use serde::de::DeserializeSeed;

use super::{deserializer::ItemDefinitionDatabaseDeserializer, ItemDefinition, ItemDefinitionId};

pub struct ItemDefinitionDatabase {
    pub(super) items: HashMap<ItemDefinitionId, Arc<ItemDefinition>>,
}

impl ItemDefinitionDatabase {
    pub fn initialize(affix_pool_db: Arc<AffixPoolDefinitionDatabase>) -> Self {
        let item_file = include_str!("../../data/item.json");

        let item_def_deserializer = ItemDefinitionDatabaseDeserializer { affix_pool_db };
        let definitions = item_def_deserializer
            .deserialize(&mut serde_json::Deserializer::from_str(item_file))
            .unwrap();

        let items = definitions
            .into_iter()
            .map(|item| (item.id, item))
            .collect::<HashMap<_, _>>();

        ItemDefinitionDatabase { items }
    }
}

impl DataDefinitionDatabase<ItemDefinition> for ItemDefinitionDatabase {
    fn validate(&self) -> bool {
        !self.items.is_empty() && self.items.iter().all(|(_id, item_def)| item_def.validate())
    }

    fn get_definition_by_id(&self, id: ItemDefinitionId) -> Option<Arc<ItemDefinition>> {
        self.items.get(&id).map(|arc| arc.to_owned())
    }

    fn definitions(&self) -> Vec<Arc<ItemDefinition>> {
        self.items.iter().map(|(_, def)| def.to_owned()).collect()
    }
}

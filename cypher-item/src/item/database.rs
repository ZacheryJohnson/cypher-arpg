use std::collections::HashMap;

use cypher_core::{
    affix::pool::AffixPoolDefinitionDatabase,
    data::{DataDefinition, DataDefinitionDatabase},
};
use serde::de::DeserializeSeed;

use super::{deserializer::ItemDefinitionDatabaseDeserializer, ItemDefinition, ItemDefinitionId};

pub struct ItemDefinitionDatabase<'db> {
    pub(super) items: HashMap<ItemDefinitionId, ItemDefinition<'db>>,
}

impl<'db> ItemDefinitionDatabase<'db> {
    pub fn initialize(affix_pool_db: &'db AffixPoolDefinitionDatabase) -> Self {
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

impl<'db> DataDefinitionDatabase<'db, ItemDefinition<'db>> for ItemDefinitionDatabase<'db> {
    fn validate(&'db self) -> bool {
        !self.items.is_empty() && self.items.iter().all(|(_id, item_def)| item_def.validate())
    }

    fn get_definition_by_id(&self, id: ItemDefinitionId) -> Option<&ItemDefinition> {
        self.items.get(&id)
    }
}

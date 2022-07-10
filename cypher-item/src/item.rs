use cypher_core::{
    affix::{Affix, AffixDefinitionDatabase, AffixDefinitionId, AffixGenerationCriteria},
    data::DataDefinitionDatabase,
};
use rand::{distributions::WeightedIndex, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type ItemDefinitionId = u64;

pub struct ItemDefinitionDatabase {
    items: HashMap<ItemDefinitionId, ItemDefinition>,
}

impl DataDefinitionDatabase for ItemDefinitionDatabase {
    type DefinitionT = ItemDefinition;
    type DefinitionId = ItemDefinitionId;

    fn initialize() -> ItemDefinitionDatabase {
        let item_file = include_str!("../data/item.json");

        let definitions: Vec<Self::DefinitionT> = serde_json::de::from_str(item_file).unwrap();

        let items = definitions
            .into_iter()
            .map(|item| (item.id, item))
            .collect::<HashMap<_, _>>();

        ItemDefinitionDatabase { items }
    }

    fn get_definition_by_id(&self, id: &Self::DefinitionId) -> Option<&Self::DefinitionT> {
        self.items.get(id)
    }
}

#[derive(Clone)]
pub struct ItemDefinitionCriteria {
    pub allowed_affix_definition_ids: Option<HashSet<AffixDefinitionId>>,

    pub disallowed_affix_definition_ids: Option<HashSet<AffixDefinitionId>>,

    /// How many affixes can this item roll? Stored as tuples, where tuple.0 = number of affixes possible, and tuple.1 = affix weight
    pub affix_count_weighting: Vec<(u8 /* count */, u64 /* weight */)>,
}

impl Default for ItemDefinitionCriteria {
    fn default() -> Self {
        Self {
            affix_count_weighting: vec![(1, 500), (2, 300), (3, 100), (4, 20), (5, 5), (6, 1)],

            allowed_affix_definition_ids: Default::default(),
            disallowed_affix_definition_ids: Default::default(),
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ItemDefinition {
    pub id: ItemDefinitionId,

    // TODO: remove this
    #[serde(skip_serializing_if = "Option::is_none")]
    debug_name: Option<String>,
}

impl ItemDefinition {
    pub fn generate(
        &self,
        affix_database: &AffixDefinitionDatabase,
        criteria: &ItemDefinitionCriteria,
    ) -> Item {
        let mut affix_criteria = AffixGenerationCriteria {
            allowed_ids: criteria.allowed_affix_definition_ids.clone(),
            disallowed_ids: criteria.disallowed_affix_definition_ids.clone(),
            ..Default::default()
        };

        let mut affixes = vec![];

        let distribution = WeightedIndex::new(
            criteria
                .affix_count_weighting
                .iter()
                .map(|pair| pair.1)
                .collect::<Vec<u64>>()
                .as_slice(),
        )
        .unwrap();
        let mut rng = rand::thread_rng();
        let affix_count = criteria.affix_count_weighting[distribution.sample(&mut rng)].0;

        for _ in 0..affix_count {
            let affix = affix_database.generate(&affix_criteria);
            if affix.is_none() {
                continue;
            }

            let affix_definition = affix_database
                .get_definition_by_id(&affix.as_ref().unwrap().definition)
                .unwrap();
            if affix_criteria.disallowed_ids.is_none() {
                affix_criteria.disallowed_ids = Some(HashSet::new());
            }
            affix_criteria
                .disallowed_ids
                .as_mut()
                .unwrap()
                .insert(affix_definition.id);

            // TODO: handle None
            affixes.push(affix.unwrap());
        }

        Item {
            definition_id: self.id,
            affixes,
        }
    }
}

#[derive(Debug)]
pub struct Item {
    pub definition_id: ItemDefinitionId,

    pub affixes: Vec<Affix>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_item_database() {
        let _item_database = ItemDefinitionDatabase::initialize();
    }
}

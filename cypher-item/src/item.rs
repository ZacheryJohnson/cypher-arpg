use cypher_core::{
    affix::{
        database::AffixDefinitionDatabase,
        definition::AffixGenerationCriteria,
        pool::{AffixPoolDefinition, AffixPoolDefinitionDatabase, AffixPoolDefinitionId},
        Affix,
    },
    data::{DataDefinition, DataDefinitionDatabase},
};
use rand::{distributions::WeightedIndex, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub type ItemDefinitionId = u64;

pub struct ItemDefinitionDatabase {
    items: HashMap<ItemDefinitionId, ItemDefinition>,
}

impl DataDefinitionDatabase<ItemDefinition> for ItemDefinitionDatabase {
    fn initialize() -> ItemDefinitionDatabase {
        let item_file = include_str!("../data/item.json");

        let definitions: Vec<ItemDefinition> = serde_json::de::from_str(item_file).unwrap();

        let items = definitions
            .into_iter()
            .map(|item| (item.id, item))
            .collect::<HashMap<_, _>>();

        ItemDefinitionDatabase { items }
    }

    fn get_definition_by_id(&self, id: &ItemDefinitionId) -> Option<&ItemDefinition> {
        self.items.get(id)
    }
}

#[derive(Clone)]
pub struct ItemDefinitionCriteria {
    /// How many affixes can this item roll? Stored as tuples, where tuple.0 = number of affixes possible, and tuple.1 = affix weight
    pub affix_count_weighting: Vec<(u8 /* count */, u64 /* weight */)>,
}

impl Default for ItemDefinitionCriteria {
    fn default() -> Self {
        Self {
            affix_count_weighting: vec![(1, 500), (2, 300), (3, 100), (4, 20), (5, 5), (6, 1)],
        }
    }
}

#[derive(Clone, Copy, Deserialize, Debug, Serialize, PartialEq)]
pub enum ItemClassification {
    Invalid,
    Equippable(ItemEquipSlot),
    Currency,
}

#[derive(Clone, Copy, Deserialize, Debug, Serialize, PartialEq)]
pub enum ItemEquipSlot {
    // These are all WIP! Expect these to change
    Head,
    LeftArm,
    RightArm,
    Body,
    Belt,
    Legs,
    Boots,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ItemDefinition {
    pub id: ItemDefinitionId,

    pub classification: ItemClassification,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub affix_pools: Option<Vec<AffixPoolDefinitionId>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl DataDefinition for ItemDefinition {
    type DefinitionTypeId = ItemDefinitionId;

    fn validate(&self) -> bool {
        self.classification != ItemClassification::Invalid
            && (self.affix_pools.is_none() || !self.affix_pools.as_ref().unwrap().is_empty())
    }
}

impl ItemDefinition {
    pub fn generate(
        &self,
        affix_database: &AffixDefinitionDatabase,
        affix_pool_database: &AffixPoolDefinitionDatabase,
        criteria: &ItemDefinitionCriteria,
    ) -> Item {
        let mut affix_criteria = AffixGenerationCriteria::default();

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

        let mut affix_pool_members = vec![];
        for pool_id in self.affix_pools.as_ref().unwrap_or(&vec![]) {
            let affix_pool = affix_pool_database.get_definition_by_id(pool_id).unwrap(); // TODO: remove unwrap
            for member in &affix_pool.members {
                affix_pool_members.push(member.to_owned());
            }
        }

        let pool = AffixPoolDefinition::from_members(affix_pool_members);

        let mut affixes = vec![];
        for _ in 0..affix_count {
            let affix = pool.generate(affix_database, &affix_criteria);
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
            classification: self.classification,
            affixes,
        }
    }
}

#[derive(Debug)]
pub struct Item {
    pub definition_id: ItemDefinitionId,

    pub classification: ItemClassification,

    pub affixes: Vec<Affix>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_item_database() {
        let _ = ItemDefinitionDatabase::initialize();
    }

    #[test]
    fn loot_generation() {
        let item_database = ItemDefinitionDatabase::initialize();
        let affix_database = AffixDefinitionDatabase::initialize();
        let affix_pool_database = AffixPoolDefinitionDatabase::initialize();

        let definition = *item_database
            .items
            .iter()
            .map(|item| item.1.to_owned())
            .collect::<Vec<&ItemDefinition>>()
            .choose(&mut rand::thread_rng())
            .unwrap();

        let criteria = ItemDefinitionCriteria::default();

        for _ in 0..10 {
            let _ = definition.generate(&affix_database, &affix_pool_database, &criteria);
        }
    }
}

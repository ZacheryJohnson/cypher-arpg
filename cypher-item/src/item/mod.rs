use cypher_core::{
    affix::{
        database::AffixDefinitionDatabase,
        definition::AffixGenerationCriteria,
        pool::{AffixPoolDefinition, AffixPoolDefinitionDatabase},
        Affix,
    },
    data::{DataDefinition, DataDefinitionDatabase},
};
use rand::{distributions::WeightedIndex, prelude::*};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub type ItemDefinitionId = u64;

pub mod database;
pub mod deserializer;

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

#[derive(Debug, Serialize)]
pub struct ItemDefinition<'db> {
    pub id: ItemDefinitionId,

    pub classification: ItemClassification,

    pub affix_pools: Option<Vec<&'db AffixPoolDefinition<'db>>>,

    name: String,
}

impl DataDefinition for ItemDefinition<'_> {
    type DefinitionTypeId = ItemDefinitionId;

    fn validate(&self) -> bool {
        self.classification != ItemClassification::Invalid
            && (self.affix_pools.is_none() || !self.affix_pools.as_ref().unwrap().is_empty())
    }
}

impl<'db> ItemDefinition<'db> {
    pub fn generate(
        &self,
        affix_database: &'db AffixDefinitionDatabase,
        affix_pool_database: &'db AffixPoolDefinitionDatabase,
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
        for pool_def in self.affix_pools.as_ref().unwrap_or(&vec![]) {
            let affix_pool = affix_pool_database
                .get_definition_by_id(pool_def.id)
                .unwrap(); // TODO: remove unwrap
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
                .get_definition_by_id(affix.as_ref().unwrap().definition.id)
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
pub struct Item<'db> {
    pub definition_id: ItemDefinitionId,

    pub classification: ItemClassification,

    pub affixes: Vec<Affix<'db>>,
}

#[cfg(test)]
mod tests {
    use super::{database::ItemDefinitionDatabase, *};

    #[test]
    fn init_item_database() {
        let affix_db = AffixDefinitionDatabase::initialize();
        let affix_pool_db = AffixPoolDefinitionDatabase::initialize(&affix_db);
        let _item_db = ItemDefinitionDatabase::initialize(&affix_pool_db);
    }

    #[test]
    fn loot_generation() {
        let affix_database = AffixDefinitionDatabase::initialize();
        let affix_pool_database = AffixPoolDefinitionDatabase::initialize(&affix_database);
        let item_database = ItemDefinitionDatabase::initialize(&affix_pool_database);

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

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
use std::{collections::HashSet, sync::Arc};

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
pub struct ItemDefinition {
    pub id: ItemDefinitionId,

    pub classification: ItemClassification,

    pub affix_pools: Option<Vec<Arc<AffixPoolDefinition>>>,

    name: String,
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
        definition: Arc<ItemDefinition>,
        affix_database: Arc<AffixDefinitionDatabase>,
        affix_pool_database: Arc<AffixPoolDefinitionDatabase>,
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

        for pool_def in definition.affix_pools.as_ref().unwrap_or(&vec![]) {
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
            let affix = pool.generate(affix_database.clone(), &affix_criteria);
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
            definition,
            affixes,
        }
    }
}

#[derive(Debug)]
pub struct Item {
    pub definition: Arc<ItemDefinition>,

    pub affixes: Vec<Affix>,
}

#[cfg(test)]
mod tests {
    use super::{database::ItemDefinitionDatabase, *};

    #[test]
    fn init_item_database() {
        let affix_db = Arc::new(AffixDefinitionDatabase::initialize());
        let affix_pool_db = Arc::new(AffixPoolDefinitionDatabase::initialize(affix_db.clone()));
        let _item_db = Arc::new(ItemDefinitionDatabase::initialize(affix_pool_db.clone()));
    }

    #[test]
    fn loot_generation() {
        let affix_database = Arc::new(AffixDefinitionDatabase::initialize());
        let affix_pool_database = Arc::new(AffixPoolDefinitionDatabase::initialize(
            affix_database.clone(),
        ));
        let item_database = Arc::new(ItemDefinitionDatabase::initialize(
            affix_pool_database.clone(),
        ));

        let definition = item_database
            .items
            .iter()
            .map(|item| item.1.to_owned())
            .collect::<Vec<Arc<ItemDefinition>>>()
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_owned();

        let criteria = ItemDefinitionCriteria::default();

        for _ in 0..10 {
            let _ = ItemDefinition::generate(
                definition.clone(),
                affix_database.clone(),
                affix_pool_database.clone(),
                &criteria,
            );
        }
    }
}

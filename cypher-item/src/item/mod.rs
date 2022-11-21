use cypher_core::{
    affix::{database::AffixDefinitionDatabase, definition::AffixGenerationCriteria, Affix},
    affix_pool::{database::AffixPoolDefinitionDatabase, definition::AffixPoolDefinition},
    data::{DataDefinition, DataDefinitionDatabase},
};
use rand::{distributions::WeightedIndex, prelude::*};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

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

#[derive(Clone, Copy, Deserialize, Debug, Serialize, PartialEq, Eq)]
pub enum ItemClassification {
    Invalid,
    Equippable(ItemEquipSlot),
    Currency,
}

#[derive(Clone, Copy, Deserialize, Debug, Serialize, PartialEq, Eq)]
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

    #[serde(serialize_with = "serialize_affix_pools_member")]
    pub affix_pools: Vec<Arc<Mutex<AffixPoolDefinition>>>,

    name: String,
}

fn serialize_affix_pools_member<S>(
    pools: &Vec<Arc<Mutex<AffixPoolDefinition>>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;

    let len = pools.len();
    let mut seq = s.serialize_seq(if len > 0 { Some(len) } else { None })?;
    for elem in pools {
        seq.serialize_element(&elem.lock().unwrap().id)?;
    }
    seq.end()
}

impl DataDefinition for ItemDefinition {
    type DefinitionTypeId = ItemDefinitionId;

    fn validate(&self) -> bool {
        self.classification != ItemClassification::Invalid
    }
}

impl ItemDefinition {
    pub fn generate(
        definition: Arc<Mutex<ItemDefinition>>,
        affix_database: Arc<Mutex<AffixDefinitionDatabase>>,
        affix_pool_database: Arc<Mutex<AffixPoolDefinitionDatabase>>,
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

        let pool_definitions = definition.lock().unwrap();
        for pool_def in pool_definitions.affix_pools.clone() {
            let affix_pool = affix_pool_database
                .lock()
                .unwrap()
                .definition(pool_def.lock().unwrap().id)
                .unwrap(); // TODO: remove unwrap
            for member in &affix_pool.lock().unwrap().members {
                affix_pool_members.push(member.to_owned());
            }
        }

        let pool = AffixPoolDefinition::with_members(affix_pool_members);

        let mut affixes = vec![];
        for _ in 0..affix_count {
            let affix = pool.generate(affix_database.clone(), &affix_criteria);
            if affix.is_none() {
                continue;
            }

            let affix_definition = affix_database
                .lock()
                .unwrap()
                .definition(affix.as_ref().unwrap().definition.lock().unwrap().id)
                .unwrap();
            if affix_criteria.disallowed_ids.is_none() {
                affix_criteria.disallowed_ids = Some(HashSet::new());
            }
            affix_criteria
                .disallowed_ids
                .as_mut()
                .unwrap()
                .insert(affix_definition.lock().unwrap().id);

            // TODO: handle None
            affixes.push(affix.unwrap());
        }

        Item {
            definition: definition.clone(),
            affixes,
        }
    }
}

#[derive(Debug)]
pub struct Item {
    pub definition: Arc<Mutex<ItemDefinition>>,

    pub affixes: Vec<Affix>,
}

#[cfg(test)]
mod tests {
    use super::{database::ItemDefinitionDatabase, *};

    #[test]
    fn init_item_database() {
        let affix_db = Arc::new(Mutex::new(AffixDefinitionDatabase::initialize()));
        let affix_pool_db = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::initialize(
            affix_db.clone(),
        )));
        let _item_db = Arc::new(ItemDefinitionDatabase::initialize(affix_pool_db.clone()));
    }

    #[test]
    fn loot_generation() {
        let affix_database = Arc::new(Mutex::new(AffixDefinitionDatabase::initialize()));
        let affix_pool_database = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::initialize(
            affix_database.clone(),
        )));
        let item_database = Arc::new(Mutex::new(ItemDefinitionDatabase::initialize(
            affix_pool_database.clone(),
        )));

        let definition = item_database
            .lock()
            .unwrap()
            .items
            .iter()
            .map(|item| item.1.to_owned())
            .collect::<Vec<Arc<Mutex<ItemDefinition>>>>()
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

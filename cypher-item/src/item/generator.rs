use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use cypher_core::{
    affix::{database::AffixDefinitionDatabase, definition::AffixGenerationCriteria},
    affix_pool::{database::AffixPoolDefinitionDatabase, definition::AffixPoolDefinition},
    data::{DataDefinitionDatabase, DataInstanceGenerator},
};

use rand::{distributions::WeightedIndex, prelude::*};

use super::{Item, ItemDefinition};

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

pub struct ItemGenerator;

impl DataInstanceGenerator<ItemDefinition, Item, ItemDefinitionCriteria> for ItemGenerator {
    type DataDependencies = (
        Arc<Mutex<AffixDefinitionDatabase>>,
        Arc<Mutex<AffixPoolDefinitionDatabase>>,
    );

    fn generate(
        &self,
        definition: Arc<Mutex<ItemDefinition>>,
        criteria: &ItemDefinitionCriteria,
        dependencies: &Self::DataDependencies,
    ) -> Item {
        let (affix_db, affix_pool_db) = dependencies;

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

        let item_definition = definition.lock().unwrap();
        for pool_def in item_definition.affix_pools.clone() {
            let affix_pool = affix_pool_db
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
            let affix = pool.generate(affix_db.clone(), &affix_criteria);
            if affix.is_none() {
                continue;
            }

            let affix_definition = affix_db
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use cypher_core::data::DataInstanceGenerator;
    use cypher_core::{
        affix::database::AffixDefinitionDatabase, affix_pool::database::AffixPoolDefinitionDatabase,
    };
    use rand::seq::SliceRandom;

    use crate::item::{database::ItemDefinitionDatabase, ItemDefinition};

    use super::{ItemDefinitionCriteria, ItemGenerator};

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

        let item_generator = ItemGenerator;
        for _ in 0..10 {
            let _ = item_generator.generate(
                definition.clone(),
                &criteria,
                &(affix_database.clone(), affix_pool_database.clone()),
            );
        }
    }
}

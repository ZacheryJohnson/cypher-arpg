use std::sync::{Arc, Mutex};
use uuid::{self, Uuid};

use cypher_core::{
    affix::{
        database::AffixDefinitionDatabase,
        generator::{AffixGenerationCriteria, AffixGenerator},
        instance::AffixInstance,
    },
    affix_pool::{
        database::AffixPoolDefinitionDatabase,
        definition::AffixPoolDefinition,
        generator::{AffixPoolGenerationCriteria, AffixPoolGenerator},
    },
    data::{DataDefinitionDatabase, DataInstanceGenerator},
};

use rand::{distributions::WeightedIndex, prelude::*};

use super::{definition::ItemDefinition, instance::ItemInstance};

pub struct ItemDefinitionCriteria {
    /// How many affixes can this item roll? Stored as tuples, where tuple.0 = number of affixes possible, and tuple.1 = affix weight
    pub affix_count_weighting: Vec<(u8 /* count */, u64 /* weight */)>,

    pub affix_generation_criteria: AffixGenerationCriteria,
}

impl Default for ItemDefinitionCriteria {
    fn default() -> Self {
        Self {
            affix_count_weighting: vec![(1, 500), (2, 300), (3, 100), (4, 20), (5, 5), (6, 1)],
            affix_generation_criteria: Default::default(),
        }
    }
}

pub struct ItemGenerator;

fn generate_from_affix_pool(
    criteria: &ItemDefinitionCriteria,
    definition: Arc<Mutex<ItemDefinition>>,
    affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
    affix_pool_db: Arc<Mutex<AffixPoolDefinitionDatabase>>,
) -> Vec<AffixInstance> {
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

    let pool = Arc::new(Mutex::new(AffixPoolDefinition::with_members(
        affix_pool_members,
    )));
    let affix_generator = AffixGenerator {};
    let affix_pool_generator = AffixPoolGenerator {};

    let mut affixes = vec![];

    for _ in 0..affix_count {
        if let Some(affix_def) = affix_pool_generator.generate(
            pool.clone(),
            &AffixPoolGenerationCriteria::default(),
            &(affix_db.clone()),
        ) {
            let affix_criteria = &criteria.affix_generation_criteria;
            let affix = affix_generator.generate(affix_def.clone(), affix_criteria, &());

            if let Some(affix_instance) = affix {
                affixes.push(affix_instance);
            } else {
                // TODO: don't continue? this seems bad
                continue;
            }
        } else {
            // TODO: don't continue? this seems bad
            continue;
        }
    }

    affixes
}

fn generate_from_fixed_affixes(
    criteria: &ItemDefinitionCriteria,
    definition: Arc<Mutex<ItemDefinition>>,
) -> Vec<AffixInstance> {
    let mut affixes = vec![];

    let affix_generator = AffixGenerator {};
    let affix_criteria = &criteria.affix_generation_criteria;

    for fixed_affix in &definition.lock().unwrap().fixed_affixes {
        let affix = affix_generator.generate(fixed_affix.clone(), affix_criteria, &());

        if let Some(affix_instance) = affix {
            affixes.push(affix_instance);
        } else {
            panic!("Failed to generate fixed affix")
        }
    }

    affixes
}

impl DataInstanceGenerator<ItemDefinition, ItemInstance, ItemDefinitionCriteria> for ItemGenerator {
    type DataDependencies = (
        Arc<Mutex<AffixDefinitionDatabase>>,
        Arc<Mutex<AffixPoolDefinitionDatabase>>,
    );

    fn generate(
        &self,
        definition: Arc<Mutex<ItemDefinition>>,
        criteria: &ItemDefinitionCriteria,
        dependencies: &Self::DataDependencies,
    ) -> Option<ItemInstance> {
        let (affix_db, affix_pool_db) = dependencies;

        let has_fixed_affixes = !definition.lock().unwrap().fixed_affixes.is_empty();

        let affixes = {
            if has_fixed_affixes {
                generate_from_fixed_affixes(criteria, definition.clone())
            } else {
                generate_from_affix_pool(
                    criteria,
                    definition.clone(),
                    affix_db.to_owned(),
                    affix_pool_db.to_owned(),
                )
            }
        };

        Some(ItemInstance {
            guid: Uuid::new_v4().to_string(),
            definition,
            affixes,
        })
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

    use crate::item::database::ItemDefinitionDatabase;
    use crate::item::definition::ItemDefinition;

    use super::{ItemDefinitionCriteria, ItemGenerator};

    #[test]
    fn loot_generation() {
        let affix_database = Arc::new(Mutex::new(AffixDefinitionDatabase::initialize()));
        let affix_pool_database = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::initialize(
            affix_database.clone(),
        )));
        let item_database = Arc::new(Mutex::new(ItemDefinitionDatabase::initialize(
            affix_database.clone(),
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

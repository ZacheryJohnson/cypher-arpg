use std::sync::{Arc, Mutex};

use cypher_core::{
    affix::database::AffixDefinitionDatabase,
    affix_pool::database::AffixPoolDefinitionDatabase,
    data::{DataDefinitionDatabase, DataInstanceGenerator},
};
use rand::{distributions::WeightedIndex, prelude::Distribution};

use crate::item::{
    database::ItemDefinitionDatabase,
    generator::{ItemDefinitionCriteria, ItemGenerator},
    instance::ItemInstance,
};

use super::definition::LootPoolDefinition;

#[derive(Default)]
pub struct LootPoolCriteria {}

#[derive(Clone, Default)]
pub struct LootPoolItemGenerator;

impl DataInstanceGenerator<LootPoolDefinition, ItemInstance, LootPoolCriteria>
    for LootPoolItemGenerator
{
    type DataDependencies = (
        Arc<Mutex<AffixDefinitionDatabase>>,
        Arc<Mutex<AffixPoolDefinitionDatabase>>,
        Arc<Mutex<ItemDefinitionDatabase>>,
    );

    fn generate(
        &self,
        definition: Arc<Mutex<LootPoolDefinition>>,
        _criteria: &LootPoolCriteria,
        dependencies: &Self::DataDependencies,
    ) -> Option<ItemInstance> {
        let (affix_db, affix_pool_db, item_db) = dependencies;

        let weights = definition
            .lock()
            .unwrap()
            .members
            .iter()
            .map(|member| member.weight)
            .collect::<Vec<u64>>();

        let distribution = WeightedIndex::new(weights.as_slice()).unwrap();
        let mut rng = rand::thread_rng();
        let item_id = definition.lock().unwrap().members[distribution.sample(&mut rng)]
            .item_def
            .lock()
            .unwrap()
            .id;

        let definition = item_db.lock().unwrap().definition(item_id).unwrap();

        let item_generator = ItemGenerator;

        item_generator.generate(
            definition,
            &ItemDefinitionCriteria::default(),
            &(affix_db.clone(), affix_pool_db.clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use cypher_core::{
        affix::database::AffixDefinitionDatabase,
        affix_pool::database::AffixPoolDefinitionDatabase, data::DataInstanceGenerator,
    };

    use crate::{
        item::database::ItemDefinitionDatabase,
        loot_pool::{database::LootPoolDefinitionDatabase, generator::LootPoolCriteria},
    };

    use super::LootPoolItemGenerator;

    #[test]
    fn loot_pool_generation() {
        let affix_database = Arc::new(Mutex::new(AffixDefinitionDatabase::initialize()));
        let affix_pool_database = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::initialize(
            affix_database.clone(),
        )));
        let item_database = Arc::new(Mutex::new(ItemDefinitionDatabase::initialize(
            affix_pool_database.clone(),
        )));
        let loot_pool_database = Arc::new(Mutex::new(LootPoolDefinitionDatabase::initialize(
            item_database.clone(),
        )));

        let database = loot_pool_database.lock().unwrap();
        let definition = database.pools.get(&1).unwrap();

        let generator = LootPoolItemGenerator;

        for _ in 0..10 {
            let item = generator.generate(
                definition.to_owned(),
                &LootPoolCriteria::default(),
                &(
                    affix_database.clone(),
                    affix_pool_database.clone(),
                    item_database.clone(),
                ),
            );
            println!("{:?}", item);
        }
    }
}

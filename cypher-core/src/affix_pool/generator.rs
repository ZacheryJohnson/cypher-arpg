use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use rand::{distributions::WeightedIndex, prelude::Distribution};

use crate::{
    affix::{
        database::AffixDefinitionDatabase,
        definition::{AffixDefinition, AffixDefinitionId},
        placement::AffixPlacement,
    },
    data::{DataDefinitionDatabase, DataInstanceGenerator},
};

use super::{definition::AffixPoolDefinition, member::AffixPoolMember};

pub struct AffixPoolGenerator {}

#[derive(Default, Debug)]
/// Requirements when generating [AffixInstance]s from an AffixPool.
pub struct AffixPoolGenerationCriteria {
    /// Which [AffixDefinition]s should be considered, if any.
    pub allowed_ids: Option<HashSet<AffixDefinitionId>>,

    /// Which [AffixDefinition]s should be excluded, if any.
    pub disallowed_ids: Option<HashSet<AffixDefinitionId>>,

    /// [AffixPlacement] to force, if any. If not Suffix or Prefix, can be either.
    pub placement: Option<AffixPlacement>,
}

impl
    DataInstanceGenerator<
        AffixPoolDefinition,
        Arc<Mutex<AffixDefinition>>,
        AffixPoolGenerationCriteria,
    > for AffixPoolGenerator
{
    type DataDependencies = Arc<Mutex<AffixDefinitionDatabase>>;

    fn generate(
        &self,
        definition: std::sync::Arc<std::sync::Mutex<AffixPoolDefinition>>,
        criteria: &AffixPoolGenerationCriteria,
        databases: &Self::DataDependencies,
    ) -> Option<Arc<Mutex<AffixDefinition>>> {
        let definition = definition.lock().unwrap();
        let filtered = definition
            .members
            .iter()
            .filter(|member| {
                criteria.allowed_ids.is_none()
                    || criteria
                        .allowed_ids
                        .as_ref()
                        .unwrap()
                        .contains(&member.affix_def.lock().unwrap().id)
            })
            .filter(|member| {
                criteria.disallowed_ids.is_none()
                    || !criteria
                        .disallowed_ids
                        .as_ref()
                        .unwrap()
                        .contains(&member.affix_def.lock().unwrap().id)
            })
            .filter(|member| {
                criteria.placement.is_none()
                    || *criteria.placement.as_ref().unwrap()
                        == member.affix_def.lock().unwrap().placement
            })
            .map(|member| member.to_owned())
            .collect::<Vec<AffixPoolMember>>();

        let weights = filtered
            .iter()
            .map(|member| member.weight)
            .collect::<Vec<u64>>();

        if let Ok(distribution) = WeightedIndex::new(weights.as_slice()) {
            let mut rng = rand::thread_rng();
            let affix_id = filtered[distribution.sample(&mut rng)]
                .affix_def
                .lock()
                .unwrap()
                .id;

            let affix_database = databases;
            Some(affix_database.lock().unwrap().definition(affix_id).unwrap())
        } else {
            None
        }
    }
}

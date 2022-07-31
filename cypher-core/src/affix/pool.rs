use std::collections::HashMap;

use rand::{distributions::WeightedIndex, prelude::*};
use serde::{de::DeserializeSeed, Serialize};

use crate::data::{DataDefinition, DataDefinitionDatabase};

use super::{
    database::AffixDefinitionDatabase,
    definition::{AffixDefinition, AffixGenerationCriteria},
    deserializer::AffixPoolDatabaseDeserializer,
    Affix,
};

pub type AffixPoolDefinitionId = u32;

pub struct AffixPoolDefinitionDatabase<'db> {
    affix_pools: HashMap<AffixPoolDefinitionId, AffixPoolDefinition<'db>>,
}

impl<'db> DataDefinitionDatabase<'db, AffixPoolDefinition<'db>>
    for AffixPoolDefinitionDatabase<'db>
{
    fn get_definition_by_id(
        &'db self,
        id: AffixPoolDefinitionId,
    ) -> Option<&'db AffixPoolDefinition<'db>> {
        self.affix_pools.get(&id)
    }
}

impl<'db> AffixPoolDefinitionDatabase<'db> {
    pub fn initialize(affix_db: &'db AffixDefinitionDatabase) -> Self {
        let affix_file = include_str!("../../data/affix_pool.json");
        let deserializer = AffixPoolDatabaseDeserializer::new(affix_db);
        let definitions = deserializer
            .deserialize(&mut serde_json::Deserializer::from_str(affix_file))
            .unwrap();

        let affix_pools = definitions
            .into_iter()
            .map(|pool| (pool.id, pool))
            .collect::<HashMap<_, _>>();

        AffixPoolDefinitionDatabase { affix_pools }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AffixPoolDefinition<'db> {
    pub id: AffixPoolDefinitionId,

    /// All [AffixPoolMember]s that can roll as part of this [AffixPoolDefinition].
    pub members: Vec<AffixPoolMember<'db>>,

    pub name: String,
}

impl<'db> DataDefinition for AffixPoolDefinition<'db> {
    type DefinitionTypeId = AffixPoolDefinitionId;

    fn validate(&self) -> bool {
        !self.members.is_empty()
    }
}

impl<'db> AffixPoolDefinition<'db> {
    /// Generates an [Affix] given a set of criteria. May return `None` if criteria would exclude all loaded [AffixDefinition]s.
    pub fn generate(
        &self,
        affix_database: &'db AffixDefinitionDatabase,
        criteria: &AffixGenerationCriteria,
    ) -> Option<Affix<'db>> {
        let filtered = self
            .members
            .iter()
            .filter(|affix| {
                criteria.allowed_ids.is_none()
                    || criteria
                        .allowed_ids
                        .as_ref()
                        .unwrap()
                        .contains(&affix.affix_def.id)
            })
            .filter(|affix| {
                criteria.disallowed_ids.is_none()
                    || !criteria
                        .disallowed_ids
                        .as_ref()
                        .unwrap()
                        .contains(&affix.affix_def.id)
            })
            /*
            TODO: support this once members hold references, not just IDs
            .filter(|affix| {
                criteria.placement.is_none()
                    || *criteria.placement.as_ref().unwrap() == affix.1.placement
            })
            */
            .map(|member| member.to_owned())
            .collect::<Vec<AffixPoolMember>>();

        let weights = filtered
            .iter()
            .map(|member| member.weight)
            .collect::<Vec<u64>>();

        if let Ok(distribution) = WeightedIndex::new(weights.as_slice()) {
            let mut rng = rand::thread_rng();
            let affix_id = filtered[distribution.sample(&mut rng)].affix_def.id;

            let affix_definition = affix_database.get_definition_by_id(affix_id).unwrap();

            affix_definition.generate(criteria)
        } else {
            None
        }
    }

    /// Creates a temporary [AffixPoolDefinition] given existing [AffixPoolMember]s.
    pub fn from_members(members: Vec<AffixPoolMember<'db>>) -> AffixPoolDefinition<'db> {
        AffixPoolDefinition {
            id: 0,
            members,
            name: String::from("from_members temp"),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AffixPoolMember<'db> {
    /// What affix will be generated when selected.
    pub affix_def: &'db AffixDefinition,

    /// Weight indicates how often this member will be chosen. A higher value = more common.
    pub weight: u64,
}

#[cfg(test)]
mod tests {
    use super::AffixPoolDefinitionDatabase;
    use crate::affix::database::AffixDefinitionDatabase;

    #[test]
    fn init_affix_pool_database() {
        let affix_db = AffixDefinitionDatabase::initialize();
        let _ = AffixPoolDefinitionDatabase::initialize(&affix_db);
    }
}

use std::{collections::HashMap, sync::Arc};

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

pub struct AffixPoolDefinitionDatabase {
    affix_pools: HashMap<AffixPoolDefinitionId, Arc<AffixPoolDefinition>>,
}

impl DataDefinitionDatabase<AffixPoolDefinition> for AffixPoolDefinitionDatabase {
    fn validate(&self) -> bool {
        !self.affix_pools.is_empty()
            && self
                .affix_pools
                .iter()
                .all(|(_id, pool_def)| pool_def.validate())
    }

    fn get_definition_by_id(&self, id: AffixPoolDefinitionId) -> Option<Arc<AffixPoolDefinition>> {
        self.affix_pools.get(&id).map(|arc| arc.to_owned())
    }

    fn definitions(&self) -> Vec<Arc<AffixPoolDefinition>> {
        self.affix_pools
            .iter()
            .map(|(_, def)| def.to_owned())
            .collect()
    }
}

impl AffixPoolDefinitionDatabase {
    pub fn initialize(affix_db: Arc<AffixDefinitionDatabase>) -> Self {
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
pub struct AffixPoolDefinition {
    pub id: AffixPoolDefinitionId,

    /// All [AffixPoolMember]s that can roll as part of this [AffixPoolDefinition].
    pub members: Vec<Arc<AffixPoolMember>>,

    pub name: String,
}

impl DataDefinition for AffixPoolDefinition {
    type DefinitionTypeId = AffixPoolDefinitionId;

    fn validate(&self) -> bool {
        !self.members.is_empty()
    }
}

impl AffixPoolDefinition {
    /// Generates an [Affix] given a set of criteria. May return `None` if criteria would exclude all loaded [AffixDefinition]s.
    pub fn generate(
        &self,
        affix_database: Arc<AffixDefinitionDatabase>,
        criteria: &AffixGenerationCriteria,
    ) -> Option<Affix> {
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
            .filter(|affix| {
                criteria.placement.is_none()
                    || *criteria.placement.as_ref().unwrap() == affix.affix_def.placement
            })
            .map(|member| member.to_owned())
            .collect::<Vec<Arc<AffixPoolMember>>>();

        let weights = filtered
            .iter()
            .map(|member| member.weight)
            .collect::<Vec<u64>>();

        if let Ok(distribution) = WeightedIndex::new(weights.as_slice()) {
            let mut rng = rand::thread_rng();
            let affix_id = filtered[distribution.sample(&mut rng)].affix_def.id;

            let affix_definition = affix_database.get_definition_by_id(affix_id).unwrap();

            AffixDefinition::generate(affix_definition, criteria)
        } else {
            None
        }
    }

    /// Creates a temporary [AffixPoolDefinition] given existing [AffixPoolMember]s.
    pub fn from_members(members: Vec<Arc<AffixPoolMember>>) -> AffixPoolDefinition {
        AffixPoolDefinition {
            id: 0,
            members,
            name: String::from("from_members temp"),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct AffixPoolMember {
    /// What affix will be generated when selected.
    pub affix_def: Arc<AffixDefinition>,

    /// Weight indicates how often this member will be chosen. A higher value = more common.
    pub weight: u64,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::AffixPoolDefinitionDatabase;
    use crate::affix::database::AffixDefinitionDatabase;

    #[test]
    fn init_affix_pool_database() {
        let affix_db = Arc::new(AffixDefinitionDatabase::initialize());
        let _ = AffixPoolDefinitionDatabase::initialize(affix_db);
    }
}

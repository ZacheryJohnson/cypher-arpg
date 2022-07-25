use std::collections::HashMap;

use rand::{distributions::WeightedIndex, prelude::*};
use serde::{Deserialize, Serialize};

use crate::data::{DataDefinition, DataDefinitionDatabase};

use super::{
    database::AffixDefinitionDatabase,
    definition::{AffixDefinitionId, AffixGenerationCriteria},
    Affix,
};

pub type AffixPoolDefinitionId = u32;

pub struct AffixPoolDefinitionDatabase {
    affix_pools: HashMap<AffixPoolDefinitionId, AffixPoolDefinition>,
}

impl DataDefinitionDatabase<AffixPoolDefinition> for AffixPoolDefinitionDatabase {
    fn initialize() -> Self {
        let affix_file = include_str!("../../data/affix_pool.json");

        let definitions: Vec<AffixPoolDefinition> = serde_json::de::from_str(affix_file).unwrap();

        let affix_pools = definitions
            .into_iter()
            .map(|pool| (pool.id, pool))
            .collect::<HashMap<_, _>>();

        AffixPoolDefinitionDatabase { affix_pools }
    }

    fn get_definition_by_id(&self, id: &AffixPoolDefinitionId) -> Option<&AffixPoolDefinition> {
        self.affix_pools.get(id)
    }
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct AffixPoolDefinition {
    pub id: AffixPoolDefinitionId,

    /// All [AffixPoolMember]s that can roll as part of this [AffixPoolDefinition].
    pub members: Vec<AffixPoolMember>,

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
        affix_database: &AffixDefinitionDatabase,
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
                        .contains(&affix.affix_id)
            })
            .filter(|affix| {
                criteria.disallowed_ids.is_none()
                    || !criteria
                        .disallowed_ids
                        .as_ref()
                        .unwrap()
                        .contains(&affix.affix_id)
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

        let distribution = WeightedIndex::new(weights.as_slice()).unwrap();
        let mut rng = rand::thread_rng();
        let affix_id = filtered[distribution.sample(&mut rng)].affix_id;

        let affix_definition = affix_database.get_definition_by_id(&affix_id).unwrap();

        affix_definition.generate(criteria)
    }

    /// Creates a temporary [AffixPoolDefinition] given existing [AffixPoolMember]s.
    pub fn from_members(members: Vec<AffixPoolMember>) -> AffixPoolDefinition {
        AffixPoolDefinition {
            id: 0,
            members,
            name: String::from("from_members temp"),
        }
    }
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct AffixPoolMember {
    /// What affix will be generated when selected.
    pub affix_id: AffixDefinitionId,

    /// Weight indicates how often this member will be chosen. A higher value = more common.
    pub weight: u64,
}

#[cfg(test)]
mod tests {
    use super::AffixPoolDefinitionDatabase;
    use crate::data::DataDefinitionDatabase;

    #[test]
    fn init_affix_pool_database() {
        let _ = AffixPoolDefinitionDatabase::initialize();
    }
}

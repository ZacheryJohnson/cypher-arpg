use std::sync::{Arc, Mutex};

use rand::{distributions::WeightedIndex, prelude::Distribution};
use serde::Serialize;

use crate::{
    affix::{
        database::AffixDefinitionDatabase,
        definition::{AffixDefinition, AffixGenerationCriteria},
        Affix,
    },
    data::{DataDefinition, DataDefinitionDatabase},
};

use super::{database::AffixPoolDefinitionId, member::AffixPoolMember};

#[derive(Clone, Debug, Serialize)]
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
        affix_database: Arc<Mutex<AffixDefinitionDatabase>>,
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
                        .contains(&affix.affix_def.lock().unwrap().id)
            })
            .filter(|affix| {
                criteria.disallowed_ids.is_none()
                    || !criteria
                        .disallowed_ids
                        .as_ref()
                        .unwrap()
                        .contains(&affix.affix_def.lock().unwrap().id)
            })
            .filter(|affix| {
                criteria.placement.is_none()
                    || *criteria.placement.as_ref().unwrap()
                        == affix.affix_def.lock().unwrap().placement
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

            let affix_definition = affix_database.lock().unwrap().definition(affix_id).unwrap();

            AffixDefinition::generate(affix_definition, criteria)
        } else {
            None
        }
    }

    /// Creates a temporary [AffixPoolDefinition] given existing [AffixPoolMember]s.
    pub fn with_members(members: Vec<AffixPoolMember>) -> AffixPoolDefinition {
        AffixPoolDefinition {
            id: 0,
            members,
            name: String::from("from_members temp"),
        }
    }
}

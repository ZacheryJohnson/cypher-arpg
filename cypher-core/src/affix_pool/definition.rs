use serde::Serialize;

use crate::data::DataDefinition;

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

    fn id(&self) -> u64 {
        self.id as u64
    }

    fn validate(&self) -> bool {
        !self.members.is_empty()
    }
}

impl AffixPoolDefinition {
    /// Creates a temporary [AffixPoolDefinition] given existing [AffixPoolMember]s.
    pub fn with_members(members: Vec<AffixPoolMember>) -> AffixPoolDefinition {
        AffixPoolDefinition {
            id: 0,
            members,
            name: String::from("from_members temp"),
        }
    }
}

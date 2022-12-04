use cypher_core::data::DataDefinition;
use serde::Serialize;

use super::member::LootPoolMember;

pub type LootPoolDefinitionId = u32;

/// A [LootPoolDefinition] is a collection of [LootPoolMember]s. When generating items from a [LootPool],
/// the item will be chosen from one of the [LootPoolMember]s.
/// Enemies may have one or more [LootPoolDefinition]s.
#[derive(Clone, Debug, Serialize)]
pub struct LootPoolDefinition {
    pub id: LootPoolDefinitionId,

    pub name: String,

    /// All [LootPoolMember]s that can drop as part of this [LootPoolDefinition].
    pub members: Vec<LootPoolMember>,
}

impl DataDefinition for LootPoolDefinition {
    type DefinitionTypeId = LootPoolDefinitionId;

    fn validate(&self) -> bool {
        !self.members.is_empty()
    }
}

use std::sync::Arc;

pub trait DataDefinition {
    type DefinitionTypeId;

    /// Validates that this definition is valid, for whatever criteria is defined by the data type.
    /// For example, an affix definition may require that it provides at least 1 bonus.
    fn validate(&self) -> bool;
}

pub trait DataDefinitionDatabase<DataDefinitionType: DataDefinition> {
    /// Returns whether a database has successfully loaded all data.
    fn validate(&self) -> bool;

    /// Returns a data definition given it's ID.
    fn get_definition_by_id(
        &self,
        id: DataDefinitionType::DefinitionTypeId,
    ) -> Option<Arc<DataDefinitionType>>;

    fn definitions(&self) -> Vec<Arc<DataDefinitionType>>;
}

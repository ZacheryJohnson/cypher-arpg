use std::sync::{Arc, Mutex};

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
    fn definition(
        &self,
        id: DataDefinitionType::DefinitionTypeId,
    ) -> Option<Arc<Mutex<DataDefinitionType>>>;

    fn definitions(&self) -> Vec<Arc<Mutex<DataDefinitionType>>>;
}

pub trait DataInstanceGenerator<
    DataDefinitionType: DataDefinition,
    DataInstanceType,
    GeneratorCriteriaType,
>
{
    type DataDependencies;

    fn generate(
        &self,
        definition: Arc<Mutex<DataDefinitionType>>,
        criteria: &GeneratorCriteriaType,
        databases: &Self::DataDependencies,
    ) -> DataInstanceType;
}

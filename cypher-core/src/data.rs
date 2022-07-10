pub trait DataDefinitionDatabase {
    type DefinitionT;
    type DefinitionId;

    fn initialize() -> Self;

    fn get_definition_by_id(&self, id: &Self::DefinitionId) -> Option<&Self::DefinitionT>;
}

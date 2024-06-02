use cypher_core::affix::definition::AffixDefinition;
use cypher_core::affix_pool::definition::AffixPoolDefinition;
use cypher_item::item::definition::ItemDefinition;

pub trait TableDisplay {
    /// What strings should be displayed for the header row?
    /// eg ID, Name, etc
    fn header_row_values() -> Vec<&'static str>;

    /// What values from this type should be displayed in the data row?
    /// eg, if header row was ID, Name: 1, "Test Item"
    fn data_row_values(&self) -> Vec<String>;
}

impl TableDisplay for AffixDefinition {
    fn header_row_values() -> Vec<&'static str> {
        vec!["Id", "Name", "Placement", "Tiers"]
    }

    fn data_row_values(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.name.clone(),
            self.placement.to_string(),
            self.tiers.len().to_string(),
        ]
    }
}

impl TableDisplay for AffixPoolDefinition {
    fn header_row_values() -> Vec<&'static str> {
        vec!["Id", "Name", "Members"]
    }

    fn data_row_values(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.name.clone(),
            self.members.len().to_string(),
        ]
    }
}

impl TableDisplay for ItemDefinition {
    fn header_row_values() -> Vec<&'static str> {
        vec!["Id", "Name", "Classification", "Pools"]
    }

    fn data_row_values(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.name.clone(),
            self.classification.to_string(),
            self.affix_pools.len().to_string(),
        ]
    }
}

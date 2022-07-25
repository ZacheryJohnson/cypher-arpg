use crate::data::DataDefinitionDatabase;
use std::collections::HashMap;

use super::{definition::AffixDefinition, AffixDefinitionId};

#[derive(Debug)]
pub struct AffixDefinitionDatabase {
    affixes: HashMap<AffixDefinitionId, AffixDefinition>,
}

impl DataDefinitionDatabase<AffixDefinition> for AffixDefinitionDatabase {
    fn initialize() -> Self {
        let affix_file = include_str!("../../data/affix.json");

        let definitions: Vec<AffixDefinition> = serde_json::de::from_str(affix_file).unwrap();

        let affixes = definitions
            .into_iter()
            .map(|affix| (affix.id, affix))
            .collect::<HashMap<_, _>>();

        AffixDefinitionDatabase { affixes }
    }

    fn get_definition_by_id(&self, id: &AffixDefinitionId) -> Option<&AffixDefinition> {
        self.affixes.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_affix_database() {
        let _ = AffixDefinitionDatabase::initialize();
    }
}

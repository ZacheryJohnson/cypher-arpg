use crate::data::{DataDefinition, DataDefinitionDatabase};
use std::{collections::HashMap, sync::Arc};

use super::{definition::AffixDefinition, AffixDefinitionId};

#[derive(Debug)]
pub struct AffixDefinitionDatabase {
    affixes: HashMap<AffixDefinitionId, Arc<AffixDefinition>>,
}

impl AffixDefinitionDatabase {
    pub fn initialize() -> Self {
        let affix_file = include_str!("../../data/affix.json");

        let definitions: Vec<Arc<AffixDefinition>> = serde_json::de::from_str(affix_file).unwrap();

        let affixes = definitions
            .into_iter()
            .map(|affix| (affix.id, affix))
            .collect::<HashMap<_, _>>();

        AffixDefinitionDatabase { affixes }
    }
}

impl DataDefinitionDatabase<AffixDefinition> for AffixDefinitionDatabase {
    /// Affixes are entirely self-contained (no references to other data)
    /// so we only check if there is at least 1 affix and all loaded affixes are valid.
    fn validate(&self) -> bool {
        !self.affixes.is_empty()
            && self
                .affixes
                .iter()
                .all(|(_id, affix_def)| affix_def.validate())
    }

    fn get_definition_by_id(&self, id: AffixDefinitionId) -> Option<Arc<AffixDefinition>> {
        self.affixes.get(&id).map(|arc| arc.to_owned())
    }

    fn definitions(&self) -> Vec<Arc<AffixDefinition>> {
        self.affixes.iter().map(|(_, def)| def.to_owned()).collect()
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

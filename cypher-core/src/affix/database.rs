use crate::data::{DataDefinition, DataDefinitionDatabase};
use std::{collections::HashMap, marker::PhantomData};

use super::{definition::AffixDefinition, AffixDefinitionId};

#[derive(Debug)]
pub struct AffixDefinitionDatabase<'db> {
    affixes: HashMap<AffixDefinitionId, AffixDefinition>,

    phantom: PhantomData<&'db ()>,
}

impl<'db> AffixDefinitionDatabase<'db> {
    pub fn initialize() -> Self {
        let affix_file = include_str!("../../data/affix.json");

        let definitions: Vec<AffixDefinition> = serde_json::de::from_str(affix_file).unwrap();

        let affixes = definitions
            .into_iter()
            .map(|affix| (affix.id, affix))
            .collect::<HashMap<_, _>>();

        AffixDefinitionDatabase {
            affixes,
            phantom: PhantomData,
        }
    }
}

impl<'db> DataDefinitionDatabase<'db, AffixDefinition> for AffixDefinitionDatabase<'db> {
    /// Affixes are entirely self-contained (no references to other data)
    /// so we only check if there is at least 1 affix and all loaded affixes are valid.
    fn validate(&'db self) -> bool {
        !self.affixes.is_empty()
            && self
                .affixes
                .iter()
                .all(|(_id, affix_def)| affix_def.validate())
    }

    fn get_definition_by_id(&'db self, id: AffixDefinitionId) -> Option<&'db AffixDefinition> {
        self.affixes.get(&id)
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

use crate::data::{DataDefinition, DataDefinitionDatabase};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::definition::{AffixDefinition, AffixDefinitionId};

#[derive(Debug)]
pub struct AffixDefinitionDatabase {
    affixes: HashMap<AffixDefinitionId, Arc<Mutex<AffixDefinition>>>,
}

impl AffixDefinitionDatabase {
    pub fn initialize() -> Self {
        let mut path = std::env::current_dir().unwrap();
        path.push("..");
        path.push("cypher-core");
        path.push("data");
        path.push("affix.json");

        Self::load_from(path.to_str().unwrap(), &())
    }
}

impl DataDefinitionDatabase<AffixDefinition> for AffixDefinitionDatabase {
    type DataDependencies = ();

    fn load_from<S: Into<String>>(path: S, _dependencies: &Self::DataDependencies) -> Self {
        let affix_file = String::from_utf8(std::fs::read(path.into()).unwrap()).unwrap();

        let definitions: Vec<AffixDefinition> =
            serde_json::de::from_str(affix_file.as_str()).unwrap();

        let affixes = definitions
            .into_iter()
            .map(|affix| (affix.id, Arc::from(Mutex::new(affix))))
            .collect::<HashMap<_, _>>();

        AffixDefinitionDatabase { affixes }
    }

    fn write_to<S: Into<String>>(&self, path: S) {
        let definition_clones = self
            .affixes
            .values()
            .map(|def| def.lock().unwrap().to_owned())
            .collect::<Vec<AffixDefinition>>();

        let serialized = serde_json::ser::to_string(&definition_clones)
            .expect("failed to serialize affix database");

        std::fs::write(path.into(), serialized).expect("failed to write serialized data to path");
    }

    /// Affixes are entirely self-contained (no references to other data)
    /// so we only check if there is at least 1 affix and all loaded affixes are valid.
    fn validate(&self) -> bool {
        !self.affixes.is_empty()
            && self
                .affixes
                .iter()
                .all(|(_id, affix_def)| affix_def.lock().unwrap().validate())
    }

    fn definition(&self, id: AffixDefinitionId) -> Option<Arc<Mutex<AffixDefinition>>> {
        self.affixes.get(&id).map(|arc| arc.to_owned())
    }

    fn definitions(&self) -> Vec<Arc<Mutex<AffixDefinition>>> {
        self.affixes.values().map(|def| def.to_owned()).collect()
    }

    fn add_definition(&mut self, definition: AffixDefinition) {
        self.affixes
            .insert(definition.id, Arc::new(Mutex::new(definition)));
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

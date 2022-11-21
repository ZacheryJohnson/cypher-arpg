use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use serde::de::DeserializeSeed;

use crate::data::{DataDefinition, DataDefinitionDatabase};

use crate::affix::database::AffixDefinitionDatabase;

use super::definition::AffixPoolDefinition;
use super::deserializer::AffixPoolDatabaseDeserializer;

pub type AffixPoolDefinitionId = u32;

pub struct AffixPoolDefinitionDatabase {
    affix_pools: HashMap<AffixPoolDefinitionId, Arc<Mutex<AffixPoolDefinition>>>,
}

impl DataDefinitionDatabase<AffixPoolDefinition> for AffixPoolDefinitionDatabase {
    fn validate(&self) -> bool {
        !self.affix_pools.is_empty()
            && self
                .affix_pools
                .iter()
                .all(|(_id, pool_def)| pool_def.lock().unwrap().validate())
    }

    fn definition(&self, id: AffixPoolDefinitionId) -> Option<Arc<Mutex<AffixPoolDefinition>>> {
        self.affix_pools.get(&id).map(|arc| arc.to_owned())
    }

    fn definitions(&self) -> Vec<Arc<Mutex<AffixPoolDefinition>>> {
        self.affix_pools
            .iter()
            .map(|(_, def)| def.to_owned())
            .collect()
    }
}

impl AffixPoolDefinitionDatabase {
    pub fn initialize(affix_db: Arc<Mutex<AffixDefinitionDatabase>>) -> Self {
        let mut path = std::env::current_dir().unwrap();
        path.push("..");
        path.push("cypher-core");
        path.push("data");
        path.push("affix_pool.json");

        Self::load_from(affix_db, path.to_str().unwrap())
    }

    pub fn load_from<S: Into<String>>(
        affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
        path: S,
    ) -> Self {
        let affix_file = String::from_utf8(std::fs::read(path.into()).unwrap()).unwrap();
        let deserializer = AffixPoolDatabaseDeserializer::new(affix_db);
        let definitions = deserializer
            .deserialize(&mut serde_json::Deserializer::from_str(affix_file.as_str()))
            .unwrap();

        let affix_pools = definitions
            .into_iter()
            .map(|pool| (pool.id, Arc::new(Mutex::new(pool))))
            .collect::<HashMap<_, _>>();

        AffixPoolDefinitionDatabase { affix_pools }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::AffixPoolDefinitionDatabase;
    use crate::affix::database::AffixDefinitionDatabase;

    #[test]
    fn init_affix_pool_database() {
        let affix_db = Arc::new(Mutex::new(AffixDefinitionDatabase::initialize()));
        let _ = AffixPoolDefinitionDatabase::initialize(affix_db);
    }
}

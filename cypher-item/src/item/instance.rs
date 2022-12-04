use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use cypher_core::affix::instance::AffixInstance;

use super::definition::ItemDefinition;

#[derive(Debug)]
pub struct ItemInstance {
    pub definition: Arc<Mutex<ItemDefinition>>,

    pub affixes: Vec<AffixInstance>,
}

impl Display for ItemInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let definition = self.definition.lock().unwrap();

        let mut buffer = String::new();
        for affix in &self.affixes {
            buffer += format!("{}\n", affix).as_str();
        }

        write!(
            f,
            "{}:\n\t{:?}\n\t{:?}",
            definition.name.as_str(),
            definition.classification,
            buffer.as_str()
        )
    }
}

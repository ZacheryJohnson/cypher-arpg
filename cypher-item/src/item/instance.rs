use serde::{Deserialize, Serialize, Serializer};
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use cypher_core::affix::instance::AffixInstance;

use super::definition::ItemDefinition;

#[derive(Debug, Serialize)]
pub struct ItemInstance {
    pub guid: String,

    #[serde(serialize_with = "serialize_definition")]
    #[serde(rename = "item_def_id")]
    pub definition: Arc<Mutex<ItemDefinition>>,

    pub affixes: Vec<AffixInstance>,
}

fn serialize_definition<S>(definition: &Arc<Mutex<ItemDefinition>>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u64(definition.lock().unwrap().id.into())
}

/// Rarity is a misnomer in our implementation, but is the standard for the genre
pub enum ItemInstanceRarityTier {
    /// An item with at most one prefix and one suffix
    Common,

    /// An item with at most two prefixes and two suffixes
    Uncommon,

    /// An item with at most three prefixes and three suffixes
    Rare,

    /// An item that possesses fixed affixes
    Fabled,
}

impl ItemInstance {
    pub fn rarity(&self) -> ItemInstanceRarityTier {
        if !self.definition.lock().unwrap().fixed_affixes.is_empty() {
            return ItemInstanceRarityTier::Fabled;
        }

        match self.affixes.len() {
            0..=2 => ItemInstanceRarityTier::Common,
            3..=4 => ItemInstanceRarityTier::Uncommon,
            5..=6 => ItemInstanceRarityTier::Rare,
            _ => panic!("abnormal affix count found"),
        }
    }
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
            "{}:\n\t{:?}\n\t{:?}\n",
            definition.name.as_str(),
            definition.classification,
            buffer.as_str()
        )
    }
}

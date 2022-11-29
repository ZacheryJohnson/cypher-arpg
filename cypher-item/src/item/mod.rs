use cypher_core::{
    affix::Affix, affix_pool::definition::AffixPoolDefinition, data::DataDefinition,
};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

pub type ItemDefinitionId = u64;

pub mod database;
pub mod deserializer;
pub mod generator;

#[derive(Clone, Copy, Deserialize, Debug, Serialize, PartialEq, Eq)]
pub enum ItemClassification {
    Invalid,
    Equippable(ItemEquipSlot),
    Currency,
}

impl std::fmt::Display for ItemClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Deserialize, Debug, Serialize, PartialEq, Eq)]
pub enum ItemEquipSlot {
    // These are all WIP! Expect these to change
    Head,
    LeftArm,
    RightArm,
    Body,
    Belt,
    Legs,
    Boots,
}

#[derive(Clone, Debug, Serialize)]
pub struct ItemDefinition {
    pub id: ItemDefinitionId,

    pub classification: ItemClassification,

    #[serde(serialize_with = "serialize_affix_pools_member")]
    pub affix_pools: Vec<Arc<Mutex<AffixPoolDefinition>>>,

    pub name: String,
}

fn serialize_affix_pools_member<S>(
    pools: &Vec<Arc<Mutex<AffixPoolDefinition>>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;

    let len = pools.len();
    let mut seq = s.serialize_seq(if len > 0 { Some(len) } else { None })?;
    for elem in pools {
        seq.serialize_element(&elem.lock().unwrap().id)?;
    }
    seq.end()
}

impl DataDefinition for ItemDefinition {
    type DefinitionTypeId = ItemDefinitionId;

    fn validate(&self) -> bool {
        self.classification != ItemClassification::Invalid
    }
}

#[derive(Debug)]
pub struct Item {
    pub definition: Arc<Mutex<ItemDefinition>>,

    pub affixes: Vec<Affix>,
}

impl Display for Item {
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

#[cfg(test)]
mod tests {
    use cypher_core::{
        affix::database::AffixDefinitionDatabase, affix_pool::database::AffixPoolDefinitionDatabase,
    };

    use super::{database::ItemDefinitionDatabase, *};

    #[test]
    fn init_item_database() {
        let affix_db = Arc::new(Mutex::new(AffixDefinitionDatabase::initialize()));
        let affix_pool_db = Arc::new(Mutex::new(AffixPoolDefinitionDatabase::initialize(
            affix_db.clone(),
        )));
        let _item_db = Arc::new(ItemDefinitionDatabase::initialize(affix_pool_db.clone()));
    }
}

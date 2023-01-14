use std::sync::{Arc, Mutex};

use cypher_core::{
    affix::definition::AffixDefinition, affix_pool::definition::AffixPoolDefinition,
    data::DataDefinition,
};
use serde::{Serialize, Serializer};

use super::classification::ItemClassification;

pub type ItemDefinitionId = u64;

#[derive(Clone, Debug, Serialize)]
pub struct ItemDefinition {
    pub id: ItemDefinitionId,

    pub classification: ItemClassification,

    #[serde(serialize_with = "serialize_affix_pools_member")]
    #[serde(rename = "affix_pools")]
    pub affix_pools: Vec<Arc<Mutex<AffixPoolDefinition>>>,

    #[serde(serialize_with = "serialize_fixed_affixes")]
    #[serde(rename = "fixed_affixes")]
    pub fixed_affixes: Vec<Arc<Mutex<AffixDefinition>>>,

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

fn serialize_fixed_affixes<S>(
    affix_defs: &Vec<Arc<Mutex<AffixDefinition>>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;

    let len = affix_defs.len();
    let mut seq = s.serialize_seq(if len > 0 { Some(len) } else { None })?;
    for elem in affix_defs {
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

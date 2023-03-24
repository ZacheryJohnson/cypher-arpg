use serde::{Serialize, Serializer};
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::stat::StatList;

use super::definition::{AffixDefinition, AffixTierId};

#[derive(Clone, Debug, Serialize)]
pub struct AffixInstance {
    #[serde(serialize_with = "serialize_definition")]
    #[serde(rename = "affix_def_id")]
    pub definition: Arc<Mutex<AffixDefinition>>,

    pub tier: AffixTierId,

    pub stats: StatList,
}

fn serialize_definition<S>(
    definition: &Arc<Mutex<AffixDefinition>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u64(definition.lock().unwrap().id.into())
}

impl Display for AffixInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "T{} {}: {}",
            self.tier,
            self.definition.lock().unwrap().name,
            self.stats
        )
    }
}

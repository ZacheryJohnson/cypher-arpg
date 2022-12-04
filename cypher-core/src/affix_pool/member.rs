use std::sync::{Arc, Mutex};

use serde::{Serialize, Serializer};

use crate::affix::definition::AffixDefinition;

#[derive(Clone, Debug, Serialize)]
pub struct AffixPoolMember {
    #[serde(serialize_with = "serialize_affix_definition_member")]
    #[serde(rename = "affix_id")]
    /// What affix will be generated when selected.
    pub affix_def: Arc<Mutex<AffixDefinition>>,

    /// Weight indicates how often this member will be chosen. A higher value = more common.
    pub weight: u64,
}

fn serialize_affix_definition_member<S>(
    definition: &Arc<Mutex<AffixDefinition>>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u64(definition.lock().unwrap().id.into())
}

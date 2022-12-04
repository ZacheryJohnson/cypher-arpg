use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::stat::StatList;

use super::definition::{AffixDefinition, AffixTierId};

#[derive(Debug)]
pub struct AffixInstance {
    pub definition: Arc<Mutex<AffixDefinition>>,

    pub tier: AffixTierId,

    pub stats: StatList,
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

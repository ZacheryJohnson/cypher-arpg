pub mod database;
pub mod definition;
pub mod placement;

use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::stat::StatList;

use self::definition::{AffixDefinition, AffixDefinitionId, AffixTierId};

#[derive(Debug)]
pub struct Affix {
    pub definition: Arc<Mutex<AffixDefinition>>,

    pub tier: AffixTierId,

    pub stats: StatList,
}

impl Display for Affix {
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

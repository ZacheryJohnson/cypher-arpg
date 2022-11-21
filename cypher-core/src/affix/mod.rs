pub mod database;
pub mod definition;
pub mod placement;

use std::sync::{Arc, Mutex};

use crate::stat::StatList;

use self::definition::{AffixDefinition, AffixDefinitionId, AffixTierId};

#[derive(Debug)]
pub struct Affix {
    pub definition: Arc<Mutex<AffixDefinition>>,

    pub tier: AffixTierId,

    pub stats: StatList,
}

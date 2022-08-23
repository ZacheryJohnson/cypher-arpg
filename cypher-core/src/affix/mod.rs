pub mod database;
pub mod definition;
pub mod placement;
pub mod pool;

mod deserializer;

use std::sync::Arc;

use crate::stat::StatList;

use self::definition::{AffixDefinition, AffixDefinitionId, AffixTierId};

#[derive(Debug)]
pub struct Affix {
    pub definition: Arc<AffixDefinition>,

    pub tier: AffixTierId,

    pub stats: StatList,
}

pub mod database;
pub mod definition;
pub mod placement;
pub mod pool;

use crate::stat::StatList;

use self::definition::{AffixDefinitionId, AffixTierId};

#[derive(Debug)]
pub struct Affix {
    pub definition: AffixDefinitionId,

    pub tier: AffixTierId,

    pub stats: StatList,
}

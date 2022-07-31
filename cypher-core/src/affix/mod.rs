pub mod database;
pub mod definition;
pub mod placement;
pub mod pool;

mod deserializer;

use crate::stat::StatList;

use self::definition::{AffixDefinition, AffixDefinitionId, AffixTierId};

#[derive(Debug)]
pub struct Affix<'db> {
    pub definition: &'db AffixDefinition,

    pub tier: AffixTierId,

    pub stats: StatList,
}

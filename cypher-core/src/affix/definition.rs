use super::placement::AffixPlacement;
use crate::{data::DataDefinition, stat::Stat};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type AffixDefinitionId = u32;
pub type AffixTierId = u16;

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct AffixDefinition {
    /// Opaque ID.
    pub id: AffixDefinitionId,

    pub placement: AffixPlacement,

    pub tiers: BTreeMap<AffixTierId, AffixDefinitionTier>,

    pub name: String,
}

impl DataDefinition for AffixDefinition {
    type DefinitionTypeId = AffixDefinitionId;

    fn validate(&self) -> bool {
        self.id > 0
            && self.placement != AffixPlacement::Invalid
            && !self.tiers.is_empty()
            && self.tiers.iter().all(|tier| tier.1.validate())
    }
}

impl AffixDefinition {
    /// Gets all [AffixDefinitionTier]s.
    pub fn tiers(&self) -> Vec<&AffixDefinitionTier> {
        self.tiers_to(&(self.tiers.len() as u16))
    }

    /// Gets all [AffixDefinitionTier]s of an [AffixDefinition], starting at [AffixTierId] 1 and ending at `upper_tier`, inclusive.
    pub fn tiers_to(&self, upper_tier: &AffixTierId) -> Vec<&AffixDefinitionTier> {
        let mut tiers = vec![];

        for (affix_tier_id, affix_tier) in &self.tiers {
            if affix_tier_id > upper_tier {
                break;
            }

            tiers.push(affix_tier);
        }

        tiers
    }
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct AffixDefinitionTier {
    pub tier: AffixTierId,

    pub stats: Vec<AffixDefinitionStat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_level_req: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision_places: Option<u32>,
}

impl AffixDefinitionTier {
    pub fn validate(&self) -> bool {
        self.tier > 0 && !self.stats.is_empty()
    }
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct AffixDefinitionStat {
    pub stat: Stat,

    pub lower_bound: f32,

    pub upper_bound: f32,
}

impl std::fmt::Display for AffixDefinitionStat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} [{}-{}]",
            self.stat, self.lower_bound, self.upper_bound
        )
    }
}

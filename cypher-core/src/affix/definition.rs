use super::{placement::AffixPlacement, Affix};
use crate::{
    data::DataDefinition,
    stat::{Stat, StatList, StatModifier},
};
use rand::{prelude::IteratorRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    sync::{Arc, Mutex},
};

pub type AffixDefinitionId = u32;
pub type AffixTierId = u16;

fn round_to(num: f32, decimal_places: u32) -> f32 {
    let factor = 10_u32.pow(decimal_places);
    (num * factor as f32).round() / factor as f32
}

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

    pub fn generate(
        definition: Arc<Mutex<AffixDefinition>>,
        criteria: &AffixGenerationCriteria,
    ) -> Option<Affix> {
        let def = definition.lock().unwrap();

        let (_id, tier) = def
            .tiers
            .iter()
            .filter(|(_id, tier)| {
                tier.item_level_req.unwrap_or(0) <= criteria.item_level.unwrap_or(0)
            })
            .choose(&mut rand::thread_rng())?;

        let stats = tier
            .stats
            .iter()
            .map(|stat| {
                StatModifier(
                    stat.stat,
                    round_to(
                        thread_rng().gen_range(stat.lower_bound..=stat.upper_bound),
                        tier.precision_places.unwrap_or(0),
                    ),
                )
            })
            .collect::<Vec<StatModifier>>();

        let stat_list = StatList::from(stats.as_slice());

        Some(Affix {
            definition: definition.clone(),
            tier: tier.tier,
            stats: stat_list,
        })
    }
}

#[derive(Default, Debug)]
/// Requirements when generating [Affix]es.
pub struct AffixGenerationCriteria {
    /// Which [AffixDefinition]s should be considered, if any.
    pub allowed_ids: Option<HashSet<AffixDefinitionId>>,

    /// Which [AffixDefinition]s should be excluded, if any.
    pub disallowed_ids: Option<HashSet<AffixDefinitionId>>,

    /// [AffixPlacement] to force, if any. If not Suffix or Prefix, can be either.
    pub placement: Option<AffixPlacement>,

    /// Maximum tier of [Affix] to generate, if any.
    pub maximum_tier: Option<AffixTierId>,

    /// Item level.
    pub item_level: Option<u8>,
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

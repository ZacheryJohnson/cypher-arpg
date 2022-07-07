use std::collections::{BTreeMap, HashMap, HashSet};

use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::stat::{Stat, StatList, StatModifier};

pub type AffixDefinitionId = u32;
pub type AffixTierId = u16;

#[derive(Clone, Deserialize, Debug, PartialEq, Serialize)]
pub enum AffixPlacement {
    Prefix,
    Suffix,
}

fn round_to(num: f32, decimal_places: u32) -> f32 {
    let factor = 10_u32.pow(decimal_places);
    (num * factor as f32).round() / factor as f32
}

#[derive(Debug)]
pub struct AffixDefinitionDatabase {
    affixes: HashMap<AffixDefinitionId, AffixDefinition>,
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

impl AffixDefinitionDatabase {
    pub fn initialize() -> AffixDefinitionDatabase {
        let affix_file = include_str!("../data/affix.json");

        let definitions: Vec<AffixDefinition> = serde_json::de::from_str(affix_file).unwrap();

        let affixes = definitions
            .into_iter()
            .map(|affix| (affix.id, affix))
            .collect::<HashMap<_, _>>();

        AffixDefinitionDatabase { affixes }
    }

    /// Generates an [Affix] given a set of criteria. May return `None` if criteria would exclude all loaded [AffixDefinition]s.
    pub fn generate(&self, criteria: &AffixGenerationCriteria) -> Option<Affix> {
        let affix_pool = self
            .affixes
            .iter()
            .filter(|affix| {
                criteria.allowed_ids.is_none()
                    || criteria.allowed_ids.as_ref().unwrap().contains(affix.0)
            })
            .filter(|affix| {
                criteria.disallowed_ids.is_none()
                    || !criteria.disallowed_ids.as_ref().unwrap().contains(affix.0)
            })
            .filter(|affix| {
                criteria.placement.is_none()
                    || *criteria.placement.as_ref().unwrap() == affix.1.placement
            })
            .map(|def| def.1)
            .collect::<Vec<&AffixDefinition>>();

        let affix_definition = affix_pool.choose(&mut rand::thread_rng())?;

        let tiers = {
            if criteria.maximum_tier.is_some() {
                affix_definition.tiers_to(&criteria.maximum_tier.unwrap())
            } else {
                affix_definition.tiers()
            }
        };

        let tier = tiers
            .iter()
            .filter(|tier| tier.item_level_req.unwrap_or(0) <= criteria.item_level.unwrap_or(0))
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
            definition: affix_definition.id,
            tier: tier.tier,
            stats: stat_list,
        })
    }

    pub fn get_definition_by_id(&self, id: &AffixDefinitionId) -> Option<&AffixDefinition> {
        self.affixes.get(id)
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AffixDefinition {
    /// Opaque ID.
    pub id: AffixDefinitionId,

    pub placement: AffixPlacement,

    pub tiers: BTreeMap<AffixTierId, AffixDefinitionTier>,
}

impl AffixDefinition {
    /// Gets all [AffixDefinitionTier]s.
    fn tiers(&self) -> Vec<&AffixDefinitionTier> {
        self.tiers_to(&(self.tiers.len() as u16))
    }

    /// Gets all [AffixDefinitionTier]s of an [AffixDefinition], starting at [AffixTierId] 1 and ending at `upper_tier`, inclusive.
    fn tiers_to(&self, upper_tier: &AffixTierId) -> Vec<&AffixDefinitionTier> {
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

#[derive(Deserialize, Debug, Serialize)]
pub struct AffixDefinitionTier {
    tier: AffixTierId,

    stats: Vec<AffixDefinitionStat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    item_level_req: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    precision_places: Option<u32>,
}

#[derive(Deserialize, Debug, Serialize)]
struct AffixDefinitionStat {
    stat: Stat,

    lower_bound: f32,

    upper_bound: f32,
}

#[derive(Debug)]
pub struct Affix {
    pub definition: AffixDefinitionId,

    pub tier: AffixTierId,

    pub stats: StatList,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_affix_database() {
        let affix_database = AffixDefinitionDatabase::initialize();

        let affix = affix_database.generate(&AffixGenerationCriteria::default());

        assert!(affix.is_some());
    }

    #[test]
    fn affix_criteria_only_contains_allowed_ids() {
        let affix_database = AffixDefinitionDatabase::initialize();

        let mut affixes = vec![];

        let mut allowed = HashSet::new();
        allowed.insert(1);

        let criteria = AffixGenerationCriteria {
            allowed_ids: Some(allowed.clone()),
            ..Default::default()
        };

        for _ in 0..=1000 {
            let affix = affix_database.generate(&criteria);
            affixes.push(affix);
        }

        for affix in affixes {
            assert!(allowed.contains(&affix.unwrap().definition));
        }
    }

    #[test]
    fn affix_criteria_does_not_contain_disallowed_ids() {
        let affix_database = AffixDefinitionDatabase::initialize();

        let mut affixes = vec![];

        let mut disallowed = HashSet::new();
        disallowed.insert(2);
        disallowed.insert(3);
        disallowed.insert(4);

        let criteria = AffixGenerationCriteria {
            disallowed_ids: Some(disallowed.clone()),
            ..Default::default()
        };

        for _ in 0..=1000 {
            let affix = affix_database.generate(&criteria);
            affixes.push(affix);
        }

        for affix in affixes {
            assert!(!disallowed.contains(&affix.unwrap().definition));
        }
    }

    #[test]
    fn affix_criteria_only_prefixes() {
        let affix_database = AffixDefinitionDatabase::initialize();

        let mut affixes = vec![];

        let criteria = AffixGenerationCriteria {
            placement: Some(AffixPlacement::Prefix),
            ..Default::default()
        };

        for _ in 0..=1000 {
            let affix = affix_database.generate(&criteria);
            affixes.push(affix);
        }

        for affix in affixes {
            let definition = affix_database
                .get_definition_by_id(&affix.unwrap().definition)
                .unwrap();
            assert!(definition.placement == AffixPlacement::Prefix);
        }
    }
}

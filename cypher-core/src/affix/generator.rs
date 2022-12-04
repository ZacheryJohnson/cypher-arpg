use rand::{seq::IteratorRandom, thread_rng, Rng};

use crate::{
    data::DataInstanceGenerator,
    stat::{StatList, StatModifier},
};

use super::{
    definition::{AffixDefinition, AffixTierId},
    instance::AffixInstance,
};

pub struct AffixGenerator {}

#[derive(Default)]
pub struct AffixGenerationCriteria {
    /// Maximum tier of [AffixInstance] to generate, if any.
    pub maximum_tier: Option<AffixTierId>,

    /// Item level.
    pub item_level: Option<u8>,
}

fn round_to(num: f32, decimal_places: u32) -> f32 {
    let factor = 10_u32.pow(decimal_places);
    (num * factor as f32).round() / factor as f32
}

impl DataInstanceGenerator<AffixDefinition, AffixInstance, AffixGenerationCriteria>
    for AffixGenerator
{
    type DataDependencies = ();

    fn generate(
        &self,
        definition: std::sync::Arc<std::sync::Mutex<AffixDefinition>>,
        criteria: &AffixGenerationCriteria,
        _databases: &Self::DataDependencies,
    ) -> Option<AffixInstance> {
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

        Some(AffixInstance {
            definition: definition.clone(),
            tier: tier.tier,
            stats: stat_list,
        })
    }
}

use cypher_core::stat::{Stat, StatList};

use crate::equipment::Equipment;

pub struct Character {
    pub stats: Vec<StatList>,

    pub equipment: Equipment,

    // TODO: refactor
    current_health: u32,
}

impl Character {
    pub fn new(stat_lists: Vec<StatList>) -> Character {
        let mut new_char = Character {
            stats: stat_lists,
            equipment: Equipment::new(),
            current_health: 0,
        };

        new_char.current_health = new_char
            .stats
            .iter()
            .map(|stat_list| *stat_list.get_stat(&Stat::Health).unwrap_or(&0.))
            .reduce(|acc, next| acc + next)
            .unwrap()
            .floor() as u32;

        new_char
    }

    /// Combines all player [StatList]s into a singular, cumulative [StatList].
    /// **This isn't performant**: creates a temporary. Should instead be implemented with iterators.
    pub fn stats(&self) -> StatList {
        let mut new_list = StatList::from(&[]);

        for stat_list in &self.stats {
            new_list.add_list(stat_list);
        }

        for stat_list in self.equipment.stats() {
            new_list.add_list(&stat_list);
        }

        new_list
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use std::{
        collections::BTreeMap,
        sync::{Arc, Mutex},
    };

    use cypher_core::{
        affix::{
            definition::{
                AffixDefinition, AffixDefinitionStat, AffixDefinitionTier, AffixDefinitionValue,
            },
            instance::AffixInstance,
            placement::AffixPlacement,
        },
        stat::{Stat, StatList, StatModifier},
    };
    use cypher_item::item::{
        classification::{ItemClassification, ItemEquipSlot},
        definition::ItemDefinition,
        instance::ItemInstance,
    };

    use super::*;

    #[test]
    fn new_sets_current_health_to_max() {
        let stat_list_1 = StatList::from(&[StatModifier(Stat::Health, 3.)]);
        let stat_list_2 = StatList::from(&[StatModifier(Stat::Health, 5.8)]);

        let character = Character::new(vec![stat_list_1, stat_list_2]);
        assert_eq!(character.current_health, 8);
    }

    #[test]
    fn can_combine_stat_lists() {
        let stat_list_1 = StatList::from(&[StatModifier(Stat::Resolve, 1.)]);
        let stat_list_2 = StatList::from(&[StatModifier(Stat::Resolve, 1.)]);

        let character = Character::new(vec![stat_list_1, stat_list_2]);

        let combined = character.stats();

        assert_eq!(*combined.get_stat(&Stat::Resolve).unwrap(), 2.);
    }

    #[test]
    fn negative_health_becomes_zero() {
        let stat_list_1 = StatList::from(&[StatModifier(Stat::Health, -1.)]);
        let stat_list_2 = StatList::from(&[StatModifier(Stat::Health, -1.)]);

        let character = Character::new(vec![stat_list_1, stat_list_2]);
        assert_eq!(character.current_health, 0);
    }

    #[test]
    fn equipment_stats_are_returned() {
        let mut affix_def_tiers: BTreeMap<u16, AffixDefinitionTier> = BTreeMap::new();
        affix_def_tiers.insert(
            1,
            AffixDefinitionTier {
                tier: 1,
                stats: vec![AffixDefinitionStat {
                    stat: Stat::Complexity,
                    value: AffixDefinitionValue::Range(1., 3.),
                }],
                item_level_req: None,
                precision_places: None,
            },
        );

        let head_item = ItemInstance {
            guid: uuid::Uuid::new_v4().to_string(),
            definition: Arc::new(Mutex::new(ItemDefinition {
                id: 1,
                classification: ItemClassification::Equippable(ItemEquipSlot::Head),
                affix_pools: vec![],
                name: String::from("test item"),
            })),
            affixes: vec![AffixInstance {
                definition: Arc::new(Mutex::new(AffixDefinition {
                    id: 1,
                    placement: AffixPlacement::Prefix,
                    tiers: affix_def_tiers,
                    name: String::from("test affix"),
                })),
                tier: 1,
                stats: StatList::from(&[StatModifier(Stat::Complexity, 2.)]),
            }],
        };

        // ZJ-TODO: characters don't have health by default but require it - that sucks
        let mut character = Character::new(vec![StatList::from(&[StatModifier(Stat::Health, 1.)])]);
        character.equipment.equip(head_item);

        let stats = character.stats();
        assert!(stats.mods().len() > 0);
    }
}

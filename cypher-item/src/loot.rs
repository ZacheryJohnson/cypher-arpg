use std::collections::HashMap;

use cypher_core::affix::AffixDefinitionDatabase;

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::Value;

use crate::item::{
    Item, ItemDefinition, ItemDefinitionCriteria, ItemDefinitionDatabase, ItemDefinitionId,
};

pub type LootPoolId = u32;

pub struct LootPoolDatabase {
    pools: HashMap<LootPoolId, LootPool>,
}

impl LootPoolDatabase {
    pub fn initialize() -> LootPoolDatabase {
        let loot_pool_file = include_str!("../data/loot_pool.json");

        let pools_database: Vec<LootPool> = serde_json::de::from_str(loot_pool_file).unwrap();

        let pools = pools_database
            .into_iter()
            .map(|pool| (pool.id, pool))
            .collect::<HashMap<_, _>>();

        LootPoolDatabase { pools }
    }
}

/// Loot refers to items acquired through random means.

/// A [LootPool] is a collection of [LootPoolMember]s. When generating items from a [LootPool],
/// the item will be chosen from one of the [LootPoolMember]s.
/// Enemies may have one or more [LootPool]s.
///
/// The lifetime `'item` is that of the [ItemDefinitionDatabase], as each [LootPoolMember] contains a reference
/// to an [ItemDefinition] within the [ItemDefinitionDatabase] instance.
#[derive(Debug)]
pub struct LootPool {
    id: LootPoolId,

    /// All [LootPoolMember]s that can drop as part of this [LootPool].
    members: Vec<LootPoolMember>,
}

impl<'de> Deserialize<'de> for LootPool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["id", "members"];

        enum Field {
            Id,
            Members,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                        formatter.write_str(format!("one of [{}]", FIELDS.join(", ")).as_str())
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        self.visit_str(std::str::from_utf8(v).unwrap())
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "id" => Ok(Field::Id),
                            "members" => Ok(Field::Members),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct LootPoolVisitor;

        impl<'de> Visitor<'de> for LootPoolVisitor {
            type Value = LootPool;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct LootPool")
            }

            fn visit_map<V>(self, mut map: V) -> Result<LootPool, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut loot_pool = LootPool {
                    id: 0,
                    members: Vec::new(),
                };

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => loot_pool.id = map.next_value()?,
                        Field::Members => {
                            // Please, fix this unholy mess
                            let value: Value = map.next_value()?;
                            match value {
                                Value::Array(values) => {
                                    for val in values {
                                        let mut member = LootPoolMember {
                                            weight: 0,
                                            item_id: 0,
                                        };
                                        match val {
                                            Value::Object(obj) => {
                                                for (k, v) in obj {
                                                    match v {
                                                        Value::Number(n) => {
                                                            if k == "weight" {
                                                                member.weight = n.as_u64().unwrap();
                                                            }
                                                            if k == "item_id" {
                                                                member.item_id =
                                                                    n.as_u64().unwrap();
                                                            }
                                                        }
                                                        _ => panic!("expected number!"),
                                                    }
                                                }
                                            }
                                            _ => panic!("expected object!"),
                                        }

                                        loot_pool.members.push(member);
                                    }
                                }
                                _ => panic!("expected array!"),
                            };
                        }
                    };
                }

                Ok(loot_pool)
            }
        }

        deserializer.deserialize_struct("LootPool", FIELDS, LootPoolVisitor)
    }
}

#[derive(Default)]
pub struct LootPoolCriteria {}

impl LootPool {
    pub fn generate(
        &self,
        item_database: &ItemDefinitionDatabase,
        affix_database: &AffixDefinitionDatabase,
        _criteria: &LootPoolCriteria,
    ) -> Item {
        let weights = self
            .members
            .iter()
            .map(|member| member.weight)
            .collect::<Vec<u64>>();

        let distribution = WeightedIndex::new(weights.as_slice()).unwrap();
        let mut rng = rand::thread_rng();
        let item_id = self.members[distribution.sample(&mut rng)].item_id;

        let definition = item_database.get_definition_by_id(&item_id).unwrap();

        definition.generate(affix_database, &ItemDefinitionCriteria::default())
    }
}

/// A [LootPoolMember] is a pairing of an item that can drop, in tandem with the chance that item will drop.
///
/// The lifetime `'item` is that of the [ItemDefinitionDatabase], as each [LootPoolMember] contains a reference
/// to an [ItemDefinition] within the [ItemDefinitionDatabase] instance.
#[derive(Debug)]
pub struct LootPoolMember {
    /// Weight indicates how often this member will be chosen. A higher value = more common.
    weight: u64,

    /// What item will be generated when selected.
    /// The affixes of the item are resolved when generating the item itself, outside of the purview of [LootPool]s.
    item_id: ItemDefinitionId,
}

impl<'de> Deserialize<'de> for LootPoolMember {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LootPoolMemberVisitor;

        impl<'de> Visitor<'de> for LootPoolMemberVisitor {
            type Value = LootPoolMember;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct LootPoolMember")
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let member = LootPoolMember {
                    weight: 0,
                    item_id: 0,
                };

                // TODO

                Ok(member)
            }
        }

        deserializer.deserialize_struct("LootPoolMember", &[], LootPoolMemberVisitor)
    }
}

impl LootPoolMember {
    fn new(weight: u64, item: &ItemDefinition) -> LootPoolMember {
        LootPoolMember {
            weight,
            item_id: item.id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ItemDefinitionDatabase;

    #[test]
    fn loot_pool_generation() {
        let affix_database = AffixDefinitionDatabase::initialize();
        let item_database = ItemDefinitionDatabase::initialize();
        let loot_pool_database = LootPoolDatabase::initialize();

        let loot_pool = loot_pool_database.pools.get(&1).unwrap();

        for _ in 0..10 {
            let item = loot_pool.generate(
                &item_database,
                &affix_database,
                &LootPoolCriteria::default(),
            );
            println!("{:?}", item);
        }
    }
}

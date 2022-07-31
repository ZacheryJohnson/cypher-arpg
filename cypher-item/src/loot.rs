use std::{collections::HashMap, marker::PhantomData};

use cypher_core::{
    affix::{database::AffixDefinitionDatabase, pool::AffixPoolDefinitionDatabase},
    data::{DataDefinition, DataDefinitionDatabase},
};

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;

use crate::item::{Item, ItemDefinition, ItemDefinitionCriteria, ItemDefinitionDatabase};

pub type LootPoolDefinitionId = u32;

pub struct LootPoolDatabase<'db> {
    pub pools: HashMap<LootPoolDefinitionId, LootPoolDefinition<'db>>,
}

impl<'db> LootPoolDatabase<'db> {
    pub fn initialize() -> Self {
        let loot_pool_file = include_str!("../data/loot_pool.json");

        let pools_database: Vec<LootPoolDefinition> =
            serde_json::de::from_str(loot_pool_file).unwrap();

        let pools = pools_database
            .into_iter()
            .map(|pool| (pool.id, pool))
            .collect::<HashMap<_, _>>();

        LootPoolDatabase { pools }
    }
}

impl<'db> DataDefinitionDatabase<'db, LootPoolDefinition<'db>> for LootPoolDatabase<'db> {
    fn get_definition_by_id(&self, id: LootPoolDefinitionId) -> Option<&LootPoolDefinition> {
        self.pools.get(&id)
    }
}

/// A [LootPoolDefinition] is a collection of [LootPoolMember]s. When generating items from a [LootPool],
/// the item will be chosen from one of the [LootPoolMember]s.
/// Enemies may have one or more [LootPoolDefinition]s.
#[derive(Debug, Serialize)]
pub struct LootPoolDefinition<'db> {
    id: LootPoolDefinitionId,

    /// All [LootPoolMember]s that can drop as part of this [LootPoolDefinition].
    members: Vec<LootPoolMember<'db>>,
}

impl<'de, 'db> Deserialize<'de> for LootPoolDefinition<'db> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["id", "members"];

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

        struct LootPoolVisitor<'db> {
            phantom: PhantomData<&'db ()>,
        }

        impl<'de, 'db> Visitor<'de> for LootPoolVisitor<'db> {
            type Value = LootPoolDefinition<'db>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct LootPool")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut loot_pool = LootPoolDefinition {
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
                                Value::Array(_values) => {
                                    panic!("Nice!");
                                    /*
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
                                    */
                                }
                                _ => panic!("expected array!"),
                            };
                        }
                    };
                }

                Ok(loot_pool)
            }
        }

        deserializer.deserialize_struct(
            "LootPool",
            FIELDS,
            LootPoolVisitor {
                phantom: PhantomData,
            },
        )
    }
}

#[derive(Default)]
pub struct LootPoolCriteria {}

impl<'db> DataDefinition for LootPoolDefinition<'db> {
    type DefinitionTypeId = LootPoolDefinitionId;

    fn validate(&self) -> bool {
        !self.members.is_empty()
    }
}

impl<'db> LootPoolDefinition<'db> {
    pub fn generate(
        &self,
        item_database: &'db ItemDefinitionDatabase,
        affix_database: &'db AffixDefinitionDatabase,
        affix_pool_database: &'db AffixPoolDefinitionDatabase,
        _criteria: &LootPoolCriteria,
    ) -> Item {
        let weights = self
            .members
            .iter()
            .map(|member| member.weight)
            .collect::<Vec<u64>>();

        let distribution = WeightedIndex::new(weights.as_slice()).unwrap();
        let mut rng = rand::thread_rng();
        let item_def = self.members[distribution.sample(&mut rng)].item_id;

        let definition = item_database.get_definition_by_id(item_def.id).unwrap();

        definition.generate(
            affix_database,
            affix_pool_database,
            &ItemDefinitionCriteria::default(),
        )
    }
}

/// A [LootPoolMember] is a pairing of an item that can drop, in tandem with the chance that item will drop.
///
/// The lifetime `'item` is that of the [ItemDefinitionDatabase], as each [LootPoolMember] contains a reference
/// to an [ItemDefinition] within the [ItemDefinitionDatabase] instance.
#[derive(Debug, Serialize)]
pub struct LootPoolMember<'db> {
    /// What item will be generated when selected.
    /// The affixes of the item are resolved when generating the item itself, outside of the purview of [LootPool]s.
    item_id: &'db ItemDefinition<'db>,

    /// Weight indicates how often this member will be chosen. A higher value = more common.
    weight: u64,
}

impl<'de, 'db> Deserialize<'de> for LootPoolMember<'db> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LootPoolMemberVisitor<'db> {
            phantom: PhantomData<&'db ()>,
        }

        impl<'de, 'db> Visitor<'de> for LootPoolMemberVisitor<'db> {
            type Value = LootPoolMember<'db>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct LootPoolMember")
            }
        }

        deserializer.deserialize_struct(
            "LootPoolMember",
            &[],
            LootPoolMemberVisitor {
                phantom: PhantomData,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::ItemDefinitionDatabase;

    #[test]
    #[ignore] // TODO: restore this test once affix pools are sorted
    fn loot_pool_generation() {
        let affix_database = AffixDefinitionDatabase::initialize();
        let affix_pool_database = AffixPoolDefinitionDatabase::initialize(&affix_database);
        let item_database = ItemDefinitionDatabase::initialize();
        let loot_pool_database = LootPoolDatabase::initialize();

        let loot_pool = loot_pool_database.pools.get(&1).unwrap();

        for _ in 0..10 {
            let item = loot_pool.generate(
                &item_database,
                &affix_database,
                &affix_pool_database,
                &LootPoolCriteria::default(),
            );
            println!("{:?}", item);
        }
    }
}

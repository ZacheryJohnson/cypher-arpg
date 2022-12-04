use std::sync::{Arc, Mutex};

use cypher_core::data::DataDefinitionDatabase;
use serde::{
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::item::{
    database::ItemDefinitionDatabase,
    definition::{ItemDefinition, ItemDefinitionId},
};

use super::{definition::LootPoolDefinition, member::LootPoolMember};

pub struct LootPoolDatabaseDeserializer {
    pub(super) item_db: Arc<Mutex<ItemDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for LootPoolDatabaseDeserializer {
    type Value = Vec<LootPoolDefinition>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LootPoolDatabaseVisitor {
            item_db: Arc<Mutex<ItemDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for LootPoolDatabaseVisitor {
            type Value = Vec<LootPoolDefinition>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ItemDefinition")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut definitions = vec![];

                while let Some(definition) = seq.next_element_seed(LootPoolDeserializer {
                    item_db: self.item_db.clone(),
                })? {
                    definitions.push(definition);
                }

                Ok(definitions)
            }
        }

        deserializer.deserialize_seq(LootPoolDatabaseVisitor {
            item_db: self.item_db,
        })
    }
}

struct LootPoolDeserializer {
    item_db: Arc<Mutex<ItemDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for LootPoolDeserializer {
    type Value = LootPoolDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["id", "name", "members"];

        enum Field {
            Id,
            Name,
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
                        E: serde::de::Error,
                    {
                        self.visit_str(std::str::from_utf8(v).unwrap())
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "id" => Ok(Field::Id),
                            "name" => Ok(Field::Name),
                            "members" => Ok(Field::Members),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct LootPoolVisitor {
            item_db: Arc<Mutex<ItemDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for LootPoolVisitor {
            type Value = LootPoolDefinition;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct LootPoolDefinition")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut loot_pool = LootPoolDefinition {
                    id: 0,
                    name: String::new(),
                    members: Vec::new(),
                };

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => loot_pool.id = map.next_value()?,
                        Field::Name => loot_pool.name = map.next_value()?,
                        Field::Members => {
                            loot_pool.members =
                                map.next_value_seed(LootPoolMemberPoolDeserializer {
                                    item_db: self.item_db.clone(),
                                })?
                        }
                    };
                }

                Ok(loot_pool)
            }
        }

        deserializer.deserialize_struct(
            "LootPoolDefinition",
            FIELDS,
            LootPoolVisitor {
                item_db: self.item_db,
            },
        )
    }
}

struct LootPoolMemberPoolDeserializer {
    item_db: Arc<Mutex<ItemDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for LootPoolMemberPoolDeserializer {
    type Value = Vec<LootPoolMember>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LootPoolMemberPoolVisitor {
            item_db: Arc<Mutex<ItemDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for LootPoolMemberPoolVisitor {
            type Value = Vec<LootPoolMember>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct LootPoolMember")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut members = vec![];

                while let Some(member) = seq.next_element_seed(LootPoolMemberDeserializer {
                    item_db: self.item_db.clone(),
                })? {
                    members.push(member);
                }

                Ok(members)
            }
        }

        deserializer.deserialize_seq(LootPoolMemberPoolVisitor {
            item_db: self.item_db,
        })
    }
}

struct LootPoolMemberDeserializer {
    item_db: Arc<Mutex<ItemDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for LootPoolMemberDeserializer {
    type Value = LootPoolMember;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["item_id", "weight"];

        enum Field {
            ItemId,
            Weight,
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
                        E: serde::de::Error,
                    {
                        self.visit_str(std::str::from_utf8(v).unwrap())
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "item_id" => Ok(Field::ItemId),
                            "weight" => Ok(Field::Weight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct LootPoolMemberVisitor {
            item_db: Arc<Mutex<ItemDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for LootPoolMemberVisitor {
            type Value = LootPoolMember;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct LootPoolMember")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut item_def: Option<Arc<Mutex<ItemDefinition>>> = None;
                let mut weight = 0;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ItemId => {
                            let item_id = map.next_value::<ItemDefinitionId>()?;

                            item_def = self.item_db.lock().unwrap().definition(item_id)
                        }
                        Field::Weight => weight = map.next_value()?,
                    };
                }

                Ok(LootPoolMember {
                    item_def: item_def.unwrap(),
                    weight,
                })
            }
        }

        deserializer.deserialize_struct(
            "LootPoolMember",
            FIELDS,
            LootPoolMemberVisitor {
                item_db: self.item_db,
            },
        )
    }
}

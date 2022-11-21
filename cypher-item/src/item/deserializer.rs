use std::sync::{Arc, Mutex};

use cypher_core::{
    affix_pool::database::{AffixPoolDefinitionDatabase, AffixPoolDefinitionId},
    data::DataDefinitionDatabase,
};
use serde::{
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::item::ItemClassification;

use super::ItemDefinition;

pub struct ItemDefinitionDatabaseDeserializer {
    pub(super) affix_pool_db: Arc<Mutex<AffixPoolDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for ItemDefinitionDatabaseDeserializer {
    type Value = Vec<ItemDefinition>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ItemDefinitionDatabaseVisitor {
            affix_pool_db: Arc<Mutex<AffixPoolDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for ItemDefinitionDatabaseVisitor {
            type Value = Vec<ItemDefinition>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ItemDefinitionDatabase")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut definitions = vec![];

                while let Some(definition) = seq.next_element_seed(ItemDefinitionDeserializer {
                    affix_pool_db: self.affix_pool_db.clone(),
                })? {
                    definitions.push(definition);
                }

                Ok(definitions)
            }
        }

        deserializer.deserialize_seq(ItemDefinitionDatabaseVisitor {
            affix_pool_db: self.affix_pool_db,
        })
    }
}

struct ItemDefinitionDeserializer {
    affix_pool_db: Arc<Mutex<AffixPoolDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for ItemDefinitionDeserializer {
    type Value = ItemDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["id", "classification", "affix_pools", "name"];

        enum Field {
            Id,
            Classification,
            AffixPools,
            Name,
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
                            "classification" => Ok(Field::Classification),
                            "affix_pools" => Ok(Field::AffixPools),
                            "name" => Ok(Field::Name),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ItemDefinitionVisitor {
            affix_pool_db: Arc<Mutex<AffixPoolDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for ItemDefinitionVisitor {
            type Value = ItemDefinition;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ItemDefinition")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut item_def = ItemDefinition {
                    id: 0,
                    classification: ItemClassification::Invalid,
                    affix_pools: vec![],
                    name: String::new(),
                };

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => item_def.id = map.next_value()?,
                        Field::Classification => item_def.classification = map.next_value()?,
                        Field::AffixPools => {
                            let mut affix_pools = vec![];

                            let affix_pool_ids: Vec<AffixPoolDefinitionId> = map.next_value()?;
                            for affix_pool_id in affix_pool_ids {
                                affix_pools.push(
                                    self.affix_pool_db
                                        .lock()
                                        .unwrap()
                                        .definition(affix_pool_id)
                                        .unwrap(),
                                );
                            }

                            item_def.affix_pools = affix_pools;
                        }
                        Field::Name => item_def.name = map.next_value()?,
                    };
                }

                Ok(item_def)
            }
        }

        deserializer.deserialize_struct(
            "ItemDefinition",
            FIELDS,
            ItemDefinitionVisitor {
                affix_pool_db: self.affix_pool_db,
            },
        )
    }
}

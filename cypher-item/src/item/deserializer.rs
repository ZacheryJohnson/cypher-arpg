use std::sync::{Arc, Mutex};

use cypher_core::affix::deserializer::AffixInstanceVecDeserializer;
use cypher_core::{
    affix::{database::AffixDefinitionDatabase, definition::AffixDefinitionId},
    affix_pool::database::{AffixPoolDefinitionDatabase, AffixPoolDefinitionId},
    data::DataDefinitionDatabase,
};
use serde::{
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::item::classification::ItemClassification;
use crate::item::database::ItemDefinitionDatabase;
use crate::item::instance::ItemInstance;

use super::definition::ItemDefinition;

pub struct ItemDefinitionDatabaseDeserializer {
    pub(super) affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
    pub(super) affix_pool_db: Arc<Mutex<AffixPoolDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for ItemDefinitionDatabaseDeserializer {
    type Value = Vec<ItemDefinition>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ItemDefinitionDatabaseVisitor {
            affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
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
                    affix_db: self.affix_db.clone(),
                    affix_pool_db: self.affix_pool_db.clone(),
                })? {
                    definitions.push(definition);
                }

                Ok(definitions)
            }
        }

        deserializer.deserialize_seq(ItemDefinitionDatabaseVisitor {
            affix_db: self.affix_db,
            affix_pool_db: self.affix_pool_db,
        })
    }
}

struct ItemDefinitionDeserializer {
    affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
    affix_pool_db: Arc<Mutex<AffixPoolDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for ItemDefinitionDeserializer {
    type Value = ItemDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "classification",
            "affix_pools",
            "fixed_affixes",
            "name",
        ];

        enum Field {
            Id,
            Classification,
            AffixPools,
            FixedAffixes,
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
                            "fixed_affixes" => Ok(Field::FixedAffixes),
                            "name" => Ok(Field::Name),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ItemDefinitionVisitor {
            affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
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
                    fixed_affixes: vec![],
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
                        Field::FixedAffixes => {
                            let mut fixed_affixes = vec![];

                            let fixed_affix_ids: Vec<AffixDefinitionId> = map.next_value()?;
                            for fixed_affix_id in fixed_affix_ids {
                                fixed_affixes.push(
                                    self.affix_db
                                        .lock()
                                        .unwrap()
                                        .definition(fixed_affix_id)
                                        .unwrap(),
                                );
                            }

                            item_def.fixed_affixes = fixed_affixes;
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
                affix_db: self.affix_db,
                affix_pool_db: self.affix_pool_db,
            },
        )
    }
}

pub struct ItemInstanceDeserializer {
    pub affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
    pub item_db: Arc<Mutex<ItemDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for ItemInstanceDeserializer {
    type Value = ItemInstance;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["guid", "item_def_id", "affixes"];

        enum Field {
            Guid,
            ItemDefId,
            Affixes,
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
                            "guid" => Ok(Field::Guid),
                            "item_def_id" => Ok(Field::ItemDefId),
                            "affixes" => Ok(Field::Affixes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ItemInstanceVisitor {
            affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
            item_db: Arc<Mutex<ItemDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for ItemInstanceVisitor {
            type Value = ItemInstance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ItemInstance")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut guid = String::new();
                let mut maybe_definition: Option<Arc<Mutex<ItemDefinition>>> = None;
                let mut affixes = vec![];

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Guid => guid = map.next_value()?,
                        Field::ItemDefId => {
                            let item_def =
                                self.item_db.lock().unwrap().definition(map.next_value()?);
                            maybe_definition = item_def;
                        }
                        Field::Affixes => {
                            affixes = map.next_value_seed(AffixInstanceVecDeserializer {
                                affix_db: self.affix_db.clone(),
                            })?
                        }
                    };
                }

                Ok(ItemInstance {
                    guid,
                    definition: maybe_definition.unwrap(),
                    affixes,
                })
            }
        }

        deserializer.deserialize_struct(
            "ItemInstance",
            FIELDS,
            ItemInstanceVisitor {
                affix_db: self.affix_db,
                item_db: self.item_db,
            },
        )
    }
}

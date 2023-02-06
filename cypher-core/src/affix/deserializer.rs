use crate::affix::database::AffixDefinitionDatabase;
use crate::affix::definition::{AffixDefinition, AffixDefinitionId, AffixTierId};
use crate::affix::instance::AffixInstance;
use crate::data::DataDefinitionDatabase;
use crate::stat::StatList;
use serde::de::{DeserializeSeed, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::sync::{Arc, Mutex};

struct AffixInstanceDeserializer {
    affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for AffixInstanceDeserializer {
    type Value = AffixInstance;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["affix_def_id", "tier", "stats"];

        enum Field {
            AffixDefId,
            Tier,
            Stats,
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
                            "affix_def_id" => Ok(Field::AffixDefId),
                            "tier" => Ok(Field::Tier),
                            "stats" => Ok(Field::Stats),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct AffixInstanceVisitor {
            affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for AffixInstanceVisitor {
            type Value = AffixInstance;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AffixInstance")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut maybe_def: Option<Arc<Mutex<AffixDefinition>>> = None;
                let mut tier: Option<AffixTierId> = None;
                let mut stats: Option<StatList> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::AffixDefId => {
                            let affix_id: AffixDefinitionId = map.next_value()?;
                            maybe_def = self.affix_db.lock().unwrap().definition(affix_id);
                        }
                        Field::Tier => tier = Some(map.next_value()?),
                        Field::Stats => stats = Some(map.next_value()?),
                    };
                }

                Ok(AffixInstance {
                    definition: maybe_def.unwrap(),
                    tier: tier.unwrap(),
                    stats: stats.unwrap(),
                })
            }
        }

        deserializer.deserialize_struct(
            "AffixInstance",
            FIELDS,
            AffixInstanceVisitor {
                affix_db: self.affix_db,
            },
        )
    }
}

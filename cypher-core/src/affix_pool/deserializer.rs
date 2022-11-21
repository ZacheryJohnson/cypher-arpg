use std::sync::{Arc, Mutex};

use serde::{
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::{
    affix::database::AffixDefinitionDatabase,
    affix::definition::{AffixDefinition, AffixDefinitionId},
    affix_pool::member::AffixPoolMember,
    data::DataDefinitionDatabase,
};

use super::definition::AffixPoolDefinition;

/// Deserializes an Affix Pool database.
pub struct AffixPoolDatabaseDeserializer {
    affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
}

impl AffixPoolDatabaseDeserializer {
    pub fn new(affix_db: Arc<Mutex<AffixDefinitionDatabase>>) -> Self {
        Self { affix_db }
    }
}

impl<'de> DeserializeSeed<'de> for AffixPoolDatabaseDeserializer {
    type Value = Vec<AffixPoolDefinition>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AffixPoolDatabaseVisitor {
            affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for AffixPoolDatabaseVisitor {
            type Value = Vec<AffixPoolDefinition>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("AffixPoolDefinitionDatabase")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut definitions = vec![];

                while let Some(definition) =
                    seq.next_element_seed(AffixPoolDefinitionDeserializer {
                        affix_db: self.affix_db.clone(),
                    })?
                {
                    definitions.push(definition);
                }

                Ok(definitions)
            }
        }

        deserializer.deserialize_seq(AffixPoolDatabaseVisitor {
            affix_db: self.affix_db,
        })
    }
}

struct AffixPoolDefinitionDeserializer {
    affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for AffixPoolDefinitionDeserializer {
    type Value = AffixPoolDefinition;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["id", "members", "name"];

        enum Field {
            Id,
            Members,
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
                            "members" => Ok(Field::Members),
                            "name" => Ok(Field::Name),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct AffixPoolDefinitionVisitor {
            affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for AffixPoolDefinitionVisitor {
            type Value = AffixPoolDefinition;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AffixPoolDefinition")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut affix_pool_def = AffixPoolDefinition {
                    id: 0,
                    members: Vec::new(),
                    name: String::new(),
                };

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => affix_pool_def.id = map.next_value()?,
                        Field::Members => {
                            affix_pool_def.members =
                                map.next_value_seed(AffixPoolMemberPoolDeserializer {
                                    affix_db: self.affix_db.clone(),
                                })?
                        }
                        Field::Name => affix_pool_def.name = map.next_value()?,
                    };
                }

                Ok(affix_pool_def)
            }
        }

        deserializer.deserialize_struct(
            "AffixPoolDefinition",
            FIELDS,
            AffixPoolDefinitionVisitor {
                affix_db: self.affix_db,
            },
        )
    }
}

/// Deserializes an array of [AffixPoolMember]s. Internal implementation detail
struct AffixPoolMemberPoolDeserializer {
    affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for AffixPoolMemberPoolDeserializer {
    type Value = Vec<AffixPoolMember>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AffixPoolMemberPoolVisitor {
            affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for AffixPoolMemberPoolVisitor {
            type Value = Vec<AffixPoolMember>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AffixPoolMember")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut members = vec![];

                while let Some(member) = seq.next_element_seed(AffixPoolMemberDeserializer {
                    affix_db: self.affix_db.clone(),
                })? {
                    members.push(member);
                }

                Ok(members)
            }
        }

        deserializer.deserialize_seq(AffixPoolMemberPoolVisitor {
            affix_db: self.affix_db,
        })
    }
}

/// Deserializes an [AffixPoolMember].
struct AffixPoolMemberDeserializer {
    affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
}

impl<'de> DeserializeSeed<'de> for AffixPoolMemberDeserializer {
    type Value = AffixPoolMember;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["affix_id", "weight"];

        enum Field {
            AffixId,
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
                            "affix_id" => Ok(Field::AffixId),
                            "weight" => Ok(Field::Weight),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct AffixPoolMemberVisitor {
            affix_db: Arc<Mutex<AffixDefinitionDatabase>>,
        }

        impl<'de> Visitor<'de> for AffixPoolMemberVisitor {
            type Value = AffixPoolMember;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AffixPoolMember")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut affix_def: Option<Arc<Mutex<AffixDefinition>>> = None;
                let mut weight = 0;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::AffixId => {
                            let affix_id = map.next_value::<AffixDefinitionId>()?;
                            affix_def = self.affix_db.lock().unwrap().definition(affix_id);
                        }
                        Field::Weight => weight = map.next_value()?,
                    };
                }

                Ok(AffixPoolMember {
                    affix_def: affix_def.unwrap(),
                    weight,
                })
            }
        }

        deserializer.deserialize_struct(
            "AffixPoolMember",
            FIELDS,
            AffixPoolMemberVisitor {
                affix_db: self.affix_db,
            },
        )
    }
}

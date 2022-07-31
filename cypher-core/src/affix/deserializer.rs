use serde::{
    de::{DeserializeSeed, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::{affix::definition::AffixDefinition, data::DataDefinitionDatabase};

use super::{
    database::AffixDefinitionDatabase,
    pool::{AffixPoolDefinition, AffixPoolMember},
};

/// Deserializes an Affix Pool database.
pub struct AffixPoolDatabaseDeserializer<'db> {
    affix_db: &'db AffixDefinitionDatabase<'db>,
}

impl<'db> AffixPoolDatabaseDeserializer<'db> {
    pub fn new(affix_db: &'db AffixDefinitionDatabase) -> Self {
        Self { affix_db }
    }
}

impl<'de, 'db> DeserializeSeed<'de> for AffixPoolDatabaseDeserializer<'db> {
    type Value = Vec<AffixPoolDefinition<'db>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AffixPoolDatabaseVisitor<'db> {
            affix_db: &'db AffixDefinitionDatabase<'db>,
        }

        impl<'de, 'db> Visitor<'de> for AffixPoolDatabaseVisitor<'db> {
            type Value = Vec<AffixPoolDefinition<'db>>;

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
                        affix_db: self.affix_db,
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

struct AffixPoolDefinitionDeserializer<'db> {
    affix_db: &'db AffixDefinitionDatabase<'db>,
}

impl<'de, 'db> DeserializeSeed<'de> for AffixPoolDefinitionDeserializer<'db> {
    type Value = AffixPoolDefinition<'db>;

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

        struct AffixPoolDefinitionVisitor<'db> {
            affix_db: &'db AffixDefinitionDatabase<'db>,
        }

        impl<'de, 'db> Visitor<'de> for AffixPoolDefinitionVisitor<'db> {
            type Value = AffixPoolDefinition<'db>;

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
                                    affix_db: self.affix_db,
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
struct AffixPoolMemberPoolDeserializer<'db> {
    affix_db: &'db AffixDefinitionDatabase<'db>,
}

impl<'de, 'db> DeserializeSeed<'de> for AffixPoolMemberPoolDeserializer<'db> {
    type Value = Vec<AffixPoolMember<'db>>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AffixPoolMemberPoolVisitor<'db> {
            affix_db: &'db AffixDefinitionDatabase<'db>,
        }

        impl<'de, 'db> Visitor<'de> for AffixPoolMemberPoolVisitor<'db> {
            type Value = Vec<AffixPoolMember<'db>>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AffixPoolMember")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut members = vec![];

                while let Some(member) = seq.next_element_seed(AffixPoolMemberDeserializer {
                    affix_db: self.affix_db,
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
struct AffixPoolMemberDeserializer<'db> {
    affix_db: &'db AffixDefinitionDatabase<'db>,
}

impl<'de, 'db> DeserializeSeed<'de> for AffixPoolMemberDeserializer<'db> {
    type Value = AffixPoolMember<'db>;

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

        struct AffixPoolMemberVisitor<'db> {
            affix_db: &'db AffixDefinitionDatabase<'db>,
        }

        impl<'de, 'db> Visitor<'de> for AffixPoolMemberVisitor<'db> {
            type Value = AffixPoolMember<'db>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct AffixPoolMember")
            }

            fn visit_map<V>(self, mut map: V) -> Result<AffixPoolMember<'db>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut affix_def: Option<&'db AffixDefinition> = None;
                let mut weight = 0;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::AffixId => {
                            affix_def = Some(
                                self.affix_db
                                    .get_definition_by_id(map.next_value::<u32>()?)
                                    .unwrap(),
                            )
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

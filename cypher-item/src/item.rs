use cypher_core::{
    affix::{
        database::AffixDefinitionDatabase,
        definition::AffixGenerationCriteria,
        pool::{AffixPoolDefinition, AffixPoolDefinitionDatabase},
        Affix,
    },
    data::{DataDefinition, DataDefinitionDatabase},
};
use rand::{distributions::WeightedIndex, prelude::*};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

pub type ItemDefinitionId = u64;

pub struct ItemDefinitionDatabase<'db> {
    items: HashMap<ItemDefinitionId, ItemDefinition<'db>>,
}

impl<'db> ItemDefinitionDatabase<'db> {
    pub fn initialize() -> Self {
        let item_file = include_str!("../data/item.json");

        let definitions: Vec<ItemDefinition> = serde_json::de::from_str(item_file).unwrap();

        let items = definitions
            .into_iter()
            .map(|item| (item.id, item))
            .collect::<HashMap<_, _>>();

        ItemDefinitionDatabase { items }
    }
}

impl<'db> DataDefinitionDatabase<'db, ItemDefinition<'db>> for ItemDefinitionDatabase<'db> {
    fn get_definition_by_id(&self, id: ItemDefinitionId) -> Option<&ItemDefinition> {
        self.items.get(&id)
    }
}

#[derive(Clone)]
pub struct ItemDefinitionCriteria {
    /// How many affixes can this item roll? Stored as tuples, where tuple.0 = number of affixes possible, and tuple.1 = affix weight
    pub affix_count_weighting: Vec<(u8 /* count */, u64 /* weight */)>,
}

impl Default for ItemDefinitionCriteria {
    fn default() -> Self {
        Self {
            affix_count_weighting: vec![(1, 500), (2, 300), (3, 100), (4, 20), (5, 5), (6, 1)],
        }
    }
}

#[derive(Clone, Copy, Deserialize, Debug, Serialize, PartialEq)]
pub enum ItemClassification {
    Invalid,
    Equippable(ItemEquipSlot),
    Currency,
}

#[derive(Clone, Copy, Deserialize, Debug, Serialize, PartialEq)]
pub enum ItemEquipSlot {
    // These are all WIP! Expect these to change
    Head,
    LeftArm,
    RightArm,
    Body,
    Belt,
    Legs,
    Boots,
}

#[derive(Debug, Serialize)]
pub struct ItemDefinition<'db> {
    pub id: ItemDefinitionId,

    pub classification: ItemClassification,

    pub affix_pools: Option<Vec<AffixPoolDefinition<'db>>>,

    name: String,
}

impl DataDefinition for ItemDefinition<'_> {
    type DefinitionTypeId = ItemDefinitionId;

    fn validate(&self) -> bool {
        self.classification != ItemClassification::Invalid
            && (self.affix_pools.is_none() || !self.affix_pools.as_ref().unwrap().is_empty())
    }
}

impl<'db> ItemDefinition<'db> {
    pub fn generate(
        &self,
        affix_database: &'db AffixDefinitionDatabase,
        affix_pool_database: &'db AffixPoolDefinitionDatabase,
        criteria: &ItemDefinitionCriteria,
    ) -> Item {
        let mut affix_criteria = AffixGenerationCriteria::default();

        let distribution = WeightedIndex::new(
            criteria
                .affix_count_weighting
                .iter()
                .map(|pair| pair.1)
                .collect::<Vec<u64>>()
                .as_slice(),
        )
        .unwrap();

        let mut rng = rand::thread_rng();
        let affix_count = criteria.affix_count_weighting[distribution.sample(&mut rng)].0;

        let mut affix_pool_members = vec![];
        for pool_def in self.affix_pools.as_ref().unwrap_or(&vec![]) {
            let affix_pool = affix_pool_database
                .get_definition_by_id(pool_def.id)
                .unwrap(); // TODO: remove unwrap
            for member in &affix_pool.members {
                affix_pool_members.push(member.to_owned());
            }
        }

        let pool = AffixPoolDefinition::from_members(affix_pool_members);

        let mut affixes = vec![];
        for _ in 0..affix_count {
            let affix = pool.generate(affix_database, &affix_criteria);
            if affix.is_none() {
                continue;
            }

            let affix_definition = affix_database
                .get_definition_by_id(affix.as_ref().unwrap().definition.id)
                .unwrap();
            if affix_criteria.disallowed_ids.is_none() {
                affix_criteria.disallowed_ids = Some(HashSet::new());
            }
            affix_criteria
                .disallowed_ids
                .as_mut()
                .unwrap()
                .insert(affix_definition.id);

            // TODO: handle None
            affixes.push(affix.unwrap());
        }

        Item {
            definition_id: self.id,
            classification: self.classification,
            affixes,
        }
    }
}

impl<'de, 'db> Deserialize<'de> for ItemDefinition<'db> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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

        struct ItemDefinitionVisitor<'db> {
            phantom: PhantomData<&'db ()>,
        }

        impl<'de, 'db> Visitor<'de> for ItemDefinitionVisitor<'db> {
            type Value = ItemDefinition<'db>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ItemDefinition")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ItemDefinition<'db>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut item_def = ItemDefinition {
                    id: 0,
                    classification: ItemClassification::Invalid,
                    affix_pools: None,
                    name: String::new(),
                };

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => item_def.id = map.next_value()?,
                        Field::Classification => item_def.classification = map.next_value()?,
                        Field::AffixPools => {
                            let _pools: Vec<u32> = map.next_value()?;
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
                phantom: PhantomData,
            },
        )
    }
}

#[derive(Debug)]
pub struct Item<'db> {
    pub definition_id: ItemDefinitionId,

    pub classification: ItemClassification,

    pub affixes: Vec<Affix<'db>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_item_database() {
        let _ = ItemDefinitionDatabase::initialize();
    }

    #[test]
    fn loot_generation() {
        let item_database = ItemDefinitionDatabase::initialize();
        let affix_database = AffixDefinitionDatabase::initialize();
        let affix_pool_database = AffixPoolDefinitionDatabase::initialize(&affix_database);

        let definition = *item_database
            .items
            .iter()
            .map(|item| item.1.to_owned())
            .collect::<Vec<&ItemDefinition>>()
            .choose(&mut rand::thread_rng())
            .unwrap();

        let criteria = ItemDefinitionCriteria::default();

        for _ in 0..10 {
            let _ = definition.generate(&affix_database, &affix_pool_database, &criteria);
        }
    }
}

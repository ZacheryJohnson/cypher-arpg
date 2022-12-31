use std::fmt::Display;

use cypher_core::stat::StatList;
use cypher_item::item::{
    classification::{ItemClassification::Equippable, ItemEquipSlot},
    instance::ItemInstance,
};

#[derive(Default)]
pub struct Equipment {
    pub head: Option<ItemInstance>,
    pub left_arm: Option<ItemInstance>,
    pub right_arm: Option<ItemInstance>,
    pub body: Option<ItemInstance>,
    pub belt: Option<ItemInstance>,
    pub legs: Option<ItemInstance>,
    pub boots: Option<ItemInstance>,
}

#[derive(Debug)]
pub enum EquipError {
    NotEquipable,
}

impl Equipment {
    pub fn equip(
        &mut self,
        equipable_item: ItemInstance,
    ) -> Result<Option<ItemInstance>, EquipError> {
        let maybe_slot = match equipable_item.definition.lock().unwrap().classification {
            Equippable(slot) => Some(slot),
            _ => None,
        };

        if maybe_slot.is_none() {
            return Err(EquipError::NotEquipable);
        }

        Ok(match maybe_slot.unwrap() {
            ItemEquipSlot::Head => std::mem::replace(&mut self.head, Some(equipable_item)),
            ItemEquipSlot::LeftArm => std::mem::replace(&mut self.left_arm, Some(equipable_item)),
            ItemEquipSlot::RightArm => std::mem::replace(&mut self.right_arm, Some(equipable_item)),
            ItemEquipSlot::Body => std::mem::replace(&mut self.body, Some(equipable_item)),
            ItemEquipSlot::Belt => std::mem::replace(&mut self.belt, Some(equipable_item)),
            ItemEquipSlot::Legs => std::mem::replace(&mut self.legs, Some(equipable_item)),
            ItemEquipSlot::Boots => std::mem::replace(&mut self.boots, Some(equipable_item)),
        })
    }

    pub fn new() -> Equipment {
        Equipment {
            ..Default::default()
        }
    }

    pub fn stats(&self) -> Vec<StatList> {
        let mut stat_list = vec![];

        let items = vec![
            &self.head,
            &self.left_arm,
            &self.right_arm,
            &self.body,
            &self.belt,
            &self.legs,
            &self.boots,
        ];

        for item in items.into_iter().flatten() {
            for affix in &item.affixes {
                stat_list.push(affix.stats.clone());
            }
        }

        stat_list
    }
}

impl Display for Equipment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items = vec![
            &self.head,
            &self.left_arm,
            &self.right_arm,
            &self.body,
            &self.belt,
            &self.legs,
            &self.boots,
        ];

        for item in items.into_iter().flatten() {
            write!(f, "{}", item)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use uuid;

    use cypher_item::item::{
        classification::{ItemClassification, ItemEquipSlot},
        definition::ItemDefinition,
        instance::ItemInstance,
    };

    use super::Equipment;

    #[test]
    fn can_equip_if_slot_empty() {
        let head_item = ItemInstance {
            guid: uuid::Uuid::new_v4().to_string(),
            definition: Arc::new(Mutex::new(ItemDefinition {
                id: 1,
                classification: ItemClassification::Equippable(ItemEquipSlot::Head),
                affix_pools: vec![],
                name: String::from("test item"),
            })),
            affixes: vec![],
        };

        let mut equipment = Equipment::new();
        let old_item = equipment.equip(head_item);

        assert!(old_item.is_ok());
        assert!(old_item.unwrap().is_none());
    }

    #[test]
    fn can_equip_if_slot_filled_and_returns_old() {
        let head_item_equipped = ItemInstance {
            guid: uuid::Uuid::new_v4().to_string(),
            definition: Arc::new(Mutex::new(ItemDefinition {
                id: 1,
                classification: ItemClassification::Equippable(ItemEquipSlot::Head),
                affix_pools: vec![],
                name: String::from("test item"),
            })),
            affixes: vec![],
        };

        let head_item_new = ItemInstance {
            guid: uuid::Uuid::new_v4().to_string(),
            definition: Arc::new(Mutex::new(ItemDefinition {
                id: 2,
                classification: ItemClassification::Equippable(ItemEquipSlot::Head),
                affix_pools: vec![],
                name: String::from("test item"),
            })),
            affixes: vec![],
        };

        let mut equipment = Equipment::new();
        equipment.head = Some(head_item_equipped);

        let old_item = equipment.equip(head_item_new);

        assert!(old_item.as_ref().is_ok());
        assert!(old_item.as_ref().unwrap().is_some());
        assert_eq!(old_item.unwrap().unwrap().definition.lock().unwrap().id, 1); // ensure the old item is returned
        assert_eq!(equipment.head.unwrap().definition.lock().unwrap().id, 2); // ensure the new item is correctly stored
    }

    #[test]
    fn can_only_equip_equipables() {
        let currency = ItemInstance {
            guid: uuid::Uuid::new_v4().to_string(),
            definition: Arc::new(Mutex::new(ItemDefinition {
                id: 1,
                classification: ItemClassification::Currency,
                affix_pools: vec![],
                name: String::from("test currency"),
            })),
            affixes: vec![],
        };

        let mut equipment = Equipment::new();
        let currency_result = equipment.equip(currency);

        assert!(currency_result.is_err());
    }
}

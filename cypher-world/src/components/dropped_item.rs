use cypher_item::item::instance::ItemInstance;
use std::sync::{Arc, Mutex};

use bevy::prelude::Component;

#[derive(Component)]
pub struct DroppedItem {
    pub item_instance: Arc<Mutex<ItemInstance>>,
}

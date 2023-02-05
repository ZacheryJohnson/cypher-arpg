use bevy::prelude::{default, Component, Entity, Events, Resource, Vec2};
use cypher_item::item::instance::ItemInstance;
use cypher_item::loot_pool::definition::LootPoolDefinition;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ZJ-TODO: refactor
#[derive(Component, Clone)]
pub struct LootPoolDropper {
    // change this name pls
    pub loot_pool_def: Arc<Mutex<LootPoolDefinition>>,
}

pub struct DeathEvent {
    pub loot_pool: Option<LootPoolDropper>,
    pub position: Vec2,
}

#[derive(Resource)]
pub struct WorldState {
    pub item_drops: HashMap<Entity, Arc<Mutex<ItemInstance>>>,

    pub death_events: Events<DeathEvent>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            item_drops: HashMap::new(),
            death_events: default(),
        }
    }
}

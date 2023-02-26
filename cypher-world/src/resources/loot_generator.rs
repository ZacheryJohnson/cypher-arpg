use crate::resources::world_state::DeathEvent;
use bevy::ecs::event::ManualEventReader;
use cypher_item::loot_pool::generator::LootPoolItemGenerator;

use bevy::prelude::Resource;

#[derive(Default, Resource)]
pub struct LootGenerator {
    pub event_reader: ManualEventReader<DeathEvent>,
    pub loot_pool_generator: LootPoolItemGenerator,
}

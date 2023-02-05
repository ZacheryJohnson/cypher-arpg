use bevy::prelude::SystemSet;

mod spawn_player;
mod spawn_projectile;
mod update_entity_transform;

pub fn get_client_systems() -> SystemSet {
    SystemSet::new()
        .label("client")
        .with_system(spawn_player::listen_for_spawn_player)
        .with_system(update_entity_transform::listen_for_entity_transform_update)
        .with_system(spawn_projectile::listen_for_spawn_projectile)
}

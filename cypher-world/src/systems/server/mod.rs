use bevy::prelude::{IntoSystemDescriptor, SystemSet};

mod handle_item_pickup;
mod loot_generation;
mod player_transform_update;
mod spawn_enemy;
mod spawn_player;
mod spawn_projectile;
mod update_projectile;

pub fn get_server_systems() -> SystemSet {
    SystemSet::new()
        .label("server")
        .with_system(spawn_player::listen_for_spawn_player)
        .with_system(spawn_projectile::listen_for_spawn_projectile)
        .with_system(update_projectile::update_projectiles)
        .with_system(
            spawn_enemy::spawn_initial_enemies
                .with_run_criteria(spawn_enemy::should_spawn_initial_enemies),
        )
        .with_system(loot_generation::loot_generation)
        .with_system(handle_item_pickup::listen_for_item_pickup)
        .with_system(player_transform_update::listen_for_player_transform_update)
}

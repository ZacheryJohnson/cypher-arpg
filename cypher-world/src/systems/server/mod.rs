use bevy::prelude::SystemSet;

mod spawn_player;
mod spawn_projectile;
mod update_projectile;

pub fn get_server_systems() -> SystemSet {
    SystemSet::new()
        .label("server")
        .with_system(spawn_player::listen_for_spawn_player)
        .with_system(spawn_projectile::listen_for_spawn_projectile)
        .with_system(update_projectile::update_projectiles)
}

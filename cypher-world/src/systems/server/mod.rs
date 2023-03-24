use bevy::app::App;
use bevy::prelude::IntoSystemConfig;

mod handle_item_pickup;
mod loot_generation;
mod player_transform_update;
mod spawn_enemy;
mod spawn_player;
mod spawn_projectile;
mod update_projectile;

pub fn register_server_systems(app: &mut App) {
    app.add_systems((
        spawn_player::listen_for_spawn_player,
        spawn_projectile::listen_for_spawn_projectile,
        update_projectile::update_projectiles,
        loot_generation::loot_generation,
        handle_item_pickup::listen_for_item_pickup,
        player_transform_update::listen_for_player_transform_update,
    ))
    .add_system(
        spawn_enemy::spawn_initial_enemies.run_if(spawn_enemy::should_spawn_initial_enemies),
    );
}

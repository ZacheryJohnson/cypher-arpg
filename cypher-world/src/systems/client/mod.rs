use bevy::app::{App, Update};

mod handle_entity_destroyed;
mod spawn_dropped_item;
mod spawn_enemy;
mod spawn_player;
mod spawn_projectile;
mod update_entity_transform;

pub fn register_client_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            spawn_player::listen_for_spawn_player,
            update_entity_transform::listen_for_entity_transform_update,
            spawn_projectile::listen_for_spawn_projectile,
            handle_entity_destroyed::handle_entity_destroyed,
            spawn_enemy::listen_for_spawn_enemy,
            spawn_dropped_item::listen_for_item_dropped,
        ),
    );
}

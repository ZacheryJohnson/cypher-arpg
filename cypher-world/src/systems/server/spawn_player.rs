use bevy::ecs::event::ManualEventReader;
use bevy::prelude::*;
use bevy_renet::renet::{DefaultChannel, RenetServer};
use cypher_character::character::Character;
use cypher_net::components::net_entity::NetEntity;
use cypher_net::components::server_entity::ServerEntity;
use cypher_net::messages::server::server_message::{ServerMessage, ServerMessageVariant};
use cypher_net::resources::lobby::Lobby;
use cypher_net::resources::server_message_dispatcher::ServerToServerMessageDispatcher;
use cypher_net::resources::server_net_entity_registry::ServerNetEntityRegistry;
use serde::Serialize;

use crate::components::collider::Collider;
use crate::components::dropped_item::DroppedItem;
use crate::components::player_controller::PlayerController;
use crate::components::team::Team;
use crate::components::world_entity::{EntityType, WorldEntity};

pub fn listen_for_spawn_player(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut dispatcher: ResMut<ServerToServerMessageDispatcher>,
    mut lobby: ResMut<Lobby>,
    mut net_entities: ResMut<ServerNetEntityRegistry>,
    players: Query<(&Transform, &WorldEntity, &NetEntity), With<ServerEntity>>,
    dropped_items: Query<(&DroppedItem)>,
) {
    let maybe_events = dispatcher.get_events(ServerMessageVariant::PlayerConnected);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ServerMessage> = Default::default();
        for event in reader.iter(&events) {
            if let ServerMessage::PlayerConnected { id } = event {
                spawn_player(
                    &mut commands,
                    &mut server,
                    &mut lobby,
                    &mut net_entities,
                    *id,
                    &players,
                    &dropped_items,
                );
            }
        }
    }
}

fn spawn_player(
    commands: &mut Commands,
    server: &mut ResMut<RenetServer>,
    lobby: &mut ResMut<Lobby>,
    net_entities: &mut ResMut<ServerNetEntityRegistry>,
    player_id: u64,
    world_entities: &Query<(&Transform, &WorldEntity, &NetEntity), With<ServerEntity>>,
    dropped_items: &Query<(&DroppedItem)>,
) {
    let transform = Transform {
        translation: Vec2 { x: 0.0, y: 0.0 }.extend(0.0),
        scale: Vec3 {
            x: 15.,
            y: 15.,
            z: 1.0,
        },
        ..default()
    };

    let mut entity_builder = commands.spawn((
        Character::default(),
        PlayerController,
        ServerEntity,
        WorldEntity {
            entity_type: crate::components::world_entity::EntityType::Player { id: player_id },
        },
        Collider,
        Team { id: 1 },
        transform,
    ));

    let entity_id = entity_builder.id();
    let net_entity = net_entities.register_new(entity_id);
    let net_entity_id = net_entity.id;

    entity_builder.insert(net_entity);

    server.broadcast_message(
        DefaultChannel::Reliable,
        ServerMessage::PlayerSpawned {
            player_id,
            net_entity_id,
            transform,
        }
        .serialize()
        .unwrap(),
    );

    // Replicate world entities to new players upon connection
    for (transform, world_entity, existing_net_entity) in world_entities {
        println!("Replicating world entity {world_entity:?}");

        match world_entity.entity_type {
            EntityType::Player { .. } => {}
            EntityType::Enemy { id: enemy_id } => {
                server.send_message(
                    player_id,
                    DefaultChannel::Reliable,
                    ServerMessage::EnemySpawned {
                        enemy_id,
                        net_entity_id: existing_net_entity.id,
                        transform: *transform,
                    }
                    .serialize()
                    .unwrap(),
                );
            }
            EntityType::Projectile { .. } => {} /* no need to replicate projectiles on connect (right?) */
            EntityType::DroppedItem { .. } => {
                let local_entity = net_entities
                    .get_local_entity(&existing_net_entity.id)
                    .unwrap();
                let dropped_item = dropped_items.get(*local_entity).unwrap();

                println!("Dropping item for new client");

                server.send_message(
                    player_id,
                    DefaultChannel::Reliable,
                    ServerMessage::ItemDropped {
                        item_instance_raw: serde_json::ser::to_vec(
                            &*dropped_item.item_instance.lock().unwrap(),
                        )
                        .unwrap(),
                        net_entity_id,
                        transform: *transform,
                    }
                    .serialize()
                    .unwrap(),
                );
            }
        }
    }

    for (other_player_id, net_entity_id) in &lobby.player_net_ids {
        let local_entity = net_entities
            .get_local_entity(net_entity_id)
            .expect("failed to get local entity for net entity");
        let (transform, _, existing_net_entity) = world_entities.get(*local_entity).unwrap();
        server.send_message(
            player_id,
            DefaultChannel::Reliable,
            ServerMessage::PlayerSpawned {
                player_id: *other_player_id,
                net_entity_id: existing_net_entity.id,
                transform: *transform,
            }
            .serialize()
            .unwrap(),
        );
    }

    lobby.player_net_ids.insert(player_id, net_entity_id);
}

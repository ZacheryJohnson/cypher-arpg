use bevy::ecs::event::ManualEventReader;
use bevy::prelude::*;
use bevy_renet::renet::{DefaultChannel, RenetServer};
use cypher_character::character::Character;
use cypher_net::messages::server::server_message::{ServerMessage, ServerMessageVariant};
use cypher_net::resources::lobby::Lobby;
use cypher_net::resources::server_message_dispatcher::ServerToServerMessageDispatcher;
use cypher_net::resources::server_net_entity_registry::ServerNetEntityRegistry;

use crate::components::collider::Collider;
use crate::components::player_controller::PlayerController;
use crate::components::team::Team;
use crate::components::world_entity::WorldEntity;

// ZJ-TODO: fixme
#[derive(Component)]
pub struct ServerPlayer;

pub fn listen_for_spawn_player(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut dispatcher: ResMut<ServerToServerMessageDispatcher>,
    mut lobby: ResMut<Lobby>,
    mut net_entities: ResMut<ServerNetEntityRegistry>,
    players: Query<(&Transform, Entity), With<ServerPlayer>>,
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
    players: &Query<(&Transform, Entity), With<ServerPlayer>>,
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

    let entity_id = commands
        .spawn((
            Character::default(),
            PlayerController,
            ServerPlayer,
            WorldEntity {
                entity_type: crate::components::world_entity::EntityType::Player { id: player_id },
            },
            Collider,
            Team { id: 1 },
            transform,
        ))
        .id();

    let net_entity_id = net_entities.register_new(entity_id);

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

    for (other_player_id, net_entity_id) in &lobby.player_net_ids {
        let local_entity = net_entities
            .get_local_entity(net_entity_id)
            .expect("failed to get local entity for net entity");
        let (transform, entity) = players.get(*local_entity).unwrap();
        server.send_message(
            player_id,
            DefaultChannel::Reliable,
            ServerMessage::PlayerSpawned {
                player_id: *other_player_id,
                net_entity_id: *net_entity_id,
                transform: *transform,
            }
            .serialize()
            .unwrap(),
        );
    }

    lobby.player_net_ids.insert(player_id, net_entity_id);
}

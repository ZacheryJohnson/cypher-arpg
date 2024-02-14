use bevy::prelude::{Commands, EventReader, ResMut};
use bevy_renet::renet::{DefaultChannel, RenetServer, ServerEvent};

use crate::{
    messages::server::server_message::ServerMessage,
    resources::{
        lobby::Lobby, server_message_dispatcher::ServerToServerMessageDispatcher,
        server_net_entity_registry::ServerNetEntityRegistry,
    },
};

pub fn process_events(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
    mut net_entities: ResMut<ServerNetEntityRegistry>,
    mut dispatcher: ResMut<ServerToServerMessageDispatcher>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Player {} connected.", client_id.raw());

                // Tell the entire server that a new player has joined
                server.broadcast_message(
                    DefaultChannel::ReliableOrdered,
                    ServerMessage::PlayerConnected {
                        id: client_id.raw(),
                    }
                    .serialize()
                    .unwrap(),
                );

                dispatcher.send(ServerMessage::PlayerConnected {
                    id: client_id.raw(),
                });
            }
            ServerEvent::ClientDisconnected {
                client_id,
                reason: _,
            } => {
                println!("Player {} disconnected.", client_id.raw());

                if let Some(player_entity) = lobby.player_net_ids.remove(&client_id.raw()) {
                    if let Some(local_entity) = net_entities.get_local_entity(&player_entity) {
                        commands.entity(*local_entity).despawn();
                    }
                }

                server.broadcast_message(
                    DefaultChannel::ReliableOrdered,
                    ServerMessage::PlayerDisconnected {
                        id: client_id.raw(),
                    }
                    .serialize()
                    .unwrap(),
                );

                dispatcher.send(ServerMessage::PlayerDisconnected {
                    id: client_id.raw(),
                });
            }
        }
    }
}

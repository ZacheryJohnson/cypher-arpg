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
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Player {} connected.", id);

                // Tell the entire server that a new player has joined
                server.broadcast_message(
                    DefaultChannel::Reliable,
                    ServerMessage::PlayerConnected { id: *id }
                        .serialize()
                        .unwrap(),
                );

                dispatcher.send(ServerMessage::PlayerConnected { id: *id });
            }
            ServerEvent::ClientDisconnected(id) => {
                println!("Player {} disconnected.", id);

                if let Some(player_entity) = lobby.player_net_ids.remove(id) {
                    if let Some(local_entity) = net_entities.get_local_entity(&player_entity) {
                        commands.entity(*local_entity).despawn();
                    }
                }

                server.broadcast_message(
                    DefaultChannel::Reliable,
                    ServerMessage::PlayerDisconnected { id: *id }
                        .serialize()
                        .unwrap(),
                );

                dispatcher.send(ServerMessage::PlayerDisconnected { id: *id });
            }
        }
    }
}

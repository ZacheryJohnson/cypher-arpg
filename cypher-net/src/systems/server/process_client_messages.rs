use bevy::prelude::ResMut;
use bevy_renet::renet::{DefaultChannel, RenetServer};

use crate::resources::server_message_dispatcher::ClientToServerMessageDispatcher;
use crate::{
    messages::{client::client_message::ClientMessage, server::server_message::ServerMessage},
    resources::lobby::Lobby,
};

pub fn process_client_messages(
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
    mut dispatcher: ResMut<ClientToServerMessageDispatcher>,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            handle_client_message(message, client_id, &mut lobby, &mut server, &mut dispatcher);
        }

        while let Some(message) = server.receive_message(client_id, DefaultChannel::Reliable) {
            handle_client_message(message, client_id, &mut lobby, &mut server, &mut dispatcher);
        }
    }
}

fn handle_client_message(
    message: Vec<u8>,
    client_id: u64,
    lobby: &mut ResMut<Lobby>,
    server: &mut ResMut<RenetServer>,
    dispatcher: &mut ResMut<ClientToServerMessageDispatcher>,
) {
    if let Some(player_net_entity) = lobby.player_net_ids.get(&client_id) {
        let client_message = serde_json::de::from_slice(&message).unwrap();
        match client_message {
            ClientMessage::PlayerTransformUpdate { transform } => {
                // ZJ-TODO: handle this elsewhere - don't immediately broadcast update, verify it's legit
                server.broadcast_message(
                    DefaultChannel::Unreliable,
                    ServerMessage::EntityTransformUpdate {
                        net_entity_id: *player_net_entity,
                        transform,
                    }
                    .serialize()
                    .unwrap(),
                )
            }
            ClientMessage::SpawnProjectile { .. } => {
                dispatcher.send(client_message);
            }
        }
    }
}

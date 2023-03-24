use bevy::prelude::ResMut;
use bevy_renet::renet::{DefaultChannel, RenetServer};

use crate::resources::lobby::Lobby;
use crate::resources::server_message_dispatcher::ClientToServerMessageDispatcher;

pub fn process_client_messages(
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
    mut dispatcher: ResMut<ClientToServerMessageDispatcher>,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            handle_client_message(message, client_id, &mut lobby, &mut dispatcher);
        }

        while let Some(message) = server.receive_message(client_id, DefaultChannel::Reliable) {
            handle_client_message(message, client_id, &mut lobby, &mut dispatcher);
        }
    }
}

fn handle_client_message(
    message: Vec<u8>,
    client_id: u64,
    lobby: &mut ResMut<Lobby>,
    dispatcher: &mut ResMut<ClientToServerMessageDispatcher>,
) {
    if lobby.player_net_ids.get(&client_id).is_some() {
        let client_message = serde_json::de::from_slice(&message).unwrap();
        dispatcher.send(client_message, client_id);
    }
}

use bevy::prelude::{Commands, EventReader, ResMut};
use bevy_renet::renet::{DefaultChannel, RenetServer, ServerEvent};

use crate::{
    messages::{client::client_message::ClientMessage, server::server_message::ServerMessage},
    resources::{lobby::Lobby, server_net_entity_registry::ServerNetEntityRegistry},
};

pub fn process_client_messages(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
    mut net_entities: ResMut<ServerNetEntityRegistry>,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            if let Some(player_net_entity) = lobby.player_net_ids.get(&client_id) {
                match bincode::deserialize::<ClientMessage>(&message).unwrap() {
                    ClientMessage::PlayerTransformUpdate { transform } => {
                        // ZJ-TODO: don't blindly trust player input
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
                }
            }
        }
    }
}

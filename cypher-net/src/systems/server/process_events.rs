use bevy::prelude::{Commands, EventReader, ResMut};
use bevy_renet::renet::{DefaultChannel, RenetServer, ServerEvent};

use crate::{
    messages::{client::client_message::ClientMessage, server::server_message::ServerMessage},
    resources::lobby::Lobby,
};

pub fn process_events(
    mut server_events: EventReader<ServerEvent>,
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("Player {} connected.", id);

                let message =
                    bincode::serialize(&ServerMessage::PlayerConnected { id: *id }).unwrap();
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
            ServerEvent::ClientDisconnected(id) => {
                println!("Player {} disconnected.", id);
                if let Some(player_entity) = lobby.players.remove(id) {
                    commands.entity(player_entity).despawn();
                }

                let message =
                    bincode::serialize(&ServerMessage::PlayerDisconnected { id: *id }).unwrap();
                server.broadcast_message(DefaultChannel::Reliable, message);
            }
        }
    }

    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::Unreliable) {
            if let Some(player_entity) = lobby.players.get(&client_id) {
                match bincode::deserialize::<ClientMessage>(&message).unwrap() {
                    ClientMessage::PlayerTransformUpdate { transform } => {
                        println!("Sending transform update {transform:?}");
                        // ZJ-TODO: don't blindly trust player input
                        server.broadcast_message(
                            DefaultChannel::Unreliable,
                            ServerMessage::EntityTransformUpdate {
                                entity: *player_entity,
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

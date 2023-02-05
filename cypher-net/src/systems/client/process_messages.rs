use bevy::prelude::{Commands, ResMut};
use bevy_renet::renet::{DefaultChannel, RenetClient};

use crate::{
    messages::server::server_message::ServerMessage,
    resources::server_message_dispatcher::ServerToClientMessageDispatcher,
};

pub fn process_messages(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    mut server_message_dispatcher: ResMut<ServerToClientMessageDispatcher>,
) {
    while let Some(msg) = client.receive_message(DefaultChannel::Reliable) {
        let event = bincode::deserialize::<ServerMessage>(&msg).unwrap();
        server_message_dispatcher.send(event);
    }

    while let Some(msg) = client.receive_message(DefaultChannel::Unreliable) {
        let event = bincode::deserialize::<ServerMessage>(&msg).unwrap();
        server_message_dispatcher.send(event);
    }
}

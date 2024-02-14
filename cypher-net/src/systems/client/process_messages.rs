use bevy::prelude::ResMut;
use bevy_renet::renet::{Bytes, DefaultChannel, RenetClient};

use serde_json;

use crate::resources::server_message_dispatcher::ServerToClientMessageDispatcher;

pub fn process_messages(
    mut client: ResMut<RenetClient>,
    mut server_message_dispatcher: ResMut<ServerToClientMessageDispatcher>,
) {
    while let Some(msg) = client.receive_message(DefaultChannel::ReliableOrdered) {
        handle_message(&mut server_message_dispatcher, msg);
    }

    while let Some(msg) = client.receive_message(DefaultChannel::Unreliable) {
        handle_message(&mut server_message_dispatcher, msg);
    }
}

fn handle_message(server_message_dispatcher: &mut ServerToClientMessageDispatcher, msg: Bytes) {
    let event = serde_json::de::from_slice(&msg).unwrap();
    server_message_dispatcher.send(event);
}

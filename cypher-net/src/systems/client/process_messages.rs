use bevy::prelude::{Res, ResMut};
use bevy_renet::renet::{DefaultChannel, RenetClient};

use serde_json;

use crate::{
    messages::server::server_message::ServerMessage,
    resources::server_message_dispatcher::ServerToClientMessageDispatcher,
};

use cypher_data::resources::data_manager::DataManager;

pub fn process_messages(
    mut client: ResMut<RenetClient>,
    mut server_message_dispatcher: ResMut<ServerToClientMessageDispatcher>,
    data_manager: Res<DataManager>,
) {
    while let Some(msg) = client.receive_message(DefaultChannel::Reliable) {
        handle_message(&data_manager, &mut server_message_dispatcher, msg);
    }

    while let Some(msg) = client.receive_message(DefaultChannel::Unreliable) {
        handle_message(&data_manager, &mut server_message_dispatcher, msg);
    }
}

fn handle_message(
    data_manager: &DataManager,
    server_message_dispatcher: &mut ServerToClientMessageDispatcher,
    msg: Vec<u8>,
) {
    let event = serde_json::de::from_slice(&msg).unwrap();
    server_message_dispatcher.send(event);
}

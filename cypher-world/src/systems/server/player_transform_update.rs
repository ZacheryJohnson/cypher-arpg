use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Res, ResMut};
use bevy_renet::renet::{DefaultChannel, RenetServer};
use cypher_net::messages::client::client_message::{ClientMessage, ClientMessageVariant};
use cypher_net::messages::server::server_message::ServerMessage;
use cypher_net::resources::lobby::Lobby;
use cypher_net::resources::server_message_dispatcher::{
    ClientMessageWithId, ClientToServerMessageDispatcher,
};

pub fn listen_for_player_transform_update(
    mut server: ResMut<RenetServer>,
    mut dispatcher: ResMut<ClientToServerMessageDispatcher>,
    lobby: Res<Lobby>,
) {
    let maybe_events = dispatcher.get_events(ClientMessageVariant::PlayerTransformUpdate);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ClientMessageWithId> = Default::default();
        for ClientMessageWithId {
            msg: event,
            id: client_id,
        } in reader.iter(events)
        {
            let ClientMessage::PlayerTransformUpdate { transform } = event else {
                panic!("what the dispatcher doin")
            };

            // ZJ-TODO: don't blindly trust client input

            let player_net_entity = lobby.player_net_ids.get(client_id).unwrap();

            let server_msg = ServerMessage::EntityTransformUpdate {
                net_entity_id: *player_net_entity,
                transform: *transform,
            }
            .serialize()
            .unwrap();

            server.broadcast_message(DefaultChannel::Unreliable, server_msg);
        }
    }
}

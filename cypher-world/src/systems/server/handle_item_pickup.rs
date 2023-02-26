use crate::components::dropped_item::DroppedItem;
use crate::components::player_controller::PlayerController;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, Entity, Query, Res, ResMut, With};
use bevy_renet::renet::{DefaultChannel, RenetServer};
use cypher_character::character::Character;
use cypher_net::components::server_entity::ServerEntity;
use cypher_net::messages::client::client_message::{ClientMessage, ClientMessageVariant};
use cypher_net::messages::server::server_message::ServerMessage;
use cypher_net::resources::lobby::Lobby;
use cypher_net::resources::server_message_dispatcher::{
    ClientMessageWithId, ClientToServerMessageDispatcher,
};
use cypher_net::resources::server_net_entity_registry::ServerNetEntityRegistry;
use std::sync::Arc;

use crate::components::projectile::Projectile;

pub fn listen_for_item_pickup(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut dispatcher: ResMut<ClientToServerMessageDispatcher>,
    mut net_entities: ResMut<ServerNetEntityRegistry>,
    mut player_query: Query<(&mut Character), With<PlayerController>>,
    mut dropped_items_query: Query<&mut DroppedItem>,
    lobby: Res<Lobby>,
) {
    let maybe_events = dispatcher.get_events(ClientMessageVariant::PickupItem);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ClientMessageWithId> = Default::default();
        for ClientMessageWithId {
            msg: event,
            id: client_id,
        } in reader.iter(&events)
        {
            let ClientMessage::PickupItem { net_entity_id } = event else {
                panic!("what the dispatcher doin")
            };

            println!("Looking for net entity ID {net_entity_id}");

            let Some(item_local_entity) = net_entities.get_local_entity(net_entity_id) else {
                println!("Unknown net entity {net_entity_id} for item pickup");
                continue;
            };

            // ZJ-TODO: validate player can pick up item
            //
            // let dropped_item = dropped_items_query.get_mut(*item_local_entity).expect("unknown item net entity");
            // let item_instance = Arc::try_unwrap(dropped_item.item_instance).expect("had more item arcs live");
            //
            // let character_net_id = lobby.player_net_ids.get(client_id).unwrap();
            // let character_local_entity = net_entities.get_local_entity(character_net_id).expect("unknown character net entity");
            //
            // let character = player_query.get_mut(*character_local_entity).expect("unknown local entity");
            // character
            //     .equipment
            //     .equip(item_instance)
            //     .expect("ZJ-TODO: UI to show failure to equip");
            //
            // println!("{}", character.equipment);

            // ZJ-TODO: actually equip the item on the server

            commands.entity(*item_local_entity).despawn();
            net_entities.delete(net_entity_id);

            server.broadcast_message(
                DefaultChannel::Reliable,
                ServerMessage::EntityDestroyed {
                    net_entity_id: *net_entity_id,
                }
                .serialize()
                .unwrap(),
            );
        }
    }
}

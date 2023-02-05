use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, ResMut};
use cypher_net::messages::server::server_message::{ServerMessage, ServerMessageVariant};
use cypher_net::resources::client_net_entity_registry::ClientNetEntityRegistry;
use cypher_net::resources::server_message_dispatcher::ServerToClientMessageDispatcher;

pub fn handle_entity_destroyed(
    mut commands: Commands,
    mut dispatcher: ResMut<ServerToClientMessageDispatcher>,
    mut net_entities: ResMut<ClientNetEntityRegistry>,
) {
    let maybe_events = dispatcher.get_events(ServerMessageVariant::EntityDestroyed);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ServerMessage> = Default::default();
        for event in reader.iter(&events) {
            let ServerMessage::EntityDestroyed { net_entity_id } = event else {
                println!("dispatcher not doing stuff right lmao");
                continue;
            };

            if let Some(local_entity) = net_entities.get_local_entity(net_entity_id) {
                commands.entity(*local_entity).despawn();
                net_entities.delete(net_entity_id);
            }
        }
    }
}

use bevy::{
    ecs::event::ManualEventReader,
    prelude::{Commands, ResMut},
};
use cypher_net::{
    messages::server::server_message::{ServerMessage, ServerMessageVariant},
    resources::{
        client_net_entity_registry::ClientNetEntityRegistry,
        server_message_dispatcher::ServerToClientMessageDispatcher,
    },
};

pub fn listen_for_entity_transform_update(
    mut dispatcher: ResMut<ServerToClientMessageDispatcher>,
    mut commands: Commands,
    mut net_entities: ResMut<ClientNetEntityRegistry>,
) {
    let maybe_events = dispatcher.get_events(ServerMessageVariant::EntityTransformUpdate);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ServerMessage> = Default::default();
        for event in reader.iter(&events) {
            if let ServerMessage::EntityTransformUpdate {
                net_entity_id,
                transform,
            } = event
            {
                let local_entity = net_entities.get_local_entity(net_entity_id);

                if local_entity.is_none() {
                    println!("Received transform update for unknown net entity {net_entity_id}");
                    continue;
                }

                commands.entity(*local_entity.unwrap()).insert(*transform);
            }
        }
    }
}

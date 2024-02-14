use crate::components::world_decoration::WorldDecoration;
use bevy::prelude::{Query, Transform, Without};
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

/// How much we should prefer the server prediction to the client prediction.
/// 0.0 = use server value only; ignore client
/// 1.0 = use client value only; ignore server
const PREDICTION_SCALE: f32 = 0.75;

pub fn listen_for_entity_transform_update(
    mut dispatcher: ResMut<ServerToClientMessageDispatcher>,
    mut commands: Commands,
    mut net_entities: ResMut<ClientNetEntityRegistry>,
    transforms: Query<&Transform, Without<WorldDecoration>>,
) {
    let maybe_events = dispatcher.get_events(ServerMessageVariant::EntityTransformUpdate);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ServerMessage> = Default::default();
        for event in reader.read(events) {
            if let ServerMessage::EntityTransformUpdate {
                net_entity_id,
                transform,
            } = event
            {
                let Some(local_entity) = net_entities.get_local_entity(net_entity_id) else {
                    println!("Received transform update for unknown net entity {net_entity_id}");
                    continue;
                };

                // We have predicted where we are currently
                // Rather than unconditionally accepting what the server says,
                // take the average of the two transforms.
                let server_transform = *transform;
                let Ok(local_transform) = transforms.get(*local_entity) else {
                    println!("Unknown transform for local entity {local_entity:?}");
                    continue;
                };

                let updated_transform = Transform {
                    translation: server_transform
                        .translation
                        .lerp(local_transform.translation, PREDICTION_SCALE),
                    rotation: server_transform
                        .rotation
                        .lerp(local_transform.rotation, PREDICTION_SCALE),
                    scale: server_transform
                        .scale
                        .lerp(local_transform.scale, PREDICTION_SCALE),
                };

                commands.entity(*local_entity).insert(updated_transform);
            }
        }
    }
}

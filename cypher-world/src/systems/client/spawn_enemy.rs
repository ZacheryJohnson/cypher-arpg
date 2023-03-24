use crate::components::collider::Collider;
use crate::components::team::Team;
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{default, Color, Commands, ResMut, Sprite, SpriteBundle, Vec2};
use cypher_net::messages::server::server_message::{ServerMessage, ServerMessageVariant};
use cypher_net::resources::client_net_entity_registry::ClientNetEntityRegistry;
use cypher_net::resources::server_message_dispatcher::ServerToClientMessageDispatcher;

pub fn listen_for_spawn_enemy(
    mut commands: Commands,
    mut dispatcher: ResMut<ServerToClientMessageDispatcher>,
    mut net_entities: ResMut<ClientNetEntityRegistry>,
) {
    let maybe_events = dispatcher.get_events(ServerMessageVariant::EnemySpawned);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ServerMessage> = Default::default();
        for event in reader.iter(events) {
            let ServerMessage::EnemySpawned { enemy_id: _, net_entity_id, transform } = event else {
                panic!("dispatcher be screwing up");
            };

            let entity = commands.spawn((
                Collider,
                Team { id: 2 },
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(1.0, 0.0, 0.3),
                        custom_size: Some(Vec2 { x: 1., y: 1. }),
                        ..default()
                    },
                    transform: *transform,
                    ..default()
                },
            ));

            let local_entity_id = entity.id();
            net_entities.register_new(*net_entity_id, local_entity_id);
        }
    }
}

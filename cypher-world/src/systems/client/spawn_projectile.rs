use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{default, Color, Commands, ResMut, Sprite, SpriteBundle, Vec2};
use cypher_net::client::Client;
use cypher_net::components::client_entity::ClientEntity;
use cypher_net::messages::server::server_message::{ServerMessage, ServerMessageVariant};
use cypher_net::resources::client_net_entity_registry::ClientNetEntityRegistry;
use cypher_net::resources::server_message_dispatcher::ServerToClientMessageDispatcher;

use crate::components::projectile::Projectile;

pub fn listen_for_spawn_projectile(
    mut commands: Commands,
    mut dispatcher: ResMut<ServerToClientMessageDispatcher>,
    mut net_entities: ResMut<ClientNetEntityRegistry>,
) {
    let maybe_events = dispatcher.get_events(ServerMessageVariant::ProjectileSpawned);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ServerMessage> = Default::default();
        for event in reader.iter(&events) {
            if let ServerMessage::ProjectileSpawned {
                projectile_id,
                net_entity_id,
                transform,
            } = event
            {
                let projectile = Projectile {
                    move_speed: 500.0,
                    lifetime: 800.0,
                    damage: 1.0,
                    team_id: 1,
                };

                println!("Spawning client projectile!");

                let entity_id = commands
                    .spawn((
                        projectile,
                        ClientEntity,
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::rgb(1.0, 0.2, 0.2),
                                custom_size: Some(Vec2 { x: 1., y: 1. }),
                                ..default()
                            },
                            transform: *transform,
                            ..default()
                        },
                    ))
                    .id();

                net_entities.register_new(*net_entity_id, entity_id);
            }
        }
    }
}

use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{Commands, ResMut};
use bevy_renet::renet::{DefaultChannel, RenetServer};
use cypher_net::messages::client::client_message::{ClientMessage, ClientMessageVariant};
use cypher_net::messages::server::server_message::ServerMessage;
use cypher_net::resources::server_message_dispatcher::ClientToServerMessageDispatcher;
use cypher_net::resources::server_net_entity_registry::ServerNetEntityRegistry;

use crate::components::projectile::Projectile;

pub fn listen_for_spawn_projectile(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut dispatcher: ResMut<ClientToServerMessageDispatcher>,
    mut net_entities: ResMut<ServerNetEntityRegistry>,
) {
    let maybe_events = dispatcher.get_events(ClientMessageVariant::SpawnProjectile);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ClientMessage> = Default::default();
        for event in reader.iter(&events) {
            if let ClientMessage::SpawnProjectile {
                projectile_id,
                transform,
            } = event
            {
                let projectile = Projectile {
                    move_speed: 500.0,
                    lifetime: 800.0,
                    damage: 1.0,
                    team_id: 1,
                };

                let entity_id = commands.spawn((projectile, *transform)).id();

                let net_entity_id = net_entities.register_new(entity_id);

                println!("Spawning server projectile!");

                server.broadcast_message(
                    DefaultChannel::Reliable,
                    ServerMessage::ProjectileSpawned {
                        projectile_id: *projectile_id,
                        net_entity_id,
                        transform: *transform,
                    }
                    .serialize()
                    .unwrap(),
                );
            }
        }
    }
}

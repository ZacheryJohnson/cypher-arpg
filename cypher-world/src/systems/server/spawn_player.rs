use bevy::prelude::*;
use bevy_renet::renet::{DefaultChannel, RenetServer, ServerEvent};
use cypher_character::character::Character;
use cypher_net::messages::server::server_message::ServerMessage;

use crate::components::collider::Collider;
use crate::components::team::Team;
use crate::components::world_entity::WorldEntity;

pub fn listen_for_spawn_player(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
) {
    for event in server_events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                spawn_player(&mut commands, &mut server);
            }
            _ => {}
        }
    }
}

fn spawn_player(mut commands: &mut Commands, mut server: &mut ResMut<RenetServer>) {
    let player_id = 123; // ZJ-TODO:

    let _entity_id = commands.spawn((
        Character::default(),
        WorldEntity {
            entity_type: crate::components::world_entity::EntityType::Player { id: player_id },
        },
        Collider,
        Team { id: 1 },
    ));

    server.broadcast_message(
        DefaultChannel::Reliable,
        ServerMessage::PlayerSpawned {
            player_id,
            transform: Transform {
                translation: Vec2 { x: 0.0, y: 0.0 }.extend(0.0),
                scale: Vec3 {
                    x: 15.,
                    y: 15.,
                    z: 1.0,
                },
                ..default()
            },
        }
        .serialize()
        .unwrap(),
    );
}

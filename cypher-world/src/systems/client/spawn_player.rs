use bevy::ecs::event::ManualEventReader;
use bevy::prelude::*;
use cypher_character::character::Character;
use cypher_net::messages::server::server_message::{ServerMessage, ServerMessageVariant};
use cypher_net::resources::server_message_dispatcher::ServerMessageDispatcher;

use crate::components::camera_follow::CameraFollow;
use crate::components::collider::Collider;
use crate::components::player_controller::PlayerController;
use crate::components::team::Team;
use crate::components::world_entity::WorldEntity;

pub fn listen_for_spawn_player(
    mut dispatcher: ResMut<ServerMessageDispatcher>,
    mut commands: Commands,
) {
    let maybe_events = dispatcher.get_events(ServerMessageVariant::PlayerSpawned);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ServerMessage> = Default::default();
        for event in reader.iter(&events) {
            spawn_player(&mut commands, event.to_owned());
        }
    }
}

pub fn spawn_player(commands: &mut Commands, spawn_event: ServerMessage) {
    if let ServerMessage::PlayerSpawned {
        player_id,
        transform,
    } = spawn_event
    {
        commands.spawn((
            Character::default(),
            PlayerController,
            CameraFollow, // ZJ-TODO: apply this conditionally
            WorldEntity {
                entity_type: crate::components::world_entity::EntityType::Player { id: player_id },
            },
            Collider,
            Team { id: 1 },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.0, 1.0, 0.7),
                    custom_size: Some(Vec2 { x: 1., y: 1. }),
                    ..default()
                },
                transform,
                ..default()
            },
        ));

        println!("player spawned!");
    }
}

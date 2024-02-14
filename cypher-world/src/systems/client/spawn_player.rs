use bevy::ecs::event::ManualEventReader;
use bevy::prelude::*;
use cypher_character::character::Character;
use cypher_net::messages::server::server_message::{ServerMessage, ServerMessageVariant};
use cypher_net::resources::client_net_entity_registry::ClientNetEntityRegistry;
use cypher_net::resources::client_state::ClientState;
use cypher_net::resources::server_message_dispatcher::ServerToClientMessageDispatcher;

use crate::components::camera_follow::CameraFollow;
use crate::components::collider::Collider;
use crate::components::player_controller::PlayerController;
use crate::components::team::Team;
use crate::components::world_entity::WorldEntity;

pub fn listen_for_spawn_player(
    mut dispatcher: ResMut<ServerToClientMessageDispatcher>,
    mut commands: Commands,
    mut net_entities: ResMut<ClientNetEntityRegistry>,
    client_state: Res<ClientState>,
) {
    let maybe_events = dispatcher.get_events(ServerMessageVariant::PlayerSpawned);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ServerMessage> = Default::default();
        for event in reader.read(events) {
            spawn_player(
                &mut commands,
                event.to_owned(),
                &mut net_entities,
                &client_state,
            );
        }
    }
}

fn spawn_player(
    commands: &mut Commands,
    spawn_event: ServerMessage,
    net_entities: &mut ResMut<ClientNetEntityRegistry>,
    client_state: &Res<ClientState>,
) {
    if let ServerMessage::PlayerSpawned {
        player_id,
        net_entity_id,
        transform,
    } = spawn_event
    {
        println!("Spawning player");

        let mut entity_builder = commands.spawn((
            Character::default(),
            PlayerController,
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

        if player_id == client_state.client_id.raw() {
            entity_builder.insert(CameraFollow);
        }

        let entity = entity_builder.id();

        net_entities.register_new(net_entity_id, entity);
    }
}

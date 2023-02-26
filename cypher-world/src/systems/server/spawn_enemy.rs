use bevy::ecs::event::ManualEventReader;
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy_renet::renet::{DefaultChannel, RenetServer};
use cypher_character::character::Character;
use cypher_core::data::DataDefinitionDatabase;
use cypher_data::resources::data_manager::DataManager;
use cypher_net::components::server_entity::ServerEntity;
use cypher_net::messages::server::server_message::{ServerMessage, ServerMessageVariant};
use cypher_net::resources::lobby::Lobby;
use cypher_net::resources::server_message_dispatcher::ServerToServerMessageDispatcher;
use cypher_net::resources::server_net_entity_registry::ServerNetEntityRegistry;

use crate::components::collider::Collider;
use crate::components::hit_points::HitPoints;
use crate::components::team::Team;
use crate::components::world_entity::{EntityType, WorldEntity};
use crate::resources::world_state::{LootPoolDropper, WorldState};

pub fn should_spawn_initial_enemies(world_state: Res<WorldState>) -> ShouldRun {
    if world_state.has_spawned_enemies {
        ShouldRun::No
    } else {
        ShouldRun::Yes
    }
}

pub fn spawn_initial_enemies(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut net_entities: ResMut<ServerNetEntityRegistry>,
    mut world_state: ResMut<WorldState>,
    data_manager: Res<DataManager>,
) {
    let positions = vec![
        Vec2 {
            x: -250.0,
            y: 250.0,
        },
        Vec2 { x: 0.0, y: 250.0 },
        Vec2 { x: 250.0, y: 250.0 },
    ];

    for pos in positions {
        let transform = Transform {
            translation: pos.extend(0.0),
            scale: Vec3 {
                x: 45.,
                y: 45.,
                z: 1.0,
            },
            ..default()
        };

        spawn_enemy(
            &mut commands,
            &mut server,
            &mut net_entities,
            &data_manager,
            1,
            &transform,
        );
    }

    world_state.has_spawned_enemies = true;
}

fn spawn_enemy(
    commands: &mut Commands,
    server: &mut ResMut<RenetServer>,
    net_entities: &mut ResMut<ServerNetEntityRegistry>,
    data_manager: &Res<DataManager>,
    enemy_id: u64,
    transform: &Transform,
) {
    let mut entity_builder = commands.spawn((
        HitPoints { health: 10.0 },
        Collider,
        Team { id: 2 },
        ServerEntity,
        WorldEntity {
            entity_type: EntityType::Enemy { id: 1 },
        },
        LootPoolDropper {
            loot_pool_def: data_manager
                .loot_pool_db
                .lock()
                .unwrap()
                .definition(1)
                .unwrap(),
        },
        transform.clone(),
    ));

    let entity_id = entity_builder.id();
    let net_entity = net_entities.register_new(entity_id);
    let net_entity_id = net_entity.id;

    entity_builder.insert(net_entity);

    server.broadcast_message(
        DefaultChannel::Reliable,
        ServerMessage::EnemySpawned {
            enemy_id,
            net_entity_id,
            transform: *transform,
        }
        .serialize()
        .unwrap(),
    );
}

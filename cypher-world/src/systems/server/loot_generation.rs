use crate::components::dropped_item::DroppedItem;
use crate::components::world_entity::{EntityType, WorldEntity};
use crate::resources::loot_generator::LootGenerator;
use crate::resources::world_state::WorldState;
use bevy::prelude::{default, Commands, Res, ResMut, Transform, Vec3};
use bevy_renet::renet::{DefaultChannel, RenetServer};
use cypher_core::data::DataInstanceGenerator;
use cypher_data::resources::data_manager::DataManager;
use cypher_item::loot_pool::generator::LootPoolCriteria;
use cypher_net::components::server_entity::ServerEntity;
use cypher_net::messages::server::server_message::ServerMessage;
use cypher_net::resources::server_net_entity_registry::ServerNetEntityRegistry;
use std::sync::{Arc, Mutex};

pub fn loot_generation(
    mut commands: Commands,
    mut game_state: ResMut<WorldState>,
    mut generator: ResMut<LootGenerator>,
    mut server: ResMut<RenetServer>,
    mut net_entities: ResMut<ServerNetEntityRegistry>,
    data_manager: Res<DataManager>,
) {
    game_state.death_events.update();

    let loot_pool_generator = generator.loot_pool_generator.clone();
    let death_events = &game_state.death_events;

    let mut new_drops = vec![];
    for death_event in generator.event_reader.iter(death_events) {
        println!("Server - received death event");

        let dropper = death_event.loot_pool.as_ref().unwrap();
        let item = loot_pool_generator.generate(
            dropper.loot_pool_def.clone(),
            &LootPoolCriteria {},
            &(
                data_manager.affix_db.clone(),
                data_manager.affix_pool_db.clone(),
                data_manager.item_db.clone(),
            ),
        );

        if let Some(item_instance) = item {
            let item_instance_raw = serde_json::ser::to_vec(&item_instance).unwrap();
            let item_arc = Arc::new(Mutex::new(item_instance));

            let transform = Transform {
                translation: death_event.position.extend(0.0),
                scale: Vec3 {
                    x: 10.0,
                    y: 10.0,
                    z: 1.0,
                },
                ..default()
            };

            let mut entity_builder = commands.spawn((
                DroppedItem {
                    item_instance: item_arc.clone(),
                },
                transform,
                WorldEntity {
                    entity_type: EntityType::DroppedItem { id: 0 },
                },
                ServerEntity,
            ));

            let entity_id = entity_builder.id();
            let net_entity = net_entities.register_new(entity_id);
            let net_entity_id = net_entity.id;

            entity_builder.insert(net_entity);

            println!("Server dropping item with net ID {net_entity_id}");

            server.broadcast_message(
                DefaultChannel::Reliable,
                ServerMessage::ItemDropped {
                    item_instance_raw,
                    net_entity_id,
                    transform,
                }
                .serialize()
                .unwrap(),
            );

            new_drops.push((entity_id, item_arc));
        }
    }

    for (entity, item_arc) in new_drops {
        game_state.item_drops.insert(entity, item_arc);
    }
}

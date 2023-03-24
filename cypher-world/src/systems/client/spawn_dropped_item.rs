use crate::components::dropped_item::DroppedItem;
use crate::components::world_entity::{EntityType, WorldEntity};
use bevy::ecs::event::ManualEventReader;
use bevy::prelude::{default, Color, Commands, Res, ResMut, Sprite, SpriteBundle, Vec2};
use cypher_data::resources::data_manager::DataManager;
use cypher_item::item::deserializer::ItemInstanceDeserializer;
use cypher_item::item::instance::ItemInstanceRarityTier;
use cypher_net::messages::server::server_message::{ServerMessage, ServerMessageVariant};
use cypher_net::resources::client_net_entity_registry::ClientNetEntityRegistry;
use cypher_net::resources::server_message_dispatcher::ServerToClientMessageDispatcher;
use serde::de::DeserializeSeed;
use std::sync::{Arc, Mutex};

pub fn listen_for_item_dropped(
    mut commands: Commands,
    mut dispatcher: ResMut<ServerToClientMessageDispatcher>,
    mut net_entities: ResMut<ClientNetEntityRegistry>,
    data_manager: Res<DataManager>,
) {
    let maybe_events = dispatcher.get_events(ServerMessageVariant::ItemDropped);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ServerMessage> = Default::default();
        for event in reader.iter(events) {
            let ServerMessage::ItemDropped { item_instance_raw, net_entity_id, transform } = event else {
                panic!("dispatcher be screwing up");
            };

            let deserializer = ItemInstanceDeserializer {
                affix_db: data_manager.affix_db.clone(),
                item_db: data_manager.item_db.clone(),
            };

            let item_instance = deserializer
                .deserialize(&mut serde_json::Deserializer::from_slice(
                    item_instance_raw.as_slice(),
                ))
                .unwrap();

            let rarity = item_instance.rarity();
            let item_arc = Arc::new(Mutex::new(item_instance));

            let entity_builder = commands.spawn((
                DroppedItem {
                    item_instance: item_arc,
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: match rarity {
                            ItemInstanceRarityTier::Common => Color::hex("aeb5b0").unwrap(), // off-gray
                            ItemInstanceRarityTier::Uncommon => Color::hex("077d1b").unwrap(), // green
                            ItemInstanceRarityTier::Rare => Color::hex("1f4acc").unwrap(), // blue
                            ItemInstanceRarityTier::Fabled => Color::hex("52288a").unwrap(), // purple
                        },
                        custom_size: Some(Vec2 { x: 1., y: 1. }),
                        ..default()
                    },
                    transform: *transform,
                    ..default()
                },
                WorldEntity {
                    entity_type: EntityType::DroppedItem { id: 0 },
                },
            ));

            let local_entity_id = entity_builder.id();
            net_entities.register_new(*net_entity_id, local_entity_id);
            println!("Dropped item with local entity {local_entity_id:?} has net entity {net_entity_id:?}");
        }
    }
}

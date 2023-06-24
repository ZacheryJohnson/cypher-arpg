// Bevy queries can get very large - allow them
#![allow(clippy::type_complexity)]

use std::time::Duration;

use bevy::reflect::erased_serde::__private::serde::de::DeserializeSeed;
use bevy::{
    app::ScheduleRunnerSettings,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::event::ManualEventReader,
    log::LogPlugin,
    prelude::*,
};
use bevy_renet::{RenetClientPlugin, RenetServerPlugin};
use cypher_character::character::Character;
use cypher_data::resources::data_manager::DataManager;
use cypher_item::item::deserializer::ItemInstanceDeserializer;
use cypher_net::messages::server::server_message::ServerMessageVariant;
use cypher_net::resources::server_message_dispatcher::{
    ClientToServerMessageDispatcher, ServerToClientMessageDispatcher,
    ServerToServerMessageDispatcher,
};
use cypher_net::{
    client::Client,
    messages::server::server_message::ServerMessage,
    resources::{
        client_net_entity_registry::ClientNetEntityRegistry, client_state::ClientState,
        lobby::Lobby, net_limiter::NetLimiter, server_net_entity_registry::ServerNetEntityRegistry,
    },
    server::GameServer,
};
use cypher_world::components::camera_follow::CameraFollow;
use cypher_world::resources::loot_generator::LootGenerator;
use cypher_world::resources::world_state::WorldState;

pub enum SimulationMode {
    ClientOnly,
    ServerOnly,
    ClientAndServer,
}

pub fn start(mode: SimulationMode) {
    let mut app = App::new();

    if let Ok(game_data_path) = std::env::var("GAME_DATA_PATH") {
        println!("Initializing with game data path {}", game_data_path);
        let data_manager = DataManager::new(game_data_path.into());
        app.insert_resource(data_manager);
    } else {
        println!("Initializing with default game data path.");
        app.init_resource::<DataManager>();
    }

    let socket_bind_override = match std::env::var("BIND_ADDR") {
        Ok(addr) => Some(addr),
        _ => None,
    };

    println!("Socket bind override: {socket_bind_override:?}");

    match mode {
        SimulationMode::ClientOnly => {
            let renet_client = Client::new_renet_client();
            let client_id = renet_client.client_id();

            app.init_resource::<WorldState>()
                .init_resource::<NetLimiter>()
                .init_resource::<ClientNetEntityRegistry>()
                .add_event::<ServerMessage>()
                .init_resource::<ServerToClientMessageDispatcher>()
                .add_plugins(DefaultPlugins)
                .add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(LogDiagnosticsPlugin::default())
                .add_plugin(RenetClientPlugin::default())
                .insert_resource(renet_client)
                .insert_resource(ClientState { client_id })
                .add_systems((on_item_picked_up.run_if(player_character_exists),));

            #[cfg(feature = "game_client")]
            {
                app.add_startup_systems((
                    cypher_ux::setup::setup,
                    cypher_world::setup::client::setup,
                ));

                cypher_world::systems::client::register_client_systems(&mut app);
                cypher_net::systems::client::register_client_systems(&mut app);
                cypher_ux::systems::register_client_systems(&mut app);
            }
        }
        SimulationMode::ServerOnly => {
            app.init_resource::<WorldState>()
                .init_resource::<Lobby>()
                .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
                    1.0 / 30.0,
                )))
                .init_resource::<ServerNetEntityRegistry>()
                .init_resource::<ServerToServerMessageDispatcher>()
                .init_resource::<ClientToServerMessageDispatcher>()
                .init_resource::<LootGenerator>()
                .add_plugins(MinimalPlugins)
                .add_plugin(LogPlugin::default())
                .add_plugin(RenetServerPlugin::default())
                .insert_resource(GameServer::new_renet_server(socket_bind_override));

            cypher_net::systems::server::register_server_systems(&mut app);
            cypher_world::systems::server::register_server_systems(&mut app);
        }
        SimulationMode::ClientAndServer => {
            let renet_client = Client::new_renet_client();
            let client_id = renet_client.client_id();

            app.init_resource::<WorldState>()
                .init_resource::<Lobby>()
                .init_resource::<NetLimiter>()
                .init_resource::<ClientNetEntityRegistry>()
                .init_resource::<ServerNetEntityRegistry>()
                .add_event::<ServerMessage>()
                .init_resource::<ServerToClientMessageDispatcher>()
                .init_resource::<ServerToServerMessageDispatcher>()
                .init_resource::<ClientToServerMessageDispatcher>()
                .init_resource::<LootGenerator>()
                .add_plugins(DefaultPlugins)
                .add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(LogDiagnosticsPlugin::default())
                .add_plugin(RenetServerPlugin::default())
                .insert_resource(GameServer::new_renet_server(socket_bind_override))
                .add_plugin(RenetClientPlugin::default())
                .insert_resource(renet_client)
                .insert_resource(ClientState { client_id })
                .add_systems((on_item_picked_up.run_if(player_character_exists),));

            #[cfg(feature = "game_client")]
            {
                app.add_startup_systems((
                    cypher_ux::setup::setup,
                    cypher_world::setup::client::setup,
                ));

                cypher_world::systems::client::register_client_systems(&mut app);
                cypher_net::systems::client::register_client_systems(&mut app);
                cypher_ux::systems::register_client_systems(&mut app);
            }

            cypher_world::systems::server::register_server_systems(&mut app);
            cypher_net::systems::server::register_server_systems(&mut app);
        }
    };

    app.run();
}

// ZJ-TODO: don't use CameraFollow
// Use a PlayerCharacter component or smth
fn player_character_exists(query: Query<(), With<CameraFollow>>) -> bool {
    query.get_single().is_ok()
}

fn on_item_picked_up(
    mut dispatcher: ResMut<ServerToClientMessageDispatcher>,
    mut character_query: Query<&mut Character, With<CameraFollow>>,
    data_manager: Res<DataManager>,
) {
    let maybe_events = dispatcher.get_events(ServerMessageVariant::ItemPickedUp);
    let Some(events) = maybe_events else {
        return;
    };

    let mut reader: ManualEventReader<ServerMessage> = Default::default();
    for event in reader.iter(events) {
        let ServerMessage::ItemPickedUp { item_instance_raw } = event else {
            println!("dispatcher not doing stuff right lmao");
            continue;
        };

        let deserializer = ItemInstanceDeserializer {
            affix_db: data_manager.affix_db.clone(),
            item_db: data_manager.item_db.clone(),
        };

        // ZJ-TODO: have equip be different

        let item_instance = deserializer
            .deserialize(&mut serde_json::Deserializer::from_slice(
                item_instance_raw.as_slice(),
            ))
            .unwrap();

        let mut character = character_query.single_mut();
        character
            .equipment
            .equip(item_instance)
            .expect("ZJ-TODO: UI to show failure to equip");
        println!("{}", character.equipment);
    }
}

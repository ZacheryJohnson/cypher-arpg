// Bevy queries can get very large - allow them
#![allow(clippy::type_complexity)]

use std::{
    collections::HashMap,
    f32::consts::FRAC_PI_2,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

use bevy::reflect::erased_serde::__private::serde::de::DeserializeSeed;
use bevy::{
    app::ScheduleRunnerSettings,
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::{event::ManualEventReader, schedule::ShouldRun},
    log::LogPlugin,
    math::Vec3A,
    prelude::*,
    render::camera::{Camera, RenderTarget},
    sprite::collide_aabb::collide,
};
use bevy_renet::{
    renet::{DefaultChannel, RenetClient},
    RenetClientPlugin, RenetServerPlugin,
};
use cypher_character::character::Character;
use cypher_core::{
    data::{DataDefinitionDatabase, DataInstanceGenerator},
    stat::Stat,
};
use cypher_data::resources::data_manager::DataManager;
use cypher_item::item::deserializer::ItemInstanceDeserializer;
use cypher_item::{
    item::instance::{ItemInstance, ItemInstanceRarityTier},
    loot_pool::{
        definition::LootPoolDefinition,
        generator::{LootPoolCriteria, LootPoolItemGenerator},
    },
};
use cypher_net::messages::server::server_message::ServerMessageVariant;
use cypher_net::resources::server_message_dispatcher::{
    ClientToServerMessageDispatcher, ServerToClientMessageDispatcher,
    ServerToServerMessageDispatcher,
};
use cypher_net::{
    client::Client,
    messages::{client::client_message::ClientMessage, server::server_message::ServerMessage},
    resources::{
        client_net_entity_registry::ClientNetEntityRegistry, client_state::ClientState,
        lobby::Lobby, net_limiter::NetLimiter, server_net_entity_registry::ServerNetEntityRegistry,
    },
    server::GameServer,
    systems::{client, server},
};
use cypher_world::components::dropped_item::DroppedItem;
use cypher_world::components::projectile::Projectile;
use cypher_world::components::world_decoration::WorldDecoration;
use cypher_world::components::{
    camera_follow::CameraFollow, collider::Collider, hit_points::HitPoints,
    player_controller::PlayerController, team::Team, world_entity::WorldEntity,
};
use cypher_world::resources::loot_generator::LootGenerator;
use cypher_world::resources::world_state::{DeathEvent, LootPoolDropper, WorldState};
use rand::{seq::SliceRandom, thread_rng};

pub enum SimulationMode {
    ClientOnly,
    ServerOnly,
    ClientAndServer,
}

pub fn start(mode: SimulationMode) {
    let mut app = App::new();

    match mode {
        SimulationMode::ClientOnly => {
            let renet_client = Client::new_renet_client();
            let client_id = renet_client.client_id();

            app.init_resource::<PlayerSettings>()
                .init_resource::<DataManager>()
                .init_resource::<WorldState>()
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
                .add_startup_system(setup)
                .add_startup_system(setup_world)
                .add_system(handle_input.with_run_criteria(client_connected))
                .add_system(adjust_camera_for_mouse_position.with_run_criteria(client_connected))
                .add_system(pickup_dropped_item_under_cursor.with_run_criteria(client_connected))
                .add_system(on_item_picked_up.with_run_criteria(client_connected))
                .add_system(
                    show_loot_on_hover
                        .with_run_criteria(client_connected)
                        .after(pickup_dropped_item_under_cursor),
                )
                .add_system_set(cypher_world::systems::client::get_client_systems())
                .add_system_set(cypher_net::systems::client::get_client_systems());
        }
        SimulationMode::ServerOnly => {
            app.init_resource::<DataManager>()
                .init_resource::<WorldState>()
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
                .add_plugin(AssetPlugin::default()) // ZJ-TODO: remove this line, projectiles needs refactor
                .add_plugin(RenetServerPlugin::default())
                .insert_resource(GameServer::new_renet_server())
                .add_startup_system(setup)
                .add_startup_system(setup_world)
                .add_system_set(server::get_server_systems());
        }
        SimulationMode::ClientAndServer => {
            let renet_client = Client::new_renet_client();
            let client_id = renet_client.client_id();

            app.init_resource::<PlayerSettings>()
                .init_resource::<DataManager>()
                .init_resource::<WorldState>()
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
                .insert_resource(GameServer::new_renet_server())
                .add_plugin(RenetClientPlugin::default())
                .insert_resource(renet_client)
                .insert_resource(ClientState { client_id })
                .add_startup_system(setup)
                .add_startup_system(setup_world)
                .add_system(handle_input.with_run_criteria(client_connected))
                .add_system(adjust_camera_for_mouse_position.with_run_criteria(client_connected))
                .add_system(pickup_dropped_item_under_cursor.with_run_criteria(client_connected))
                .add_system(on_item_picked_up.with_run_criteria(client_connected))
                .add_system(
                    show_loot_on_hover
                        .with_run_criteria(client_connected)
                        .after(pickup_dropped_item_under_cursor),
                )
                .add_system_set(cypher_world::systems::server::get_server_systems())
                .add_system_set(cypher_world::systems::client::get_client_systems())
                .add_system_set(cypher_net::systems::server::get_server_systems())
                .add_system_set(cypher_net::systems::client::get_client_systems());
        }
    };

    app.run();
}

#[derive(Default, Resource)]
struct PlayerSettings {
    pub mouse_pan_enabled: bool,
    pub alt_mode_enabled: bool,
}

#[derive(Component)]
struct UiItemText;

#[derive(Component)]
struct UiItemTextBox;

fn client_connected(client: Res<RenetClient>, time: Res<Time>) -> ShouldRun {
    // ZJ-TODO: this is a hack - we should instead listen for our player being spawned
    if client.is_connected() && time.startup().elapsed().as_secs_f32() > 1.0 {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn setup(mut commands: Commands, data_manager: Res<DataManager>, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn((
            UiItemTextBox,
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(10.0), Val::Percent(20.0)),
                    flex_wrap: FlexWrap::Wrap,
                    flex_direction: FlexDirection::Column,
                    flex_shrink: 0.03,
                    display: Display::Flex,
                    overflow: Overflow::Hidden,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                background_color: Color::rgba(0.15, 0.15, 0.15, 0.0).into(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                UiItemText,
                TextBundle::from_section(
                    "foobar",
                    TextStyle {
                        font: asset_server.load("fonts/Exo-Regular.ttf"),
                        font_size: 15.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(5.0)),
                    ..default()
                }),
            ));
        });
}

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    const TILE_SIZE: i32 = 64;
    let tile1 = SpriteBundle {
        texture: asset_server.load("sprite/medievalTile_57.png"),
        ..default()
    };

    let tile2 = SpriteBundle {
        texture: asset_server.load("sprite/medievalTile_58.png"),
        ..default()
    };

    let tileset = vec![tile1, tile2];

    for y in -75..75 {
        for x in -75..75 {
            let mut tile = tileset.choose(&mut thread_rng()).unwrap().clone();
            tile.transform.translation = Vec2 {
                x: (x * TILE_SIZE) as f32,
                y: (y * TILE_SIZE) as f32,
            }
            .extend(-10.0);

            commands.spawn((tile, WorldDecoration));
        }
    }
}

fn handle_input(
    mut commands: Commands,
    mut player: Query<
        (&mut Transform, &Character),
        (
            With<WorldEntity>,
            With<PlayerController>,
            With<CameraFollow>, /* ZJ-TODO: this is a hack, don't use CameraFollow */
        ),
    >,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut settings: ResMut<PlayerSettings>,
    collidables: Query<&Transform, (With<Collider>, Without<PlayerController>)>,
    mut client: ResMut<RenetClient>,
    mut net_limiter: ResMut<NetLimiter>,
) {
    let maybe_player = player.get_single_mut();
    let Ok((mut player_transform, character)) = maybe_player else {
        println!("Failed to find player to handle input for");
        return;
    };

    let mut trans = (0.0, 0.0);
    const BASE_MOVE_SPEED: f32 = 100.;
    let move_speed = BASE_MOVE_SPEED + character.stats().get_stat(&Stat::MoveSpeed).unwrap_or(&0.);
    let delta = time.delta().as_secs_f32() * move_speed;

    if keyboard_input.pressed(KeyCode::W) {
        trans.1 += delta;
    } else if keyboard_input.pressed(KeyCode::S) {
        trans.1 -= delta;
    }
    if keyboard_input.pressed(KeyCode::A) {
        trans.0 -= delta;
    } else if keyboard_input.pressed(KeyCode::D) {
        trans.0 += delta;
    }

    settings.alt_mode_enabled = keyboard_input.pressed(KeyCode::LAlt);

    let new_transform = (
        player_transform.translation.x + trans.0,
        player_transform.translation.y + trans.1,
    );
    for collidable in &collidables {
        let collision = collide(
            Vec3 {
                x: new_transform.0,
                y: new_transform.1,
                z: 0.0,
            },
            player_transform.scale.truncate(),
            collidable.translation,
            collidable.scale.truncate(),
        );

        if collision.is_some() {
            // ZJ-TODO: would be nice if being blocked on one axis (eg west) didn't block move on unblocked axis (eg north)
            return;
        }
    }

    player_transform.translation.x = new_transform.0;
    player_transform.translation.y = new_transform.1;

    let msg = ClientMessage::PlayerTransformUpdate {
        transform: *player_transform,
    };
    net_limiter.try_send(&mut client, &msg, DefaultChannel::Unreliable);

    if mouse_input.just_pressed(MouseButton::Middle) {
        settings.mouse_pan_enabled ^= true;
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        let transform = Transform {
            translation: player_transform.translation - player_transform.local_y() * 25.0,
            rotation: player_transform.rotation,
            scale: Vec3 {
                x: 5.,
                y: 5.,
                z: 1.0,
            },
        };

        net_limiter.try_send(
            &mut client,
            &ClientMessage::SpawnProjectile {
                projectile_id: 1, // ZJ-TODO: sadge; would prefer just shoving a Projectile in there,
                // but then cypher-net would have dependency on game libs
                transform: transform.clone(),
            },
            DefaultChannel::Reliable,
        );
    }
}

fn adjust_camera_for_mouse_position(
    mut query: Query<&mut Transform, With<CameraFollow>>,
    mut camera_query: Query<(&Camera, &mut GlobalTransform)>,
    windows: Res<Windows>,
    settings: Res<PlayerSettings>,
) {
    if let Ok((camera, mut camera_transform)) = camera_query.get_single_mut() {
        let window = if let RenderTarget::Window(id) = camera.target {
            windows.get(id).unwrap()
        } else {
            windows.get_primary().unwrap()
        };

        let mut player_transform = query.single_mut();
        let mut camera_position = (
            player_transform.translation.x,
            player_transform.translation.y,
        );
        const OFFSET_SCALE: usize = 100;

        if let Some(cursor_position) = window.cursor_position() {
            if settings.mouse_pan_enabled {
                if let Some(size) = camera.logical_viewport_size() {
                    let (x_offset, y_offset) = (
                        (cursor_position.x / (size.x / 2.0)) - 1.0,
                        (cursor_position.y / (size.y / 2.0)) - 1.0,
                    );
                    camera_position.0 += x_offset * OFFSET_SCALE as f32;
                    camera_position.1 += y_offset * OFFSET_SCALE as f32;
                }
            }

            // get the size of the window
            let window_size = Vec2::new(window.width(), window.height());

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = (cursor_position / window_size) * 2.0 - Vec2::ONE;

            // matrix for undoing the projection and camera transform
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();

            // use it to convert ndc to world-space coordinates
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
            let diff = world_pos - player_transform.translation;
            let angle = diff.y.atan2(diff.x) + FRAC_PI_2;
            player_transform.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), angle);
        }

        *camera_transform.translation_mut() = Vec3A::from(Vec3 {
            x: camera_position.0,
            y: camera_position.1,
            z: 0.0,
        });
    }
}

fn show_loot_on_hover(
    mut ui_elements: Query<&mut BackgroundColor, With<UiItemTextBox>>,
    mut ui_text: Query<&mut Text, With<UiItemText>>,
    mut camera_query: Query<(&Camera, &GlobalTransform)>,
    dropped_items: Query<(&DroppedItem, &Transform)>,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    player_settings: Res<PlayerSettings>,
) {
    let mut color = ui_elements.get_single_mut().unwrap();
    let mut text = ui_text.get_single_mut().unwrap();
    color.0 = Color::rgba(0.15, 0.15, 0.15, 0.0);
    text.sections.clear();

    if let Ok((camera, camera_transform)) = camera_query.get_single_mut() {
        let window = if let RenderTarget::Window(id) = camera.target {
            windows.get(id).unwrap()
        } else {
            windows.get_primary().unwrap()
        };

        if let Some(cursor_position) = window.cursor_position() {
            // get the size of the window
            let window_size = Vec2::new(window.width(), window.height());

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = (cursor_position / window_size) * 2.0 - Vec2::ONE;

            // matrix for undoing the projection and camera transform
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();

            // use it to convert ndc to world-space coordinates
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            for (item_drop, item_transform) in &dropped_items {
                if collide(
                    world_pos,
                    Vec2 { x: 10.0, y: 10.0 },
                    item_transform.translation,
                    Vec2 { x: 10.0, y: 10.0 },
                )
                .is_some()
                {
                    let item_instance = item_drop.item_instance.clone();

                    color.0 = Color::rgba(0.15, 0.15, 0.15, 1.0);
                    text.sections.push(TextSection {
                        value: item_instance
                            .lock()
                            .unwrap()
                            .definition
                            .lock()
                            .unwrap()
                            .name
                            .clone(),
                        style: TextStyle {
                            font: asset_server.load("fonts/Exo-Regular.ttf"),
                            font_size: 15.0,
                            color: Color::WHITE,
                        },
                    });
                    for affix in &item_instance.lock().unwrap().affixes {
                        let mut affix_str = "\n".to_owned() + &affix.stats.to_string();
                        if player_settings.alt_mode_enabled {
                            affix_str += format!(" (T{})", affix.tier).as_str();
                        }
                        text.sections.push(TextSection {
                            value: affix_str,
                            style: TextStyle {
                                font: asset_server.load("fonts/Exo-Regular.ttf"),
                                font_size: 15.0,
                                color: Color::GOLD,
                            },
                        })
                    }
                    break;
                }
            }
        }
    }
}

fn pickup_dropped_item_under_cursor(
    mut camera_query: Query<(&Camera, &GlobalTransform)>,
    dropped_items: Query<(Entity, &Transform), With<DroppedItem>>,
    keyboard_input: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    mut client: ResMut<RenetClient>,
    mut net_entities: ResMut<ClientNetEntityRegistry>,
) {
    if !keyboard_input.pressed(KeyCode::E) {
        return;
    }

    if let Ok((camera, camera_transform)) = camera_query.get_single_mut() {
        let window = if let RenderTarget::Window(id) = camera.target {
            windows.get(id).unwrap()
        } else {
            windows.get_primary().unwrap()
        };

        if let Some(cursor_position) = window.cursor_position() {
            // get the size of the window
            let window_size = Vec2::new(window.width(), window.height());

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = (cursor_position / window_size) * 2.0 - Vec2::ONE;

            // matrix for undoing the projection and camera transform
            let ndc_to_world =
                camera_transform.compute_matrix() * camera.projection_matrix().inverse();

            // use it to convert ndc to world-space coordinates
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            for (entity, item_transform) in &dropped_items {
                if collide(
                    world_pos,
                    Vec2 { x: 10.0, y: 10.0 },
                    item_transform.translation,
                    Vec2 { x: 10.0, y: 10.0 },
                )
                .is_some()
                {
                    println!("Looking for local entity {entity:?}");
                    if let Some(net_entity) = net_entities.get_net_entity(entity) {
                        client.send_message(
                            DefaultChannel::Reliable,
                            ClientMessage::PickupItem {
                                net_entity_id: *net_entity,
                            }
                            .serialize()
                            .unwrap(),
                        );
                    } else {
                        panic!("failed to find net entity for local entity - this should be a UI warning");
                    }
                    return;
                }
            }
        }
    }
}

fn on_item_picked_up(
    mut dispatcher: ResMut<ServerToClientMessageDispatcher>,
    mut character_query: Query<&mut Character, With<CameraFollow>>,
    data_manager: Res<DataManager>,
) {
    let maybe_events = dispatcher.get_events(ServerMessageVariant::ItemPickedUp);
    if let Some(events) = maybe_events {
        let mut reader: ManualEventReader<ServerMessage> = Default::default();
        for event in reader.iter(&events) {
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
                    &item_instance_raw.as_slice(),
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
}

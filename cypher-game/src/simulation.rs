// Bevy queries can get very large - allow them
#![allow(clippy::type_complexity)]

use std::{
    collections::HashMap,
    f32::consts::FRAC_PI_2,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};

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
use cypher_item::{
    item::instance::{ItemInstance, ItemInstanceRarityTier},
    loot_pool::{
        definition::LootPoolDefinition,
        generator::{LootPoolCriteria, LootPoolItemGenerator},
    },
};
use cypher_net::{
    client::Client,
    messages::{client::client_message::ClientMessage, server::server_message::ServerMessage},
    resources::{
        lobby::Lobby, net_limiter::NetLimiter, server_message_dispatcher::ServerMessageDispatcher,
    },
    server::GameServer,
    systems::{client, server},
};
use cypher_world::components::{
    camera_follow::CameraFollow, collider::Collider, hit_points::HitPoints,
    player_controller::PlayerController, team::Team, world_entity::WorldEntity,
};
use rand::{seq::SliceRandom, thread_rng};

use crate::data_manager::DataManager;

pub enum SimulationMode {
    ClientOnly,
    ServerOnly,
    ClientAndServer,
}

pub fn start(mode: SimulationMode) {
    let mut app = App::new();

    match mode {
        SimulationMode::ClientOnly => {
            app.init_resource::<PlayerSettings>()
                .init_resource::<DataManager>()
                .init_resource::<GameState>()
                .init_resource::<NetLimiter>()
                .add_event::<ServerMessage>()
                .init_resource::<ServerMessageDispatcher>()
                .add_plugins(DefaultPlugins)
                .add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(LogDiagnosticsPlugin::default())
                .add_plugin(RenetClientPlugin::default())
                .insert_resource(Client::new_renet_client())
                .add_startup_system(setup)
                .add_startup_system(setup_world)
                .add_system(handle_input.with_run_criteria(client_connected))
                .add_system(adjust_camera_for_mouse_position)
                .add_system(update_projectiles)
                .add_system(pickup_dropped_item_under_cursor)
                .add_system(show_loot_on_hover.after(pickup_dropped_item_under_cursor))
                .add_system_set(client::get_client_systems());
        }
        SimulationMode::ServerOnly => {
            app.init_resource::<DataManager>()
                .init_resource::<GameState>()
                .init_resource::<LootGenerator>()
                .init_resource::<Lobby>()
                .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f64(
                    1.0 / 30.0,
                )))
                .add_plugins(MinimalPlugins)
                .add_plugin(LogPlugin::default())
                .add_plugin(AssetPlugin::default()) // ZJ-TODO: remove this line, projectiles needs refactor
                .add_plugin(RenetServerPlugin::default())
                .insert_resource(GameServer::new_renet_server())
                .add_startup_system(setup)
                .add_startup_system(setup_world)
                .add_system(update_projectiles)
                .add_system(loot_generation)
                .add_system_set(server::get_server_systems());
        }
        SimulationMode::ClientAndServer => {
            app.init_resource::<PlayerSettings>()
                .init_resource::<DataManager>()
                .init_resource::<GameState>()
                .init_resource::<LootGenerator>()
                .init_resource::<Lobby>()
                .init_resource::<NetLimiter>()
                .add_event::<ServerMessage>()
                .init_resource::<ServerMessageDispatcher>()
                .add_plugins(DefaultPlugins)
                .add_plugin(FrameTimeDiagnosticsPlugin::default())
                .add_plugin(LogDiagnosticsPlugin::default())
                .add_plugin(RenetServerPlugin::default())
                .insert_resource(GameServer::new_renet_server())
                .add_plugin(RenetClientPlugin::default())
                .insert_resource(Client::new_renet_client())
                .add_startup_system(setup)
                .add_startup_system(setup_world)
                .add_system(handle_input.with_run_criteria(client_connected))
                .add_system(adjust_camera_for_mouse_position.with_run_criteria(client_connected))
                .add_system(update_projectiles)
                .add_system(loot_generation)
                .add_system(pickup_dropped_item_under_cursor.with_run_criteria(client_connected))
                .add_system(
                    show_loot_on_hover
                        .with_run_criteria(client_connected)
                        .after(pickup_dropped_item_under_cursor),
                )
                .add_system(cypher_world::systems::client::spawn_player::listen_for_spawn_player)
                .add_system(cypher_world::systems::server::spawn_player::listen_for_spawn_player)
                .add_system_set(server::get_server_systems())
                .add_system_set(client::get_client_systems());
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
struct Projectile {
    pub move_speed: f32,
    pub lifetime: f32,
    pub damage: f32,
    pub team_id: u16,
}

#[derive(Component, Clone)]
struct LootPoolDropper {
    // change this name pls
    pub loot_pool_def: Arc<Mutex<LootPoolDefinition>>,
}

#[derive(Component)]
struct ItemDrop {
    pub item_instance: Weak<Mutex<ItemInstance>>,
}

struct DeathEvent {
    pub loot_pool: Option<LootPoolDropper>,
    pub position: Vec2,
}

#[derive(Default, Resource)]
struct LootGenerator {
    pub event_reader: ManualEventReader<DeathEvent>,
    pub loot_pool_generator: LootPoolItemGenerator,
}

#[derive(Resource)]
struct GameState {
    pub item_drops: HashMap<Entity, Arc<Mutex<ItemInstance>>>,

    pub death_events: Events<DeathEvent>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            item_drops: HashMap::new(),
            death_events: default(),
        }
    }
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

    spawn_enemies(commands, data_manager);
}

fn spawn_enemies(mut commands: Commands, data_manager: Res<DataManager>) {
    let positions = vec![
        Vec2 {
            x: -250.0,
            y: 250.0,
        },
        Vec2 { x: 0.0, y: 250.0 },
        Vec2 { x: 250.0, y: 250.0 },
    ];

    for position in positions {
        commands.spawn((
            HitPoints { health: 10.0 },
            Collider,
            Team { id: 2 },
            LootPoolDropper {
                loot_pool_def: data_manager
                    .loot_pool_db
                    .lock()
                    .unwrap()
                    .definition(1)
                    .unwrap(),
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.0, 0.3),
                    custom_size: Some(Vec2 { x: 1., y: 1. }),
                    ..default()
                },
                transform: Transform {
                    translation: position.extend(0.0),
                    scale: Vec3 {
                        x: 45.,
                        y: 45.,
                        z: 1.0,
                    },
                    ..default()
                },
                ..default()
            },
        ));
    }
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

            commands.spawn(tile);
        }
    }
}

fn handle_input(
    mut commands: Commands,
    mut players: Query<(&mut Transform, &Character), (With<WorldEntity>, With<PlayerController>)>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut settings: ResMut<PlayerSettings>,
    collidables: Query<&Transform, (With<Collider>, Without<PlayerController>)>,
    mut client: ResMut<RenetClient>,
    mut net_limiter: ResMut<NetLimiter>,
) {
    let (mut player_transform, character) = players.single_mut();

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
        // temp: spawn "bullet"
        commands.spawn((
            Projectile {
                move_speed: 500.0,
                lifetime: 800.0,
                damage: 1.0,
                team_id: 1,
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.2, 0.2),
                    custom_size: Some(Vec2 { x: 1., y: 1. }),
                    ..default()
                },
                transform: Transform {
                    translation: player_transform.translation - player_transform.local_y() * 25.0,
                    rotation: player_transform.rotation,
                    scale: Vec3 {
                        x: 5.,
                        y: 5.,
                        z: 1.0,
                    },
                },
                ..default()
            },
        ));
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

fn update_projectiles(
    mut commands: Commands,
    mut projectiles: Query<(&mut Transform, &mut Projectile, Entity)>,
    mut collidables: Query<
        (
            &Transform,
            &mut HitPoints,
            &Team,
            Option<&LootPoolDropper>,
            Entity,
        ),
        (With<Collider>, Without<Projectile>),
    >,
    time: Res<Time>,
    mut game_state: ResMut<GameState>,
) {
    for (mut projectile_transform, mut projectile, entity) in &mut projectiles {
        let forward = -projectile_transform.local_y();
        let distance = forward * projectile.move_speed * time.delta().as_secs_f32();
        projectile_transform.translation += distance;
        projectile.lifetime -= distance.length();

        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        for (collidable_transform, mut hit_points, team, maybe_loot, collider_entity) in
            &mut collidables
        {
            // Don't let projectiles hurt their own team members
            if team.id == projectile.team_id {
                continue;
            }

            if collide(
                projectile_transform.translation,
                projectile_transform.scale.truncate(),
                collidable_transform.translation,
                collidable_transform.scale.truncate(),
            )
            .is_some()
            {
                hit_points.health -= projectile.damage;
                if hit_points.health <= 0.0 {
                    commands.entity(collider_entity).despawn();

                    game_state.death_events.send(DeathEvent {
                        loot_pool: maybe_loot.map(|loot| loot.to_owned()),
                        position: Vec2 {
                            x: collidable_transform.translation.x,
                            y: collidable_transform.translation.y,
                        },
                    });
                }

                commands.entity(entity).despawn();
                continue;
            }
        }
    }
}

fn loot_generation(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut generator: ResMut<LootGenerator>,
    data_manager: Res<DataManager>,
) {
    game_state.death_events.update();

    let loot_pool_generator = generator.loot_pool_generator.clone();
    let death_events = &game_state.death_events;

    let mut maybe_item_arc: Option<(Arc<Mutex<ItemInstance>>, Entity)> = None;

    for death_event in generator.event_reader.iter(death_events) {
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
            let item_arc = Arc::new(Mutex::new(item_instance));
            let rarity = item_arc.lock().unwrap().rarity();

            let new_entity = commands.spawn((
                ItemDrop {
                    item_instance: Arc::downgrade(&item_arc),
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
                    transform: Transform {
                        translation: death_event.position.extend(0.0),
                        scale: Vec3 {
                            x: 10.0,
                            y: 10.0,
                            z: 1.0,
                        },
                        ..default()
                    },
                    ..default()
                },
            ));

            maybe_item_arc = Some((item_arc, new_entity.id()));
        }
    }

    if let Some(item) = maybe_item_arc {
        game_state.item_drops.insert(item.1, item.0);
    }
}

fn show_loot_on_hover(
    mut ui_elements: Query<&mut BackgroundColor, With<UiItemTextBox>>,
    mut ui_text: Query<&mut Text, With<UiItemText>>,
    mut camera_query: Query<(&Camera, &GlobalTransform)>,
    dropped_items: Query<(&ItemDrop, &Transform)>,
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
                    let Some(item_instance) = item_drop.item_instance.upgrade() else {
                        break;
                    };

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
    mut commands: Commands,
    mut camera_query: Query<(&Camera, &GlobalTransform)>,
    mut character_query: Query<&mut Character, With<PlayerController>>,
    dropped_items: Query<(Entity, &Transform), With<ItemDrop>>,
    keyboard_input: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    mut game_state: ResMut<GameState>,
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
                    let mut character = character_query.single_mut();

                    // Remove existing components that reference the item
                    let item_instance = game_state.item_drops.remove(&entity).unwrap();
                    commands.entity(entity).despawn_recursive();

                    // Knowing that we have the only instance, unwrap the Arc<Mutex<ItemInstance>> to move out the ItemInstance
                    let item = Arc::try_unwrap(item_instance)
                        .unwrap()
                        .into_inner()
                        .unwrap();
                    character
                        .equipment
                        .equip(item)
                        .expect("ZJ-TODO: UI to show failure to equip");

                    println!("{}", character.equipment);

                    return;
                }
            }
        }
    }
}

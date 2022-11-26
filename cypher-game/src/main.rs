// Bevy queries can get very large - allow them
#![allow(clippy::type_complexity)]

use std::{
    f32::consts::FRAC_PI_2,
    sync::{Arc, Mutex},
};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::Vec3A,
    prelude::*,
    render::camera::{Camera, RenderTarget},
    sprite::collide_aabb::collide,
};
use cypher_core::data::{DataDefinitionDatabase, DataInstanceGenerator};
use cypher_item::loot_pool::{
    generator::{LootPoolCriteria, LootPoolItemGenerator},
    LootPoolDefinition,
};
use cypher_world::WorldEntity;
use rand::{seq::SliceRandom, thread_rng};

pub mod data_manager;
use data_manager::DataManager;

fn main() {
    App::new()
        .init_resource::<PlayerSettings>()
        .init_resource::<DataManager>()
        .init_resource::<GameState>()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_startup_system(setup_world)
        .add_system(handle_input)
        .add_system(adjust_camera_for_mouse_position)
        .add_system(update_projectiles)
        .run();
}

#[derive(Component)]
struct CameraFollow;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Team {
    pub id: u16,
}

#[derive(Component)]
struct HitPoints {
    health: f32,
}

#[derive(Default, Resource)]
struct PlayerSettings {
    pub mouse_pan_enabled: bool,
}

#[derive(Component)]
struct Projectile {
    pub move_speed: f32,
    pub lifetime: f32,
    pub damage: f32,
    pub team_id: u16,
}

#[derive(Component)]
struct LootPoolDropper {
    // change this name pls
    pub loot_pool_def: Arc<Mutex<LootPoolDefinition>>,
}

#[derive(Resource)]
struct GameState {
    pub loot_pool_generator: LootPoolItemGenerator,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            loot_pool_generator: LootPoolItemGenerator,
        }
    }
}

fn setup(mut commands: Commands, data_manager: Res<DataManager>) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        WorldEntity,
        CameraFollow,
        HitPoints { health: 10.0 },
        Collider,
        Team { id: 1 },
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 1.0, 0.7),
                custom_size: Some(Vec2 { x: 1., y: 1. }),
                ..default()
            },
            transform: Transform {
                translation: Vec2 { x: 0.0, y: 0.0 }.extend(0.0),
                scale: Vec3 {
                    x: 15.,
                    y: 15.,
                    z: 1.0,
                },
                ..default()
            },
            ..default()
        },
    ));

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
                translation: Vec2 { x: 250.0, y: 250.0 }.extend(0.0),
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
    mut players: Query<&mut Transform, With<WorldEntity>>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut settings: ResMut<PlayerSettings>,
    collidables: Query<&Transform, (With<Collider>, Without<WorldEntity>)>,
) {
    let mut player_transform = players.single_mut();

    let mut trans = (0.0, 0.0);
    const MOVE_SPEED: usize = 100;
    let delta = time.delta().as_secs_f32() * MOVE_SPEED as f32;

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
            let window_size = Vec2::new(window.width() as f32, window.height() as f32);

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
    data_manager: Res<DataManager>,
    game_state: Res<GameState>,
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

                    // eventually do something with this
                    if let Some(loot) = maybe_loot {
                        let def = loot.loot_pool_def.clone();
                        let item = game_state.loot_pool_generator.generate(
                            def,
                            &LootPoolCriteria {},
                            &(
                                data_manager.affix_db.clone(),
                                data_manager.affix_pool_db.clone(),
                                data_manager.item_db.clone(),
                            ),
                        );

                        println!("{}", item);
                    }
                }

                commands.entity(entity).despawn();
                continue;
            }
        }
    }
}

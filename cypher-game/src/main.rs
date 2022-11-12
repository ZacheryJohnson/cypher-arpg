use std::f32::consts::FRAC_PI_2;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::Vec3A,
    prelude::*,
    render::camera::{Camera, RenderTarget},
    sprite::collide_aabb::collide,
};
use cypher_world::WorldEntity;

fn main() {
    App::new()
        .init_resource::<Settings>()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
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
    pub debug: String,
}

#[derive(Component)]
struct HitPoints {
    health: f32,
}

#[derive(Default)]
struct Settings {
    pub mouse_pan_enabled: bool,
}

#[derive(Component)]
struct Projectile {
    pub move_speed: f32,
    pub lifetime: f32,
    pub damage: f32,
    pub team_id: u16,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn()
        .insert(WorldEntity)
        .insert(CameraFollow)
        .insert(HitPoints { health: 10.0 })
        .insert(Collider)
        .insert(Team {
            id: 1,
            debug: String::from("Player"),
        })
        .insert_bundle(SpriteBundle {
            texture: asset_server.load("hell.png"),
            transform: Transform {
                translation: Vec2 { x: 0.0, y: 0.0 }.extend(0.0),
                scale: Vec3 {
                    x: 0.15,
                    y: 0.15,
                    z: 1.0,
                },
                ..default()
            },
            ..default()
        });

    commands
        .spawn()
        .insert(HitPoints { health: 10.0 })
        .insert(Collider)
        .insert(Team {
            id: 2,
            debug: String::from("Enemy"),
        })
        .insert_bundle(SpriteBundle {
            texture: asset_server.load("hell.png"),
            transform: Transform {
                translation: Vec2 { x: 250.0, y: 250.0 }.extend(0.0),
                scale: Vec3 {
                    x: 0.25,
                    y: 0.25,
                    z: 1.0,
                },
                ..default()
            },
            ..default()
        });
}

fn handle_input(
    mut commands: Commands,
    mut players: Query<&mut Transform, With<WorldEntity>>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut settings: ResMut<Settings>,
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
    player_transform.translation.x = new_transform.0;
    player_transform.translation.y = new_transform.1;

    if mouse_input.just_pressed(MouseButton::Middle) {
        settings.mouse_pan_enabled ^= true;
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        // temp: spawn "bullet"
        commands
            .spawn()
            .insert(Projectile {
                move_speed: 500.0,
                lifetime: 800.0,
                damage: 1.0,
                team_id: 1,
            })
            .insert_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.2, 0.2),
                    custom_size: Some(Vec2 { x: 150.0, y: 150.0 }),
                    ..default()
                },
                transform: Transform {
                    translation: player_transform.translation.clone()
                        - player_transform.local_y() * 25.0,
                    rotation: player_transform.rotation.clone(),
                    scale: Vec3 {
                        x: 0.05,
                        y: 0.05,
                        z: 1.0,
                    },
                    ..default()
                },
                ..default()
            });
    }
}

fn adjust_camera_for_mouse_position(
    mut query: Query<&mut Transform, With<CameraFollow>>,
    mut camera_query: Query<(&Camera, &mut GlobalTransform)>,
    windows: Res<Windows>,
    settings: Res<Settings>,
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
        (&Transform, &mut HitPoints, &Team, Entity),
        (With<Collider>, Without<Projectile>),
    >,
    time: Res<Time>,
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

        for (collidable_transform, mut hit_points, team, collider_entity) in &mut collidables {
            // Don't let projectiles hurt their own team members
            if team.id == projectile.team_id {
                continue;
            }

            let collision = collide(
                projectile_transform.translation,
                Vec2 { x: 0.05, y: 0.05 },
                collidable_transform.translation,
                Vec2 { x: 70.0, y: 70.0 },
            );

            if let Some(_) = collision {
                println!("Collision with {}", team.debug);
                hit_points.health -= projectile.damage;
                if hit_points.health <= 0.0 {
                    commands.entity(collider_entity).despawn();
                }

                commands.entity(entity).despawn();
                continue;
            }
        }
    }
}

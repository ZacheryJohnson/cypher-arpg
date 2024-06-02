use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::prelude::Vec2;
use bevy::{
    prelude::{ButtonInput, KeyCode, Query, Res, ResMut, Transform, With, Without},
    time::Time,
};
use bevy_renet::renet::{DefaultChannel, RenetClient};
use cypher_character::character::Character;
use cypher_core::stat::Stat;
use cypher_net::{
    messages::client::client_message::ClientMessage, resources::net_limiter::NetLimiter,
};
use cypher_world::components::{
    camera_follow::CameraFollow, collider::Collider, player_controller::PlayerController,
    world_entity::WorldEntity,
};

use crate::resources::player_settings::PlayerSettings;

pub fn handle_keyboard_input(
    mut player: Query<
        (&mut Transform, &Character),
        (
            With<WorldEntity>,
            With<PlayerController>,
            With<CameraFollow>, /* ZJ-TODO: this is a hack, don't use CameraFollow */
        ),
    >,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
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

    if keyboard_input.pressed(KeyCode::KeyW) {
        trans.1 += delta;
    } else if keyboard_input.pressed(KeyCode::KeyS) {
        trans.1 -= delta;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        trans.0 -= delta;
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        trans.0 += delta;
    }

    settings.alt_mode_enabled = keyboard_input.pressed(KeyCode::AltLeft);

    let new_transform = (
        player_transform.translation.x + trans.0,
        player_transform.translation.y + trans.1,
    );
    for collidable in &collidables {
        let player_collider = Aabb2d::new(
            Vec2 {
                x: new_transform.0,
                y: new_transform.1,
            },
            player_transform.scale.truncate(),
        );
        let collidable_collider = Aabb2d::new(
            collidable.translation.truncate(),
            collidable.scale.truncate(),
        );

        if player_collider.intersects(&collidable_collider) {
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
}

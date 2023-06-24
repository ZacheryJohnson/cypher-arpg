use bevy::prelude::{Input, MouseButton, Query, Res, ResMut, Transform, Vec3, With};
use bevy_renet::renet::{DefaultChannel, RenetClient};
use cypher_character::character::Character;
use cypher_net::{
    messages::client::client_message::ClientMessage, resources::net_limiter::NetLimiter,
};
use cypher_world::components::{
    camera_follow::CameraFollow, player_controller::PlayerController, world_entity::WorldEntity,
};

use crate::resources::player_settings::PlayerSettings;

pub fn handle_mouse_input(
    player: Query<
        (&Transform, &Character),
        (
            With<WorldEntity>,
            With<PlayerController>,
            With<CameraFollow>, /* ZJ-TODO: this is a hack, don't use CameraFollow */
        ),
    >,
    mouse_input: Res<Input<MouseButton>>,
    mut settings: ResMut<PlayerSettings>,
    mut client: ResMut<RenetClient>,
    mut net_limiter: ResMut<NetLimiter>,
) {
    let maybe_player = player.get_single();
    let Ok((player_transform, _)) = maybe_player else {
        println!("Failed to find player to handle input for");
        return;
    };

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
                transform,
            },
            DefaultChannel::Reliable,
        );
    }
}

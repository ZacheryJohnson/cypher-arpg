use bevy::math::bounding::{Aabb2d, IntersectsVolume};
use bevy::{
    prelude::{
        ButtonInput, Camera, Entity, GlobalTransform, KeyCode, Query, Res, ResMut, Transform, Vec2,
        With, Without,
    },
    window::{PrimaryWindow, Window},
};
use bevy_renet::renet::{DefaultChannel, RenetClient};
use cypher_net::{
    components::server_entity::ServerEntity, messages::client::client_message::ClientMessage,
    resources::client_net_entity_registry::ClientNetEntityRegistry,
};
use cypher_world::components::dropped_item::DroppedItem;

pub fn pickup_dropped_item_under_cursor(
    mut camera_query: Query<(&Camera, &GlobalTransform)>,
    dropped_items: Query<(Entity, &Transform), (With<DroppedItem>, Without<ServerEntity>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut client: ResMut<RenetClient>,
    mut net_entities: ResMut<ClientNetEntityRegistry>,
) {
    if !keyboard_input.just_pressed(KeyCode::KeyE) {
        return;
    }

    if let Ok((camera, camera_transform)) = camera_query.get_single_mut() {
        let window = window_query
            .get_single()
            .expect("failed to get primary camera");

        if let Some(cursor_position) = window.cursor_position() {
            // use it to convert ndc to world-space coordinates
            let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position)
            else {
                // Couldn't convert - mouse likely outside of window
                // Don't log - this would get spammy
                return;
            };

            for (entity, item_transform) in &dropped_items {
                let world_pos_collider = Aabb2d::new(world_pos, Vec2 { x: 10.0, y: 10.0 });
                let item_collider = Aabb2d::new(
                    item_transform.translation.truncate(),
                    Vec2 { x: 10.0, y: 10.0 },
                );
                if world_pos_collider.intersects(&item_collider) {
                    if let Some(net_entity) = net_entities.get_net_entity(entity) {
                        client.send_message(
                            DefaultChannel::ReliableOrdered,
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

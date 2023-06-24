use bevy::{
    prelude::{
        Camera, Entity, GlobalTransform, Input, KeyCode, Query, Res, ResMut, Transform, Vec2, With,
        Without,
    },
    sprite::collide_aabb::collide,
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
    keyboard_input: Res<Input<KeyCode>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut client: ResMut<RenetClient>,
    mut net_entities: ResMut<ClientNetEntityRegistry>,
) {
    if !keyboard_input.just_pressed(KeyCode::E) {
        return;
    }

    if let Ok((camera, camera_transform)) = camera_query.get_single_mut() {
        let window = window_query
            .get_single()
            .expect("failed to get primary camera");

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

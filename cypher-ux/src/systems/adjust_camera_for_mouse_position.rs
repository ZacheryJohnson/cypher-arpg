use std::f32::consts::FRAC_PI_2;

use bevy::prelude::GlobalTransform;
use bevy::{
    prelude::{Camera, Quat, Query, Res, Transform, Vec3, With, Without},
    window::{PrimaryWindow, Window},
};
use cypher_world::components::camera_follow::CameraFollow;

use crate::resources::player_settings::PlayerSettings;

pub fn adjust_camera_for_mouse_position(
    mut query: Query<&mut Transform, (With<CameraFollow>, Without<Camera>)>,
    mut camera_query: Query<(&Camera, &GlobalTransform, &mut Transform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    settings: Res<PlayerSettings>,
) {
    if let Ok((camera, camera_global_transform, mut camera_transform)) =
        camera_query.get_single_mut()
    {
        let window = window_query
            .get_single()
            .expect("failed to get primary camera");

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
                    camera_position.1 -= y_offset * OFFSET_SCALE as f32;
                }
            }

            // use it to convert ndc to world-space coordinates
            let Some(world_pos) =
                camera.viewport_to_world_2d(camera_global_transform, cursor_position)
            else {
                // Couldn't convert - mouse likely outside of window
                // Don't log - this would get spammy
                return;
            };

            let diff = world_pos.extend(0.0) - player_transform.translation;
            let angle = diff.y.atan2(diff.x) + FRAC_PI_2;
            player_transform.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), angle);
        }

        camera_transform.translation = Vec3 {
            x: camera_position.0,
            y: camera_position.1,
            z: 0.0,
        };
    }
}

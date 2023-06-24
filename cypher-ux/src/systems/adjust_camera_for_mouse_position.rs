use std::f32::consts::FRAC_PI_2;

use bevy::{
    prelude::{Camera, Quat, Query, Res, Transform, Vec2, Vec3, With, Without},
    window::{PrimaryWindow, Window},
};
use cypher_world::components::camera_follow::CameraFollow;

use crate::resources::player_settings::PlayerSettings;

pub fn adjust_camera_for_mouse_position(
    mut query: Query<&mut Transform, (With<CameraFollow>, Without<Camera>)>,
    mut camera_query: Query<(&Camera, &mut Transform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    settings: Res<PlayerSettings>,
) {
    if let Ok((camera, mut camera_transform)) = camera_query.get_single_mut() {
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

        camera_transform.translation = Vec3 {
            x: camera_position.0,
            y: camera_position.1,
            z: 0.0,
        };
    }
}

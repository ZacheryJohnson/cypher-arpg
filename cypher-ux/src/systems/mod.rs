use bevy::app::Update;
use bevy::prelude::{App, IntoSystemConfigs, Query, With};
use cypher_world::components::camera_follow::CameraFollow;

use crate::resources::player_settings::PlayerSettings;

pub mod adjust_camera_for_mouse_position;
pub mod handle_keyboard_input;
pub mod handle_mouse_input;
pub mod pickup_dropped_item_under_cursor;
pub mod show_loot_on_hover;

// ZJ-TODO: don't use CameraFollow
// Use a PlayerCharacter component or smth
fn player_character_exists(query: Query<(), With<CameraFollow>>) -> bool {
    query.get_single().is_ok()
}

pub fn register_client_systems(app: &mut App) {
    app.init_resource::<PlayerSettings>();

    app.add_systems(
        Update,
        (
            adjust_camera_for_mouse_position::adjust_camera_for_mouse_position
                .run_if(player_character_exists),
            show_loot_on_hover::show_loot_on_hover.run_if(player_character_exists),
            pickup_dropped_item_under_cursor::pickup_dropped_item_under_cursor
                .run_if(player_character_exists),
            handle_mouse_input::handle_mouse_input.run_if(player_character_exists),
            handle_keyboard_input::handle_keyboard_input.run_if(player_character_exists),
        ),
    );
}

use bevy::prelude::Resource;

#[derive(Default, Resource)]
pub struct PlayerSettings {
    pub mouse_pan_enabled: bool,
    pub alt_mode_enabled: bool,
}

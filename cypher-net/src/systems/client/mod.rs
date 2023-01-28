use bevy::prelude::SystemSet;

pub fn get_client_systems() -> SystemSet {
    SystemSet::new().label("client")
}

use bevy::prelude::SystemSet;

pub mod process_messages;

pub fn get_client_systems() -> SystemSet {
    SystemSet::new()
        .label("client")
        .with_system(process_messages::process_messages)
}

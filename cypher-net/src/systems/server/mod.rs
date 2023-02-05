use bevy::prelude::SystemSet;

mod process_client_messages;
mod process_events;

pub fn get_server_systems() -> SystemSet {
    SystemSet::new()
        .label("server")
        .with_system(process_events::process_events)
        .with_system(process_client_messages::process_client_messages)
}

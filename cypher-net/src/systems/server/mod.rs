use bevy::prelude::SystemSet;

mod process_events;

pub fn get_server_systems() -> SystemSet {
    SystemSet::new()
        .label("server")
        .with_system(process_events::process_events)
}

use bevy::app::{App, Update};

mod process_client_messages;
mod process_events;

pub fn register_server_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            process_events::process_events,
            process_client_messages::process_client_messages,
        ),
    );
}

use bevy::app::App;

mod process_client_messages;
mod process_events;

pub fn register_server_systems(app: &mut App) {
    app.add_systems((
        process_events::process_events,
        process_client_messages::process_client_messages,
    ));
}

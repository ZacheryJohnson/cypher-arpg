use bevy::app::App;

pub mod process_messages;

pub fn register_client_systems(app: &mut App) {
    app.add_system(process_messages::process_messages);
}

use bevy::app::{App, Update};

pub mod process_messages;

pub fn register_client_systems(app: &mut App) {
    app.add_systems(Update, process_messages::process_messages);
}

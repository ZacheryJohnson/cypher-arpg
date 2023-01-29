use bevy::prelude::{Commands, ResMut};
use bevy_renet::renet::{DefaultChannel, RenetClient};

use crate::messages::server::server_message::ServerMessage;

pub fn process_messages(mut commands: Commands, mut client: ResMut<RenetClient>) {
    while let Some(msg) = client.receive_message(DefaultChannel::Unreliable) {
        match bincode::deserialize::<ServerMessage>(&msg).unwrap() {
            ServerMessage::PlayerConnected { id } => println!("Player {id} connected!"),
            ServerMessage::PlayerDisconnected { id } => println!("Player {id} disconnected!"),
            ServerMessage::EntityTransformUpdate { entity, transform } => {
                if let Some(mut world_entity) = commands.get_entity(entity) {
                    world_entity.insert(transform);
                    println!("updated {entity:?} to {transform:?}");
                } else {
                    println!("unknown entity");
                }
            }
        }
    }
}

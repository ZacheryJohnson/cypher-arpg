use bevy::{
    ecs::event::Event,
    prelude::{Entity, Transform},
};
use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumDiscriminants)]
// ZJ-TODO: if Rust adds types for enum variants, we can remove the strum discriminant functionality
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(ServerMessageVariant))]
#[strum_discriminants(derive(Hash))]
pub enum ServerMessage {
    PlayerConnected {
        id: u64,
    },
    PlayerDisconnected {
        id: u64,
    },
    PlayerSpawned {
        player_id: u64,
        transform: Transform,
    },
    EntityTransformUpdate {
        entity: Entity,
        transform: Transform,
    },
}

impl ServerMessage {
    pub fn serialize(&self) -> Option<Vec<u8>> {
        if let Ok(bytes) = bincode::serialize(&self) {
            Some(bytes)
        } else {
            None
        }
    }
}

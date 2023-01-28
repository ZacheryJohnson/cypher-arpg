use bevy::prelude::{Entity, Transform};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    PlayerConnected {
        id: u64,
    },
    PlayerDisconnected {
        id: u64,
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

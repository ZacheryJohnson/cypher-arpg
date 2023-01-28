use bevy::prelude::Transform;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Requests to have the player character's transform updated on the server. This will be normalized by the server.
    /// If a client lies and tries to update to an unobtainable transform, the server will apply the correct values.
    /// For example, if a player tried to move 1000 units northwest but their move speed would only allow 200,
    /// the server would update their transform 200 units northwest (if possible, eg no walls).
    PlayerTransformUpdate { transform: Transform },
}

impl ClientMessage {
    pub fn serialize(&self) -> Option<Vec<u8>> {
        if let Ok(bytes) = bincode::serialize(&self) {
            Some(bytes)
        } else {
            None
        }
    }
}

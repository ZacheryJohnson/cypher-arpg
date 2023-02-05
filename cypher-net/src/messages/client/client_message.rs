use bevy::prelude::Transform;
use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, EnumDiscriminants)]
// ZJ-TODO: if Rust adds types for enum variants, we can remove the strum discriminant functionality
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(ClientMessageVariant))]
#[strum_discriminants(derive(Hash))]
pub enum ClientMessage {
    /// Requests to have the player character's transform updated on the server. This will be normalized by the server.
    /// If a client lies and tries to update to an unobtainable transform, the server will apply the correct values.
    /// For example, if a player tried to move 1000 units northwest but their move speed would only allow 200,
    /// the server would update their transform 200 units northwest (if possible, eg no walls).
    PlayerTransformUpdate { transform: Transform },

    /// ZJ-TODO: refactor
    ///
    /// Requests to spawn a projectile at the given transform.
    SpawnProjectile {
        projectile_id: u64,
        transform: Transform,
    },
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

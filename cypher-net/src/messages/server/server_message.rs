use bevy::prelude::Transform;
use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;

use crate::components::net_entity::NetEntityT;

#[derive(Clone, Debug, Deserialize, Serialize, EnumDiscriminants)]
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
        net_entity_id: NetEntityT,
        transform: Transform,
    },
    EntityTransformUpdate {
        net_entity_id: NetEntityT,
        transform: Transform,
    },
    EntityDestroyed {
        net_entity_id: NetEntityT,
    },
    ProjectileSpawned {
        projectile_id: u64,
        net_entity_id: NetEntityT,
        transform: Transform,
    },
    EnemySpawned {
        enemy_id: u64,
        net_entity_id: NetEntityT,
        transform: Transform,
    },
    ItemDropped {
        item_instance_raw: Vec<u8>,
        net_entity_id: NetEntityT,
        transform: Transform,
    },
    ItemPickedUp {
        item_instance_raw: Vec<u8>,
    },
}

impl ServerMessage {
    pub fn serialize(&self) -> Option<Vec<u8>> {
        if let Ok(bytes) = serde_json::ser::to_vec(&self) {
            Some(bytes)
        } else {
            None
        }
    }
}

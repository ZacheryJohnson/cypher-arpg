use bevy::prelude::Entity;

pub type NetEntityId = u32;

pub struct NetEntity {
    /// This server's local entity. The entity ID can differ per-client.
    pub entity: Entity,

    /// A global ID for an entity, shared by all clients. This is the ID clients should use to refer to a specific entity.
    pub id: NetEntityId,
}

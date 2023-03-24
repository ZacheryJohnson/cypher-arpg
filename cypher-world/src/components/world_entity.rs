use bevy::prelude::Component;

/// Any interactable object in world space
#[derive(Component, Debug)]
pub struct WorldEntity {
    pub entity_type: EntityType,
}

#[derive(Debug)]
pub enum EntityType {
    Player { id: u64 },
    Enemy { id: u64 },
    Projectile { id: u64 },
    DroppedItem { id: u64 },
}

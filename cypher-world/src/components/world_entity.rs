use bevy::prelude::Component;

/// Any interactable object in world space
#[derive(Component)]
pub struct WorldEntity {
    pub entity_type: EntityType,
}

pub enum EntityType {
    Player { id: u64 },
    Projectile { id: u64 },
}

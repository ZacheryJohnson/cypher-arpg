use bevy::prelude::Component;

#[derive(Component)]
pub struct Projectile {
    pub move_speed: f32,
    pub lifetime: f32,
    pub damage: f32,
    pub team_id: u16,
}

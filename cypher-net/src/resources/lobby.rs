use bevy::{
    prelude::{Entity, Resource},
    utils::HashMap,
};

#[derive(Default, Debug, Resource)]
pub struct Lobby {
    pub players: HashMap<u64, Entity>,
}

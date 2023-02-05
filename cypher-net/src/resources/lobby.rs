use bevy::{prelude::Resource, utils::HashMap};

use crate::net_entity::NetEntityT;

#[derive(Default, Debug, Resource)]
pub struct Lobby {
    pub player_net_ids: HashMap<u64, NetEntityT>,
}

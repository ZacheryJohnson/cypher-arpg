use bevy::{
    prelude::{Entity, Resource},
    utils::HashMap,
};

use crate::components::net_entity::NetEntityT;

#[derive(Default, Debug, Resource)]
pub struct ClientNetEntityRegistry {
    net_entities: HashMap<NetEntityT, Entity>,
}

impl ClientNetEntityRegistry {
    pub fn get_local_entity(&mut self, net_entity: &NetEntityT) -> Option<&mut Entity> {
        self.net_entities.get_mut(net_entity)
    }

    pub fn get_net_entity(&mut self, local_entity: Entity) -> Option<&NetEntityT> {
        self.net_entities
            .iter()
            .find(|(net, local)| **local == local_entity)
            .map(|(net, local)| net)
    }

    pub fn register_new(&mut self, net_entity_id: NetEntityT, local_entity: Entity) -> NetEntityT {
        self.net_entities.insert(net_entity_id, local_entity);

        net_entity_id
    }

    pub fn delete(&mut self, net_entity: &NetEntityT) {
        self.net_entities.remove(net_entity);
    }
}

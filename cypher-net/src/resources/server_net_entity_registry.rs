use crate::components::net_entity::{NetEntity, NetEntityT};
use bevy::{
    prelude::{Entity, Resource},
    utils::HashMap,
};

#[derive(Default, Debug, Resource)]
pub struct ServerNetEntityRegistry {
    net_entities: HashMap<NetEntityT, Entity>,
}

impl ServerNetEntityRegistry {
    pub fn get_local_entity(&mut self, net_entity: &NetEntityT) -> Option<&mut Entity> {
        self.net_entities.get_mut(net_entity)
    }

    pub fn register_new(&mut self, entity: Entity) -> NetEntity {
        let net_entity = NetEntity::default();

        self.net_entities.insert(net_entity.id, entity);

        net_entity
    }

    pub fn delete(&mut self, net_entity: &NetEntityT) {
        self.net_entities.remove(net_entity);
    }
}

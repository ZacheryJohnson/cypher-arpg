use bevy::{
    prelude::{Entity, Resource},
    utils::HashMap,
};

use crate::net_entity::NetEntityT;

#[derive(Default, Debug, Resource)]
pub struct ServerNetEntityRegistry {
    net_entities: HashMap<NetEntityT, Entity>,
}

impl ServerNetEntityRegistry {
    pub fn get_local_entity(&mut self, net_entity: &NetEntityT) -> Option<&mut Entity> {
        self.net_entities.get_mut(net_entity)
    }

    pub fn register_new(&mut self, entity: Entity) -> NetEntityT {
        let net_entity_id = crate::net_entity::next();

        self.net_entities.insert(net_entity_id, entity);

        println!("Registering entity {entity:?} as net entity {net_entity_id}");

        net_entity_id
    }
}

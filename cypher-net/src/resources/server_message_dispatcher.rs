use std::collections::HashMap;

use bevy::prelude::{Events, Resource};

use crate::messages::server::server_message::{ServerMessage, ServerMessageVariant};

#[derive(Default, Resource)]
pub struct ServerMessageDispatcher {
    event_map: HashMap<ServerMessageVariant, Events<ServerMessage>>,
}

impl ServerMessageDispatcher {
    pub fn send(&mut self, message: ServerMessage) {
        println!("sending");
        let variant: ServerMessageVariant = message.into();

        if let Some(events) = self.event_map.get_mut(&variant) {
            events.send(message);
        } else {
            let mut new_events = Events::<ServerMessage>::default();
            new_events.send(message);
            self.event_map.insert(variant, new_events);
        }
    }

    pub fn get_events(
        &mut self,
        variant: ServerMessageVariant,
    ) -> Option<&mut Events<ServerMessage>> {
        // ZJ-TODO: yuck
        if let Some(events) = self.event_map.get_mut(&variant) {
            events.update();
        }

        self.event_map.get_mut(&variant)
    }
}

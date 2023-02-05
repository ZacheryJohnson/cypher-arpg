use std::collections::HashMap;

use bevy::prelude::{Events, Resource};

use crate::messages::server::server_message::{ServerMessage, ServerMessageVariant};

#[derive(Default, Resource)]
pub struct ServerToClientMessageDispatcher {
    event_map: HashMap<ServerMessageVariant, Events<ServerMessage>>,
}

#[derive(Default, Resource)]
pub struct ServerToServerMessageDispatcher {
    event_map: HashMap<ServerMessageVariant, Events<ServerMessage>>,
}

impl ServerToClientMessageDispatcher {
    pub fn send(&mut self, message: ServerMessage) {
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
        // ZJ-TODO: yuck - explicitly updating events and calling get_mut twice feels bad
        if let Some(events) = self.event_map.get_mut(&variant) {
            events.update();
        }

        self.event_map.get_mut(&variant)
    }
}

impl ServerToServerMessageDispatcher {
    pub fn send(&mut self, message: ServerMessage) {
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
        // ZJ-TODO: yuck - explicitly updating events and calling get_mut twice feels bad
        if let Some(events) = self.event_map.get_mut(&variant) {
            events.update();
        }

        self.event_map.get_mut(&variant)
    }
}

use std::collections::HashMap;

use crate::messages::client::client_message::{ClientMessage, ClientMessageVariant};
use bevy::prelude::{Events, Resource};

use crate::messages::server::server_message::{ServerMessage, ServerMessageVariant};

pub struct ClientMessageWithId {
    pub msg: ClientMessage,
    pub id: u64,
}

#[derive(Default, Resource)]
pub struct ClientToServerMessageDispatcher {
    event_map: HashMap<ClientMessageVariant, Events<ClientMessageWithId>>,
}

#[derive(Default, Resource)]
pub struct ServerToClientMessageDispatcher {
    event_map: HashMap<ServerMessageVariant, Events<ServerMessage>>,
}

#[derive(Default, Resource)]
pub struct ServerToServerMessageDispatcher {
    event_map: HashMap<ServerMessageVariant, Events<ServerMessage>>,
}

impl ClientToServerMessageDispatcher {
    pub fn send(&mut self, message: ClientMessage, client_id: u64) {
        let variant: ClientMessageVariant = message.into();
        let msg_with_id = ClientMessageWithId {
            msg: message,
            id: client_id,
        };

        if let Some(events) = self.event_map.get_mut(&variant) {
            events.send(msg_with_id);
        } else {
            let mut new_events = Events::<ClientMessageWithId>::default();
            new_events.send(msg_with_id);
            self.event_map.insert(variant, new_events);
        }
    }

    pub fn get_events(
        &mut self,
        variant: ClientMessageVariant,
    ) -> Option<&mut Events<ClientMessageWithId>> {
        // ZJ-TODO: yuck - explicitly updating events and calling get_mut twice feels bad
        if let Some(events) = self.event_map.get_mut(&variant) {
            events.update();
        }

        self.event_map.get_mut(&variant)
    }
}

impl ServerToClientMessageDispatcher {
    pub fn send(&mut self, message: ServerMessage) {
        let variant: ServerMessageVariant = message.clone().into();

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
        let variant: ServerMessageVariant = message.clone().into();

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

use std::{collections::HashMap, mem::Discriminant, time::Instant};

use bevy::prelude::Resource;
use bevy_renet::renet::RenetClient;

use crate::messages::client::client_message::ClientMessage;

#[derive(Resource)]
pub struct NetLimiter {
    messages_per_second: f32,
    messages_by_id: HashMap<Discriminant<ClientMessage>, Instant>,
}

impl Default for NetLimiter {
    fn default() -> Self {
        Self {
            messages_per_second: 40.0,
            messages_by_id: Default::default(),
        }
    }
}

impl NetLimiter {
    pub fn try_send<ChannelT>(
        &mut self,
        client: &mut RenetClient,
        msg: &ClientMessage,
        channel: ChannelT,
    ) -> bool
    where
        ChannelT: Into<u8>,
    {
        if let Some(last_msg_sent) = self.messages_by_id.get_mut(&std::mem::discriminant(msg)) {
            if last_msg_sent.elapsed().as_secs_f32() <= (1.0 / self.messages_per_second) {
                return false;
            }

            *last_msg_sent = Instant::now();
        } else {
            self.messages_by_id
                .insert(std::mem::discriminant(msg), Instant::now());
        }

        let serialized = msg.serialize().unwrap();
        client.send_message(channel, serialized);

        true
    }
}

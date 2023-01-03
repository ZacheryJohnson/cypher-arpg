use bevy::prelude::Transform;
use postcard;
use serde::{Deserialize, Serialize};
use strum_macros::EnumProperty;

use crate::{net_character::NetEntityId, net_message::NetMessage};

#[derive(Copy, Clone, Debug, EnumProperty, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Updates clients with the position of a net entity.
    EntityTransformUpdate(NetEntityId, Transform),
}

impl NetMessage for ServerMessage {
    fn from_bytes(mut bytes: Vec<u8>) -> Self {
        let _message_id =
            u16::from_be_bytes(bytes.drain(0..2).collect::<Vec<u8>>().try_into().unwrap());

        let message: Self = postcard::from_bytes(bytes.as_slice()).unwrap();

        message
    }
}

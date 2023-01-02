use bevy::prelude::Transform;
use postcard;
use serde::{Deserialize, Serialize};
use strum::EnumProperty;
use strum_macros::EnumProperty;

type MessageTypeId = u16;
const MESSAGE_TYPE_ID_STR: &'static str = "TypeId";

pub trait NetMessage {
    fn as_bytes(&self) -> Vec<u8>;
}

#[derive(Copy, Clone, EnumProperty, Serialize, Deserialize)]
pub enum ClientMessage {
    #[strum(props(TypeId = "1"))]
    EntityTransformUpdate(Transform),
}

impl NetMessage for ClientMessage {
    fn as_bytes(&self) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![];

        // First bytes - message ID
        let id = self
            .get_str(MESSAGE_TYPE_ID_STR)
            .expect("all messages need a type ID")
            .parse::<MessageTypeId>()
            .unwrap();
        buffer.extend_from_slice(&id.to_be_bytes());

        // Next bytes - message body
        let mut data_buffer = [0u8; 1024];
        let data = postcard::to_slice(self, &mut data_buffer).unwrap();
        buffer.extend_from_slice(&data);

        buffer
    }
}

pub enum ServerMessage {}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use bevy::{math::quat, prelude::Vec3};

    use super::*;

    #[test]
    fn can_serialize_net_message() {
        let transform = Transform {
            translation: Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            rotation: quat(10.0, 20.0, 30.0, 40.0),
            scale: Vec3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
        };
        let net_message = ClientMessage::EntityTransformUpdate(transform);
        let serialized = net_message.as_bytes();

        assert!(serialized.len() > 0);
        assert_eq!(serialized.as_slice()[0..2], [0x00, 0x01]);
    }
}

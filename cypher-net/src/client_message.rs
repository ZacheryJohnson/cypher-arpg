use bevy::prelude::Transform;
use postcard;
use serde::{Deserialize, Serialize};
use strum_macros::EnumProperty;

use crate::net_character::NetEntityId;
use crate::net_message::NetMessage;

#[derive(Copy, Clone, Debug, EnumProperty, Serialize, Deserialize)]
pub enum ClientMessage {
    #[strum(props(TypeId = "1"))]
    /// Requests the server update the given net entity. The following restrictions apply:
    /// The client must be authoritative over the provided NetEntityId.
    /// The provided transform's translation will be treated as a unit vector
    /// with the entity's actual transform being calculated by the server.
    EntityTransformUpdate(NetEntityId, Transform),
}

impl NetMessage for ClientMessage {
    fn from_bytes(mut bytes: Vec<u8>) -> Self {
        let _message_id =
            u16::from_be_bytes(bytes.drain(0..2).collect::<Vec<u8>>().try_into().unwrap());

        let message: Self = postcard::from_bytes(bytes.as_slice()).unwrap();

        message
    }
}

#[cfg(test)]
mod tests {
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
        let net_message = ClientMessage::EntityTransformUpdate(0, transform);
        let serialized = net_message.as_bytes();
        println!("Serialized: {:?}", serialized);

        assert!(serialized.len() > 0);
        assert_eq!(serialized.as_slice()[0..2], [0x00, 0x01]);
    }

    #[test]
    fn can_deserialize_net_message() {
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
        let net_message = ClientMessage::EntityTransformUpdate(0, transform);
        let serialized = net_message.as_bytes();

        let deserialized = ClientMessage::from_bytes(serialized);
        println!("Deserialized: {:?}", deserialized);
    }
}

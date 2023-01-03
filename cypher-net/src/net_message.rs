use serde::Serialize;
use strum::EnumProperty;

type MessageTypeId = u16;
const MESSAGE_TYPE_ID_STR: &str = "TypeId";

pub trait NetMessage {
    fn as_bytes(&self) -> Vec<u8>
    where
        Self: EnumProperty + Serialize,
    {
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
        buffer.extend_from_slice(data);

        buffer
    }

    // ZJ-TODO: add default impl here; needs lifetime for where Self: Deserialize
    fn from_bytes(bytes: Vec<u8>) -> Self;
}

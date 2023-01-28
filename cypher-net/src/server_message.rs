use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
}

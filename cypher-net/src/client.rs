use std::{net::UdpSocket, time::SystemTime};

use bevy_renet::renet::{ClientAuthentication, RenetClient, RenetConnectionConfig};

use crate::protocol::PROTOCOL_ID;

pub struct Client;

impl Client {
    pub fn new_renet_client() -> RenetClient {
        let server_addr = "127.0.0.1:5000".parse().unwrap();
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let connection_config = RenetConnectionConfig::default();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let client_id = current_time.as_millis() as u64;
        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: PROTOCOL_ID,
            server_addr,
            user_data: None,
        };
        RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
    }
}

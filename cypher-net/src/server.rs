use std::{net::UdpSocket, time::SystemTime};

use bevy_renet::renet::{RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig};

use crate::protocol::PROTOCOL_ID;

pub struct GameServer;

impl GameServer {
    pub fn new_renet_server(bind_override: Option<String>) -> RenetServer {
        let server_addr = String::from("127.0.0.1:5000");
        let bind_addr = bind_override.unwrap_or(server_addr.clone());
        let socket = UdpSocket::bind(bind_addr).unwrap();
        println!("Binding server to {:?}", socket.local_addr());
        let connection_config = RenetConnectionConfig::default();
        let server_config = ServerConfig::new(
            8,
            PROTOCOL_ID,
            server_addr.parse().unwrap(),
            ServerAuthentication::Unsecure,
        );
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
    }
}

use bevy::app::App;
use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::renet::{ConnectionConfig, RenetServer};
use bevy_renet::transport::NetcodeServerPlugin;
use bevy_renet::RenetServerPlugin;
use std::net::UdpSocket;
use std::time::SystemTime;

use crate::protocol::PROTOCOL_ID;

pub struct GameServer;

impl GameServer {
    pub fn initialize(app: &mut App) {
        app.add_plugins((RenetServerPlugin, NetcodeServerPlugin));

        let server = RenetServer::new(ConnectionConfig::default());
        app.insert_resource(server);

        let server_addr = "127.0.0.1:5000".parse().unwrap();
        let socket = UdpSocket::bind(server_addr).unwrap();
        let server_config = ServerConfig {
            current_time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap(),
            max_clients: 64,
            protocol_id: PROTOCOL_ID,
            public_addresses: vec![server_addr],
            authentication: ServerAuthentication::Unsecure,
        };

        let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
        app.insert_resource(transport);
    }
}

use bevy::app::App;
use bevy_renet::renet::transport::{ClientAuthentication, NetcodeClientTransport};
use bevy_renet::renet::{ClientId, ConnectionConfig, RenetClient};
use bevy_renet::transport::NetcodeClientPlugin;
use bevy_renet::RenetClientPlugin;
use std::net::{SocketAddr, UdpSocket};
use std::time::SystemTime;

use rand::Rng;

use crate::protocol::PROTOCOL_ID;

pub struct Client;

impl Client {
    pub fn initialize(app: &mut App) -> ClientId {
        app.add_plugins((RenetClientPlugin, NetcodeClientPlugin));

        let renet_client = RenetClient::new(ConnectionConfig::default());
        app.insert_resource(renet_client);

        let client_id: u64 = rand::thread_rng().gen();

        let auth = ClientAuthentication::Unsecure {
            server_addr: SocketAddr::new("127.0.0.1".parse().unwrap(), 5000),
            client_id,
            user_data: None,
            protocol_id: PROTOCOL_ID,
        };

        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let transport = NetcodeClientTransport::new(current_time, auth, socket).unwrap();

        app.insert_resource(transport);

        ClientId::from_raw(client_id)
    }
}

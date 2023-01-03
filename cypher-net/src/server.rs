use std::sync::{Arc, Mutex};

// ZJ-TODO: remove these - used in hack_remove_pls
use bevy::{
    math::quat,
    prelude::{Transform, Vec3},
};

use tokio::net::UdpSocket;

use crate::{client_message::ClientMessage, net_message::NetMessage};

#[derive(Default)]
pub struct Server {
    message_buffer: Arc<Mutex<Vec<ClientMessage>>>,
}

impl Server {
    pub async fn listen(&self, addr: &str, port: u16) {
        let socket = UdpSocket::bind(format!("{addr}:{port}")).await.unwrap();
        let message_buffer = self.message_buffer.clone();

        tokio::task::spawn(async move {
            loop {
                let mut buffer = vec![];
                let (len, addr) = socket.recv_from(buffer.as_mut_slice()).await.unwrap();
                println!("{:?} bytes received from {:?}", len, addr);

                let client_message = ClientMessage::from_bytes(buffer);
                message_buffer.lock().unwrap().push(client_message);
                println!("Client message: {:?}", client_message);
            }
        });
    }

    // To allow game to test prior to full end-to-end
    pub fn hack_remove_pls(&mut self) {
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
        self.message_buffer
            .lock()
            .unwrap()
            .push(ClientMessage::EntityTransformUpdate(1, transform))
    }

    pub fn get_messages(&mut self) -> Vec<ClientMessage> {
        let mut messages = self.message_buffer.lock().unwrap();
        let drained = messages.drain(..).collect::<Vec<ClientMessage>>();
        drained
    }
}

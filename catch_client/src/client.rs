use std::collections::HashMap;
use std::io::Read;

use cereal::CerealData;
use enet;

use shared::player::{PlayerId, PlayerInfo};
use shared::net::{ClientMessage, ServerMessage};

pub struct Client {
    host: enet::Host,
    server_peer: enet::Peer,
    connected: bool,

    my_name: String,
    my_id: Option<PlayerId>,
}

impl Client {
    pub fn connect(timeout_ms: u32,
                   host_name: String,
                   port: u16,
                   my_name: String) -> Result<Client, String> {
        let (host, server_peer) =
            try!(enet::Host::connect(timeout_ms,
                                     host_name,
                                     port,
                                     2,
                                     0, 0));

        Ok(Client {
            host: host,
            server_peer: server_peer,
            connected: false,
            my_name: my_name,
            my_id: None
        })
    }

    fn send(&self, message: &ClientMessage) {
        let mut data = Vec::new();
        match message.write(&mut data) {
            Err(_) => {
                println!("Error encoding message {:?}", message);
                return;
            },
            Ok(_) => ()
        }

        self.server_peer.send(&data, enet::ffi::ENET_PACKET_FLAG_RELIABLE, 0);
    }

    pub fn finish_connecting(&mut self, timeout_ms: u32) -> Result<PlayerId, String> {
        assert!(!self.connected);

        self.send(&ClientMessage::WishConnect {
            name: self.my_name.clone()
        });

        match self.host.service(timeout_ms) {
            Err(error) =>
                Err(error),

            Ok(enet::Event::None) =>
                Err("Server did not reply to our connection wish".to_string()),

            Ok(enet::Event::Connect(_)) =>
                Err("Unexpected enet connect event (already connected)".to_string()),

            Ok(enet::Event::Disconnect(_)) =>
                Err("Got disconnected".to_string()),

            Ok(enet::Event::Receive(_, packet)) => {
                let mut data = packet.data().clone();
                match ServerMessage::read(&mut data) {
                    Ok(ServerMessage::AcceptConnect { your_id: my_id }) => {
                        self.connected = true;
                        self.my_id = Some(my_id);
                        Ok(my_id)
                    },
                    Ok(_) =>
                        Err("Received unexpected message from server while connecting".to_string()),
                    Err(_) => 
                        Err("Received invalid message from server".to_string())
                }
            },
        }
    }

    pub fn service(&mut self) -> Result<Option<ServerMessage>, String> {
        assert!(self.connected);

        match self.host.service(0) {
            Err(error) => Err(error),

            Ok(enet::Event::None) => Ok(None),

            Ok(enet::Event::Connect(_)) =>
                Err("Unexpected enet connect event (already connected)".to_string()),

            Ok(enet::Event::Disconnect(_)) => {
                self.connected = false;
                Err("Got disconnected".to_string())
            },

            Ok(enet::Event::Receive(_, packet)) => {
                let mut data = packet.data().clone();

                match ServerMessage::read(&mut data) {
                    Ok(message) =>
                        Ok(Some(message)),
                    Err(_) =>
                        Err("Received invalid message".to_string())
                }
            }
        }
    }
}

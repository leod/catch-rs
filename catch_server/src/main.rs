#![feature(libc)]

#[macro_use]
extern crate ecs;
extern crate catch_shared as shared;
extern crate renet as enet;
extern crate libc;
extern crate cereal;
extern crate time;

pub mod components;
pub mod services;
pub mod systems;
pub mod state;

use std::collections::HashMap;
use std::io::Read;

use cereal::CerealData;

use shared::net;
use shared::player::{PlayerId, PlayerInfo};
use shared::net::{ClientMessage, ServerMessage};
use state::GameState;

#[derive(PartialEq, Eq, Clone, Copy)]
enum ClientState {
    Connecting,
    Normal,
    Disconnected
}

struct Client {
    peer: enet::Peer,
    state: ClientState,

    ping_sent_time: Option<time::Timespec>,
    ping: Option<time::Duration>,
}

struct Server {
    entity_types: net::EntityTypes,

    host: enet::Host,
    player_id_counter: PlayerId,
    clients: HashMap<PlayerId, Client>,

    game_state: GameState,
}

impl Server {
    fn start(entity_types: net::EntityTypes,
             port: u16,
             peer_count: u32) -> Result<Server, String> {
        let host = try!(enet::Host::new_server(port, peer_count,
                                               net::NUM_CHANNELS as u32,
                                               0, 0));

        println!("Server started");

        Ok(Server {
            entity_types: entity_types,
            host: host,
            player_id_counter: 0,
            clients: HashMap::new(),
            game_state: GameState::new(),
        })
    }

    fn service(&mut self) -> bool {
        let event = self.host.service(0); 
        match event {
            Ok(enet::Event::Connect(peer)) => {
                self.player_id_counter += 1;

                println!("Client {} is connecting", self.player_id_counter);

                assert!(self.clients.get(&self.player_id_counter).is_none());
                peer.set_user_data(self.player_id_counter as *mut libc::c_void);
                self.clients.insert(self.player_id_counter,
                    Client {
                        peer: peer,
                        state: ClientState::Connecting,
                        ping_sent_time: None,
                        ping: None,
                    });

                return true;
            },

            Ok(enet::Event::Disconnect(peer)) => {
                let player_id = peer.get_user_data() as u32; 
                let client_state = self.clients[&player_id].state;

                println!("Client {} disconnected", player_id);

                if client_state == ClientState::Normal {
                    // The client was already fully connected, so tell the other
                    // clients about the disconnection
                    self.broadcast(&ServerMessage::PlayerDisconnect {
                        id: player_id
                    });
                }

                self.clients.remove(&player_id);

                return true;
            },

            Ok(enet::Event::Receive(peer, channel_id, packet)) => {
                let player_id = peer.get_user_data() as u32;
                assert!(self.clients.get(&player_id).is_some());

                if channel_id != net::Channel::Messages as u8 {
                    println!("Received packet on non-message channel from client {}", player_id);
                }
                
                let mut data = packet.data().clone();
                match ClientMessage::read(&mut data) {
                    Ok(message) => 
                        self.process_client_message(player_id, &message),
                    Err(_) => 
                        println!("Received invalid message from client {}", player_id),
                };

                return true;
            },
            
            Ok(enet::Event::None) => return false,

            Err(error) => {
                println!("Error servicing: {}", error);
                return false;
            }
        }
    }

    fn broadcast(&self, message: &ServerMessage) {
        for (_, client) in &self.clients {
            if client.state == ClientState::Normal {
                let mut data = Vec::new();
                match message.write(&mut data) {
                    Err(_) => {
                        println!("Error encoding message {:?}", message);
                        return;
                    },
                    Ok(_) => ()
                }

                client.peer.send(&data, enet::ffi::ENET_PACKET_FLAG_RELIABLE,
                                 net::Channel::Messages as u8);
            }
        }
    }

    fn send(&self, client: &Client, message: &ServerMessage) {
        assert!(client.state == ClientState::Normal);

        let mut data = Vec::new();
        match message.write(&mut data) {
            Err(_) => {
                println!("Error encoding message {:?}", message);
                return;
            },
            Ok(_) => ()
        }

        client.peer.send(&data, enet::ffi::ENET_PACKET_FLAG_RELIABLE,
                         net::Channel::Messages as u8);
    }

    fn process_client_message(&mut self, player_id: PlayerId, message: &ClientMessage) {
        match message {
            &ClientMessage::Pong => {
                println!("Got pong from {}", player_id);
                let client = self.clients.get_mut(&player_id).unwrap();
                
                match client.ping_sent_time {
                    Some(ping_sent_time) =>
                        client.ping = Some(time::get_time() - ping_sent_time),
                    None =>
                        println!("Received unwarranted pong from {}",
                                 player_id),
                };

                client.ping_sent_time = None;
            },

            &ClientMessage::WishConnect { ref name } => {
                let client_state = self.clients[&player_id].state;

                if client_state != ClientState::Connecting {
                    println!("Connected player {} is trying to connect again, ignoring",
                             player_id);
                    return;
                }

                println!("Player {} connected with name {}",
                         player_id, name);

                self.broadcast(&ServerMessage::PlayerConnect {
                    id: player_id,
                    name: name.clone()
                });

                self.clients.get_mut(&player_id).unwrap().state = ClientState::Normal;
                self.send(&self.clients[&player_id],
                          &ServerMessage::AcceptConnect {
                              your_id: player_id
                          });

                let player_info = PlayerInfo::new(player_id, name.clone());
                self.game_state.add_player(player_info);
            }

            &ClientMessage::PlayerInput { ref input } => {
                println!("Received input from {}: {:?}", player_id, input);
            }
        }
    }

    fn run(&mut self) {
        loop {
            self.service();
        }
    }
}

fn main() {
    enet::initialize();

    let entity_types = net::all_entity_types();

    match Server::start(entity_types, 2338, 32).as_mut() {
        Ok(server) =>
            server.run(),

        Err(error) =>
            println!("Couldn't start server: {}", error),
    };
}

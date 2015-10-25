#![feature(libc)]

#[macro_use] extern crate log;
extern crate env_logger;
#[macro_use] extern crate ecs;
#[macro_use] extern crate catch_shared as shared;
extern crate renet as enet;
extern crate libc;
extern crate rustc_serialize;
extern crate bincode;
extern crate time;
extern crate rand;
extern crate hprof;

pub mod components;
pub mod entities;
pub mod services;
pub mod systems;
pub mod state;

use std::collections::HashMap;
use std::thread;
use time::{Duration, Timespec};

use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};

use shared::net;
use shared::{PlayerId, PlayerInfo, TickNumber, GameInfo, Tick};
use shared::net::{ClientMessage, ServerMessage};
use shared::util::PeriodicTimer;
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

    ping_sent_time: Option<Timespec>,
    ping: Option<Duration>,

    // Not adjusted for ping
    at_tick: Option<TickNumber>,
}

struct Server {
    game_info: GameInfo,

    host: enet::Host,
    player_id_counter: PlayerId,
    clients: HashMap<PlayerId, Client>,

    game_state: GameState,

    tick_timer: PeriodicTimer,

    // Statistics and stuff
    print_prof_timer: PeriodicTimer,
    sum_tick_size: usize,
    samples_tick_size: usize,
}

impl Server {
    fn start(game_info: &GameInfo,
             port: u16,
             peer_count: u32) -> Result<Server, String> {
        let host = try!(enet::Host::new_server(port, peer_count,
                                               net::NUM_CHANNELS as u32,
                                               0, 0));

        info!("server started on port {}", port);
        info!("game info: {:?}", game_info);

        let tick_duration_s = 1.0 / (game_info.ticks_per_second as f32);

        Ok(Server {
            game_info: game_info.clone(),
            host: host,
            player_id_counter: 0,
            clients: HashMap::new(),
            game_state: GameState::new(game_info),
            tick_timer: PeriodicTimer::new(tick_duration_s),

            print_prof_timer: PeriodicTimer::new(5.0),
            sum_tick_size: 0,
            samples_tick_size: 0,
        })
    }

    fn tick_time(&self) -> f32 {
        self.game_state.tick_number() as f32 + self.tick_timer.progress()
    }

    fn service(&mut self) -> bool {
        let event = self.host.service(0); 
        match event {
            Ok(enet::Event::Connect(peer)) => {
                self.player_id_counter += 1;

                info!("client {} is connecting", self.player_id_counter);

                assert!(self.clients.get(&self.player_id_counter).is_none());
                peer.set_user_data(self.player_id_counter as *mut libc::c_void);
                self.clients.insert(self.player_id_counter,
                    Client {
                        peer: peer,
                        state: ClientState::Connecting,
                        ping_sent_time: None,
                        ping: None,
                        at_tick: None,
                    });

                return true;
            }
            Ok(enet::Event::Disconnect(peer)) => {
                let player_id = peer.get_user_data() as u32; 
                let client_state = self.clients[&player_id].state;

                info!("client {} disconnected", player_id);

                if client_state == ClientState::Normal {
                    // The client was already fully connected, so tell the other
                    // clients about the disconnection
                    self.broadcast(&ServerMessage::PlayerDisconnect {
                        id: player_id
                    });
                }

                self.clients.remove(&player_id);
                self.game_state.remove_player(player_id);

                return true;
            }
            Ok(enet::Event::Receive(peer, channel_id, packet)) => {
                let player_id = peer.get_user_data() as u32;
                assert!(self.clients.get(&player_id).is_some());

                if channel_id != net::Channel::Messages as u8 {
                    warn!("received packet on non-message channel from client {}", player_id);
                }
                
                match decode(&packet.data()) {
                    Ok(message) => 
                        self.process_client_message(player_id, &message),
                    Err(_) => 
                        warn!("received invalid message from client {}", player_id),
                };

                return true;
            }
            Ok(enet::Event::None) => return false,
            Err(error) => {
                warn!("error servicing: {}", error);
                return false;
            }
        }
    }

    fn broadcast(&self, message: &ServerMessage) {
        for (_, client) in &self.clients {
            if client.state == ClientState::Normal {
                let data = encode(message, SizeLimit::Infinite).unwrap();
                client.peer.send(&data, enet::ffi::ENET_PACKET_FLAG_RELIABLE,
                                 net::Channel::Messages as u8);
            }
        }
    }

    fn send(&self, client: &Client, message: &ServerMessage) {
        //print!("sending message {:?}", message);
        assert!(client.state == ClientState::Normal);

        let data = encode(message, SizeLimit::Infinite).unwrap();
        client.peer.send(&data, enet::ffi::ENET_PACKET_FLAG_RELIABLE,
                         net::Channel::Messages as u8);
    }

    fn process_client_message(&mut self, player_id: PlayerId, message: &ClientMessage) {
        match message {
            &ClientMessage::Pong => {
                debug!("got pong from {}", player_id);
                let client = self.clients.get_mut(&player_id).unwrap();
                
                match client.ping_sent_time {
                    Some(ping_sent_time) =>
                        client.ping = Some(time::get_time() - ping_sent_time),
                    None =>
                        warn!("received unwarranted pong from {}",
                                 player_id)
                };

                client.ping_sent_time = None;
            }
            &ClientMessage::WishConnect { ref name } => {
                let client_state = self.clients[&player_id].state;

                if client_state != ClientState::Connecting {
                    warn!("connected player {} is trying to connect again, ignoring",
                          player_id);
                    return;
                }

                info!("player {} connected with name {}", player_id, name);

                self.broadcast(&ServerMessage::PlayerConnect {
                    id: player_id,
                    name: name.clone()
                });

                self.clients.get_mut(&player_id).unwrap().state = ClientState::Normal;
                self.send(&self.clients[&player_id],
                          &ServerMessage::AcceptConnect {
                              your_id: player_id,
                              game_info: self.game_info.clone(),
                          });

                let player_info = PlayerInfo::new(player_id, name.clone());
                self.game_state.add_player(player_info);
            }
            &ClientMessage::PlayerInput(ref input)  => {
                self.game_state.on_player_input(player_id, input);
            }
            &ClientMessage::StartingTick { ref tick } => {
                self.clients.get_mut(&player_id).unwrap().at_tick = Some(*tick);
            }
        }
    }

    fn run(&mut self) {
        loop {
            let start_time_s = time::precise_time_s() as f32;

            hprof::start_frame();

            {
                let _g = hprof::enter("service");

                // Is this how DDOS happens?
                while self.service() {};
            }

            {
                let _g = hprof::enter("ticks");

                // Start ticks
                while self.tick_timer.next() {
                    self.tick();
                }
            }

            {
                let _g = hprof::enter("sleep");
                thread::sleep_ms(1);
            }

            hprof::end_frame();

            let end_time_s = time::precise_time_s() as f32;

            let delta_s = end_time_s - start_time_s;
            self.tick_timer.add(delta_s);
            self.print_prof_timer.add(delta_s);

            if self.print_prof_timer.next_reset() {
                hprof::profiler().print_timing();  

                if self.samples_tick_size > 0 {
                    info!("average tick size over last {} ticks: {:.2} bytes, {:.2} kb/s",
                          self.samples_tick_size,
                          self.sum_tick_size as f64 / self.samples_tick_size as f64,
                          self.sum_tick_size as f64 / (1000.0 * 5.0));
                }
            }
        }
    }

    fn tick(&mut self) {
        self.game_state.tick();
        
        // Broadcast tick to clients
        let _g = hprof::enter("broadcast");

        for (player_id, client) in self.clients.iter() {
            if client.state == ClientState::Normal {
                // Build tick for each client separately. This makes it possible to do
                // delta encoding and stuff.
                let _g = hprof::enter("store");
                let tick_number = self.game_state.tick_number;

                let mut tick = Tick::new(tick_number);
                tick.events = self.game_state.world.services.next_player_events[player_id]
                                  .clone();

                self.game_state.world.systems.net_entity_system
                    .store_in_tick_state(*player_id, &mut tick.state,
                                         &mut self.game_state.world.data);
                drop(_g);
                let _g = hprof::enter("encode");

                let data = encode(&tick, SizeLimit::Infinite).unwrap();

                drop(_g);
                let _g = hprof::enter("send");

                self.sum_tick_size += data.len();
                self.samples_tick_size += 1;

                client.peer.send(&data, enet::ffi::ENET_PACKET_FLAG_RELIABLE,
                                 net::Channel::Ticks as u8);

                self.game_state.world.services.next_player_events
                    .get_mut(&player_id).unwrap().clear();
            }
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    enet::initialize().unwrap();

    let entity_types = shared::entities::all_entity_types();
    let game_info = GameInfo {
        map_name: "data/maps/desert.tmx".to_string(),
        entity_types: entity_types,
        ticks_per_second: 30,
    };

    match Server::start(&game_info, 9988, 32).as_mut() {
        Ok(server) =>
            server.run(),
        Err(error) =>
            error!("Couldn't start server: {}", error),
    };
}

use std::collections::VecDeque;
use time;

use enet;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode, decode_from};

use shared::net;
use shared::net::{ClientMessage, ServerMessage};
use shared::{GameInfo, PlayerId, Tick, TickNumber};

pub struct Client {
    host: enet::Host,
    server_peer: enet::Peer,
    connected: bool,

    my_name: String,
    my_id: Option<PlayerId>,

    game_info: Option<GameInfo>,

    // Ticks received from the server together with the time at which they were received
    tick_deque: VecDeque<(time::Timespec, Tick)>,

    last_tick: Option<Tick>,
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
                                     net::NUM_CHANNELS as u32,
                                     0, 0));

        Ok(Client {
            host: host,
            server_peer: server_peer,
            connected: false,
            my_name: my_name,
            my_id: None,
            game_info: None,
            tick_deque: VecDeque::new(),
            last_tick: None,
        })
    }

    pub fn send(&self, message: &ClientMessage) {
        let data: Vec<u8> = encode(message, SizeLimit::Infinite).unwrap();
        self.server_peer.send(&data, enet::ffi::ENET_PACKET_FLAG_RELIABLE, 0);
    }

    pub fn my_id(&self) -> PlayerId {
        self.my_id.unwrap()
    }

    pub fn game_info(&self) -> &GameInfo {
        self.game_info.as_ref().unwrap()
    }

    pub fn num_ticks(&self) -> usize {
        self.tick_deque.len()         
    }

    pub fn get_tick(&self, i: usize) -> &(time::Timespec, Tick) {
        &self.tick_deque[i]
    }

    pub fn get_next_tick(&mut self) -> &(time::Timespec, Tick) {
        &self.tick_deque.front().unwrap()
    }

    pub fn pop_next_tick(&mut self) -> (time::Timespec, Tick) {
        self.tick_deque.pop_front().unwrap() 
    }

    pub fn finish_connecting(&mut self, timeout_ms: u32) -> Result<(), String> {
        assert!(!self.connected);

        self.send(&ClientMessage::WishConnect {
            name: self.my_name.clone()
        });

        // Wait for an AcceptConnect reply to our WishConnect
        match self.host.service(timeout_ms) {
            Err(error) =>
                Err(error),
            Ok(enet::Event::None) =>
                Err("Server did not reply to our connection wish".to_string()),
            Ok(enet::Event::Connect(_)) =>
                Err("Unexpected enet connect event (already connected)".to_string()),
            Ok(enet::Event::Disconnect(_)) =>
                Err("Got disconnected".to_string()),
            Ok(enet::Event::Receive(_, channel_id, packet)) => {
                if channel_id != net::Channel::Messages as u8 {
                    return Err("Received tick data while not yet fully connected".to_string());
                }

                match decode(&packet.data()) {
                    Ok(ServerMessage::AcceptConnect { your_id: my_id, game_info }) => {
                        self.connected = true;
                        self.my_id = Some(my_id);
                        self.game_info = Some(game_info);

                        Ok(())
                    }
                    Ok(_) =>
                        Err("Received unexpected message from server while connecting".to_string()),
                    Err(_) => 
                        Err("Received invalid message from server".to_string())
                }
            }
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
            }
            Ok(enet::Event::Receive(_, channel_id, packet)) => {
                if channel_id == net::Channel::Messages as u8 {
                    match decode(&packet.data()) {
                        Ok(message) => {
                            //println!("Received message {:?}", message);
                            Ok(Some(message))
                        }
                        Err(_) =>
                            Err("Received invalid message".to_string())
                    }
                } else if channel_id == net::Channel::Ticks as u8 {
                    //println!("Received tick of size {}: {:?}", data.len(), &data);

                    let mut data = packet.data().clone(); // TODO: clone
                    let delta_tick: Option<TickNumber> =
                        decode_from(&mut data, SizeLimit::Infinite).unwrap();

                    match decode_from(&mut data, SizeLimit::Infinite) {
                        Ok(tick) => {
                            if let Some(delta_tick) = delta_tick {
                                assert!(self.last_tick.as_ref().unwrap().tick_number == delta_tick);
                                self.last_tick.as_mut().unwrap().load_delta(&tick);
                            } else {
                                self.last_tick = Some(tick); // TODO: clone
                            }
                            self.tick_deque.push_back((time::get_time(),
                                                       self.last_tick.clone().unwrap()));

                        }
                        Err(_) =>
                            return Err("Received invalid tick".to_string())
                    };
                    
                    // We received a tick, but still need a Option<ServerMessage>... kind of awkward
                    self.service()
                } else {
                    Err("Invalid channel id".to_string())
                }
            }
        }
    }
}

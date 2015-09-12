use std::collections::HashMap;

use ecs::ServiceManager;

use shared::net;
use shared::tick::Tick;
use shared::player::PlayerId;
use shared::event::GameEvent;

pub struct Services {
    // Tick duration in seconds
    pub tick_dur_s: f64,

    // Stores the state of the current tick before sending it off to clients
    //pub next_tick: Option<Tick>,

    // Events generated in a tick that are to be performed on the server as well
    // as sent to the clients
    pub next_events: Vec<GameEvent>,
    
    // Game events for the current tick that are to be sent to clients
    // are stored in `next_player_events`.
    // Each event in `next_events` is also stored for each player here.
    pub next_player_events: HashMap<PlayerId, Vec<GameEvent>>,
}

impl Services {
    pub fn prepare_for_tick<T: Iterator<Item=PlayerId>>(&mut self, number: net::TickNumber, player_ids: T) {
        //self.next_tick = Some(Tick::new(number));     
        assert!(self.next_events.is_empty());
        for (_, ref events) in self.next_player_events.iter() {
            assert!(events.is_empty());
        }

        self.next_player_events.clear();
        for player_id in player_ids {
            self.next_player_events.insert(player_id, Vec::new()); 
        }
    }

    pub fn add_event(&mut self, event: &GameEvent) {
        //self.next_tick.as_mut().unwrap().events.push(event.clone());

        // Send event to every player
        let player_ids = self.next_player_events.keys().map(|k| *k)
                             .collect::<Vec<_>>();

        for player_id in player_ids.iter() {
            self.next_player_events.get_mut(player_id).unwrap()
                .push(event.clone());
        }
    }

    pub fn add_event_to_run(&mut self, event: &GameEvent) {
        self.add_event(&event);
        self.next_events.push(event.clone());
    }
    
    pub fn add_player_event(&mut self, player_id: PlayerId, event: GameEvent) {
        self.next_player_events.get_mut(&player_id).unwrap().push(event);
    }
}

impl Default for Services {
    fn default() -> Services {
        Services {
            tick_dur_s: 0.0,
            //next_tick: None,
            next_events: Vec::new(),
            next_player_events: HashMap::new(),
        }
    }
}

impl ServiceManager for Services {}

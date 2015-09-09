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
    pub next_tick: Option<Tick>,
    
    // Game events for the current tick that are to be sent only to specific clients
    // can be stored in `next_player_events`
    pub next_player_events: HashMap<PlayerId, Vec<GameEvent>>,
}

impl Services {
    pub fn prepare_for_tick<T: Iterator<Item=PlayerId>>(&mut self, number: net::TickNumber, player_ids: T) {
        self.next_tick = Some(Tick::new(number));     

        self.next_player_events.clear();
        for player_id in player_ids {
            self.next_player_events.insert(player_id, Vec::new()); 
        }
    }

    pub fn add_event(&mut self, event: GameEvent) {
        self.next_tick.as_mut().unwrap().events.push(event);
    }
    
    pub fn add_player_event(&mut self, player_id: PlayerId, event: GameEvent) {
        self.next_player_events.get_mut(&player_id).unwrap().push(event);
    }
}

impl Default for Services {
    fn default() -> Services {
        Services {
            tick_dur_s: 0.0,
            next_tick: None,
            next_player_events: HashMap::new(),
        }
    }
}

impl ServiceManager for Services {}

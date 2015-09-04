use std::collections::HashMap;
use std::io::{Read};

use cereal::{CerealData, CerealError, CerealResult};

use components::{Position, Orientation, PlayerState};
use event::GameEvent;
use net;

/// Stores the state of all net components in a tick
pub type ComponentsNetState<T> = HashMap<net::EntityId, T>;

#[derive(CerealData)]
pub struct NetState {
    pub position: ComponentsNetState<Position>, 
    pub orientation: ComponentsNetState<Orientation>,
    pub player_state: ComponentsNetState<PlayerState>,
}

#[derive(CerealData)]
pub struct Tick {
    pub tick_number: net::TickNumber,
    pub events: Vec<GameEvent>,
    pub net_state: NetState,
}

impl NetState {
    pub fn new() -> NetState {
        NetState {
            position: HashMap::new(),
            orientation: HashMap::new(),
            player_state: HashMap::new(),
        }
    }
}

impl Tick {
    pub fn new(tick_number: net::TickNumber) -> Tick {
        Tick {
            tick_number: tick_number,
            events: Vec::new(),
            net_state: NetState::new(),
        }
    }
}

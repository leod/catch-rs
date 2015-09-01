use std::collections::HashMap;
use std::io::{Read};

use cereal::{CerealData, CerealError, CerealResult};

use components::{Position};
use event::GameEvent;
use net;

/// Stores the state of all net components in a tick
pub type ComponentsNetState<T> = HashMap<net::EntityId, T>;
pub struct NetState {
    pub position: ComponentsNetState<Position>, 
}

pub struct Tick {
    pub tick_number: net::TickNumber,
    pub events: Vec<GameEvent>,
    pub net_state: NetState,
}

impl NetState {
    fn new() -> NetState {
        NetState {
            position: HashMap::new(),
        }
    }
}

impl Tick {
    pub fn read(entity_types: &net::EntityTypes, r: &mut Read) -> CerealResult<Tick> {
        let tick_number: net::TickNumber = try!(CerealData::read(r));
        let events: Vec<GameEvent> = try!(CerealData::read(r));
        let num_entities: i32 = try!(CerealData::read(r));
        let mut net_state = NetState::new();

        for _ in 0..num_entities {
            let entity_id: net::EntityId = try!(CerealData::read(r));
            let entity_type_id: net::EntityTypeId = try!(CerealData::read(r)); 
            let entity_type: &net::EntityType = match entity_types.get(entity_type_id as usize) {
                Some(&(ref name, ref entity_type)) =>
                    entity_type,
                None =>
                    return Err(CerealError::Msg("Invalid entity type id in net state".to_string())),
            };

            for component_type in &entity_type.component_types {
                match *component_type {
                    net::ComponentType::Position =>
                        net_state.position.insert(entity_id, try!(CerealData::read(r))),
                };
            }
        }

        Ok(Tick {
            tick_number: tick_number,
            events: events,
            net_state: net_state
        })
    }
}

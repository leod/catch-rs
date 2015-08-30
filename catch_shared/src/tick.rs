use std::collections::HashMap;
use std::io::{Read, Write};

use ecs::World;
use cereal::{CerealData, CerealError, CerealResult};

use components::{Position};
use net::{NetEntityId, NetEntityTypeId, TickNumber, NetEntityType, NetEntityTypes};
use event::GameEvent;

pub type TickNumber = i32;

/// Stores the state of all net components in a tick
pub struct ComponentsNetState<T>(HashMap<NetEntityId, T>);
struct NetState {
    position: ComponentsNetState<Position>, 
}

struct Tick {
    tick_number: TickNumber,
    events: Vec<GameEvent>,
    net_state: NetState,
}

impl NetState {
    fn new() -> NetState {
        NetState {
            position: HashMap::new(),
        }
    }
}

impl Tick {
    fn read(types: &NetEntityTypes, r: &mut Read) -> CerealResult<Tick> {
        let tick_number: TickNumber = try!(CerealData::read(r));
        let events: Vec<GameEvent> = try!(CerealData::read(r));
        let num_entities: i32 = try!(CerealData::read(r));
        let mut net_state = NetState::new();

        for i in 0..num_entities {
            let net_entity_id: NetEntityId = try!(CerealData::read(r));
            let net_entity_type_id: NetEntityTypeId = try!(CerealData::read(r)); 
            let net_entity_type = try!(types.get(&net_entity_type_id));

            for net_component_type in net_entity_type.net_component_types {
                match net_component_type {
                    Position => net_state.position.insert(net_entity_id, CerealData::read(r)),
                }
            }
        }

        TickState {
            tick_number: tick_number,
            events: events,
            net_state: net_state
        }
    }

    fn write(&self, types: &NetEntityTypes, w: &mut Write) -> CerealResult<()> {
        try!(CerealData::write(self.tick_number));
        try!(CerealData::write(self.events));
        try!(CerealData::write(self.
    }
}

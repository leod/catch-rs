use std::collections::HashMap;
use std::io::{Read, Write};

use ecs::World;
use cereal::{CerealData, CerealError, CerealResult};

use components::{Position};
use net::{NetEntityId, NetEntityTypeId, TickNumber, NetEntityType, NetEntityTypes};
use event::GameEvent;

/// Stores the state of all net components in a tick
struct NetState {
    position: ComponentsNetState<Position>, 
}

pub struct ComponentsNetState<T>(HashMap<NetEntityId, T>);

struct TickState {
    tick_number: TickNumber,
    events: Vec<GameEvent>,
    net_state: NetState,
}

impl TickState {
    fn read(types: &NetEntityTypes, r: &mut Read) -> CerealResult<TickState> {
        let tick_number: TickNumber = try!(CerealData::read(r));
        let events: Vec<GameEvent> = try!(CerealData::read(r));
        let num_entities: i32 = try!(CerealData::read(r));
        let mut net_state: NetState = NetState {
            position: HashMap::new(),
        };

        for i in 0..num_entities {
            let net_entity_id: NetEntityId = try!(CerealData::read(r));
            let net_entity_type_id: NetEntityTypeId = try!(CerealData::read(r)); 
            let net_entity_type = try!(types.get(&net_entity_type_id));

            match net_entity_type {
                Position => net_state.position.insert(net_entity_id, CerealData::read(r)),
            }
        }

        return TickState {
            tick_number: tick_number,
            events: events,
            net_state: net_state
        };
    }
}

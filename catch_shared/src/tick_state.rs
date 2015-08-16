use std::io::{Read, Write};

use ecs::World;

use components::{Position};
use {NetStateComponent, NetEntityId, TickNumber};
use cereal::{CerealData, CerealError, CerealResult};

/// Stores the state of all net components in a tick
struct NetState {
    position: ComponentsNetState<Position>, 
}

pub struct ComponentsNetState<T: NetData>(HashMap<NetEntityId, T>);

struct TickState {
    tick_number: TickNumber,
    events: Vec<GameEvent>,
    net_state: NetState,
}

impl TickState {
    fn read(r: &mut Read) -> CerealResult<TickState> {
        let tick_number: TickNumber = try!(CerealData::read(r));
        let events: Vec<GameEvent> = try!(CerealData::read(r));
        let num_entities: i32 = try!(CerealData::read(r));

        for i in 0..num_entities {
            let net_entity_id: NetEntityId = try!(CerealData::read(r));

        }
    }

    fn write_world_tick_state(w: &mut Write, EntityIter<>) -> CerealResult<()> {
          
    }
}

use std::iter::Iterator;

use rustc_serialize::{Encoder, Decoder, Encodable, Decodable};

use net_components::{NetComponents, ComponentType};
use super::{EntityId, TickNumber, GameEvent};

/// Stores the state of net components in a tick
pub type TickEntities = Vec<(EntityId, NetComponents)>;

#[derive(Default)]
pub struct TickState {
    // Should always be ordered by EntityId (ascending)
    pub entities: TickEntities,

    // List of components that should not be interpolated into this tick
    // (e.g. you wouldn't want to interpolate the position of a player that was just teleported)
    pub forced_components: Vec<(EntityId, ComponentType)>,
}

pub struct Tick {
    pub tick_number: TickNumber,
    pub events: Vec<GameEvent>,
    pub state: TickState,
}

impl TickState {
    pub fn iter_pairs<'a>(&'a self, other_state: &'a TickState) -> EntityPairIterator<'a> {
        EntityPairIterator {
            a: &self.entities, b: &other_state.entities,
            i: 0, j: 0,
        }
    }

    pub fn sort(&mut self) {
        self.entities.sort_by(|a, b| {
            a.0.cmp(&b.0)
        });
    }

    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_seq(self.entities.len(), |s| {
            for &(id, ref e) in &self.entities {
                try!(s.emit_u32(id));
                try!(e.encode(s));
            }
            self.forced_components.encode(s)
        })
    }

    fn decode<D: Decoder>(d: &mut D) -> Result<TickState, D::Error> {
        d.read_seq(|d, len| {
            let mut entities = Vec::with_capacity(len);
            for _ in 0..len {
                let id = try!(d.read_u32());
                let e = try!(NetComponents::decode(d));
                entities.push((id, e));
            }

            let forced_components = try!(Vec::<(EntityId, ComponentType)>::decode(d));

            Ok(TickState {
                entities: entities,
                forced_components: forced_components,
            })
        })
    }
}

impl Tick {
    pub fn new(tick_number: TickNumber) -> Tick {
        Tick {
            tick_number: tick_number,
            events: Vec::new(),
            state: TickState::default(),
        }
    }
}

impl Encodable for Tick {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        try!(self.tick_number.encode(s));
        try!(self.events.encode(s));
        try!(self.state.encode(s));
        Ok(())
    }
}

impl Decodable for Tick {
    fn decode<D: Decoder>(d: &mut D) -> Result<Tick, D::Error> {
        let tick_number = try!(TickNumber::decode(d));
        let events = try!(Vec::<GameEvent>::decode(d));
        let state = try!(TickState::decode(d));

        Ok(Tick {
            tick_number: tick_number,
            events: events,
            state: state,
        })
    }
}

pub struct EntityPairIterator<'a> {
    a: &'a TickEntities,
    b: &'a TickEntities,

    i: usize,
    j: usize,
}

pub enum EntityPair {
    // Entity is present only in tick A
    OnlyA(usize),

    // Entity is present only in tick B
    OnlyB(usize),

    // Entity is present in both ticks
    Both(usize, usize),
}

impl<'a> Iterator for EntityPairIterator<'a> {
    type Item = EntityPair;

    fn next(&mut self) -> Option<EntityPair> {
        if self.i == self.a.len() && self.j == self.b.len() {
            // Reached both ends
            None
        } else if self.j == self.b.len() ||
                  (self.i < self.a.len() && self.a[self.i].0 < self.b[self.j].0) {
            // Tick B is ahead in entity ids
            let item = EntityPair::OnlyA(self.i);
            self.i += 1;
            Some(item)
        } else if self.i == self.a.len() ||
                  (self.j < self.b.len() && self.a[self.i].0 > self.b[self.j].0) {
            // Tick A is ahead in entity ids 
            let item = EntityPair::OnlyB(self.j);
            self.j += 1;
            Some(item)
        } else {
            // We have a match
            assert!(self.a[self.i].0 == self.b[self.j].0);
            let item = EntityPair::Both(self.i, self.j);
            self.i += 1;
            self.j += 1;
            Some(item)
        }
    }
}

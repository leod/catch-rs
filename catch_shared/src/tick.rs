use std::mem;
use std::iter::Iterator;

use rustc_serialize::{Encoder, Decoder, Encodable, Decodable};

use net_components::{NetComponents, ComponentType};
use super::{EntityId, TickNumber, GameEvent};

/// Stores the state of net components in a tick
pub type TickEntities = Vec<(EntityId, NetComponents)>;

#[derive(Default, Clone)]
pub struct TickState {
    // Should always be ordered by EntityId (ascending)
    pub entities: TickEntities,

    // List of components that should not be interpolated into this tick
    // (e.g. you wouldn't want to interpolate the position of a player that was just teleported)
    pub forced_components: Vec<(EntityId, ComponentType)>,
}

#[derive(Clone)]
pub struct Tick {
    pub tick_number: TickNumber,
    pub events: Vec<GameEvent>,
    pub state: TickState,
}

pub struct DeltaEncodeTick<'a> {
    pub last_tick: &'a Tick,
    pub tick: &'a Tick,
}

impl TickState {
    pub fn iter_pairs<'a>(&'a self, other_state: &'a TickState) -> EntityPairIterator<'a> {
        EntityPairIterator {
            a: &self.entities, b: &other_state.entities,
            i: 0, j: 0,
        }
    }

    pub fn iter_pairs_mut<'a>(&'a mut self, other_state: &'a TickState)
                              -> EntityPairIteratorMut<'a> {
        EntityPairIteratorMut {
            a: &mut self.entities[..], b: &other_state.entities, j: 0,
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
            Ok(())
        });

        self.forced_components.encode(s)
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

    fn delta_encode<S: Encoder>(&self, last_state: &TickState, s: &mut S) -> Result<(), S::Error> {
        // Check which components changed between this tick and the last
        let mut neq_components = Vec::new();
        let mut len = 0;
        for (id, pair) in last_state.iter_pairs(self) {
            match pair {
                EntityPair::OnlyB(_) => {
                    len += 1;
                }
                EntityPair::Both(e_last, e) => {
                    let bit_set = e.neq_components(e_last);
                    neq_components.push(bit_set);
                    if bit_set > 0 {
                        len += 1;
                    }
                }
                _ => {}
            }
        }

        s.emit_seq(len, |s| {
            let mut index = 0;
            for (id, pair) in last_state.iter_pairs(self) {
                match pair {
                    EntityPair::OnlyA(_) => {
                        // Entity stopped existing
                    }
                    EntityPair::OnlyB(e) => {
                        // New entity
                        try!(s.emit_u32(id));
                        try!(e.encode(s));
                    }
                    EntityPair::Both(e_last, e) => {
                        // Delta encode
                        if neq_components[index] > 0 {
                            try!(s.emit_u32(id));
                            try!(e.delta_encode(neq_components[index], s));
                        }
                        index += 1;
                    }
                }
            }
            Ok(())
        });

        self.forced_components.encode(s)
    }

    pub fn load_delta(&mut self, new_state: &TickState) {
        self.forced_components = new_state.forced_components.clone();

        let mut to_remove: Vec<EntityId> = Vec::new();
        let mut to_add: Vec<(EntityId, NetComponents)> = Vec::new();

        for (id, pair) in self.iter_pairs_mut(new_state) {
            match pair {
                EntityPairMut::OnlyA(_) => {
                }
                EntityPairMut::OnlyB(components) => {
                    // New entity, add it after this loop
                    to_add.push((id, components.clone()));
                }
                EntityPairMut::Both(components, new_components) => {
                    // Delta update components
                    components.load_delta(new_components);
                }
            }
        }

        for &remove_id in &to_remove {
        }

        for &(add_id, ref add_components) in &to_add {
            self.entities.push((add_id, add_components.clone()));
        }

        if to_add.len() > 0 {
            self.sort();
        }
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

    pub fn load_delta(&mut self, new_tick: &Tick) {
        self.tick_number = new_tick.tick_number;
        self.events = new_tick.events.clone();
        self.state.load_delta(&new_tick.state);

        for event in &self.events {
            match event {
                &GameEvent::RemoveEntity(remove_id) => {
                    let index =
                        self.state.entities.iter().position(|&(id, _)| id == remove_id).unwrap();
                    self.state.entities.remove(index);
                }
                _ => {}
            }
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

impl<'a> Encodable for DeltaEncodeTick<'a> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        try!(self.tick.tick_number.encode(s));
        try!(self.tick.events.encode(s));
        try!(self.tick.state.delta_encode(&self.last_tick.state, s));
        Ok(())
    }
}

pub struct EntityPairIterator<'a> {
    a: &'a TickEntities,
    b: &'a TickEntities,

    i: usize,
    j: usize,
}

pub enum EntityPair<'a> {
    // Entity is present only in tick A
    OnlyA(&'a NetComponents),

    // Entity is present only in tick B
    OnlyB(&'a NetComponents),

    // Entity is present in both ticks
    Both(&'a NetComponents, &'a NetComponents),
}

impl<'a> Iterator for EntityPairIterator<'a> {
    type Item = (EntityId, EntityPair<'a>);

    fn next(&mut self) -> Option<(EntityId, EntityPair<'a>)> {
        if self.i == self.a.len() && self.j == self.b.len() {
            // Reached both ends
            None
        } else if self.j == self.b.len() ||
                  (self.i < self.a.len() && self.a[self.i].0 < self.b[self.j].0) {
            // Tick B is ahead in entity ids
            let item = (self.a[self.i].0, EntityPair::OnlyA(&self.a[self.i].1));
            self.i += 1;
            Some(item)
        } else if self.i == self.a.len() ||
                  (self.j < self.b.len() && self.a[self.i].0 > self.b[self.j].0) {
            // Tick A is ahead in entity ids 
            let item = (self.b[self.j].0, EntityPair::OnlyB(&self.b[self.j].1));
            self.j += 1;
            Some(item)
        } else {
            // We have a match
            assert!(self.a[self.i].0 == self.b[self.j].0);
            let item = (self.a[self.i].0, EntityPair::Both(&self.a[self.i].1, &self.b[self.j].1));
            self.i += 1;
            self.j += 1;
            Some(item)
        }
    }
}

pub struct EntityPairIteratorMut<'a> {
    a: &'a mut [(EntityId, NetComponents)],
    b: &'a TickEntities,

    j: usize,
}

pub enum EntityPairMut<'a> {
    // Entity is present only in tick A
    OnlyA(&'a mut NetComponents),

    // Entity is present only in tick B
    OnlyB(&'a NetComponents),

    // Entity is present in both ticks
    Both(&'a mut NetComponents, &'a NetComponents),
}

impl<'a> Iterator for EntityPairIteratorMut<'a> {
    type Item = (EntityId, EntityPairMut<'a>);

    fn next(&mut self) -> Option<(EntityId, EntityPairMut<'a>)> {
        // https://github.com/rust-lang/rust/blob/master/src/doc/nomicon/borrow-splitting.md

        if self.a.len() == 0 && self.j == self.b.len() {
            // Reached both ends
            None
        } else if self.j == self.b.len() ||
                  (self.a.len() > 0 && self.a[0].0 < self.b[self.j].0) {
            // Tick B is ahead in entity ids
            let slice = mem::replace(&mut self.a, &mut []);
            let (l, r) = slice.split_at_mut(1);
            self.a = r;
            Some((l[0].0, EntityPairMut::OnlyA(&mut l[0].1)))
        } else if self.a.len() == 0 ||
                  (self.j < self.b.len() && self.a[0].0 > self.b[self.j].0) {
            // Tick A is ahead in entity ids 
            let item = (self.b[self.j].0, EntityPairMut::OnlyB(&self.b[self.j].1));
            self.j += 1;
            Some(item)
        } else {
            // We have a match
            assert!(self.a[0].0 == self.b[self.j].0);
            let slice = mem::replace(&mut self.a, &mut []);
            let (l, r) = slice.split_at_mut(1);
            self.a = r;
            self.j += 1;
            Some((self.b[self.j-1].0,
                  EntityPairMut::Both(&mut l[0].1, &self.b[self.j-1].1)))
        }
    }
}

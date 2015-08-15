use std::collections::HashMap;
use std::io::{Read, Write};

use cereal::{CerealData, CerealError, CerealResult};
use esc::{Aspect, EntityData, DataHelper};
use esc::system::System;

pub type NetEntityId = i32;

pub struct NetComponent {
    pub id: NetEntityId,
}

pub trait NetStateType: CerealData {
    fn new();
}

pub trait InterpNetStateType: NetStateType {
    fn interpolate(&mut self, a: &Self, b: &Self, time: f32);
}

pub struct NetStateComponent<T: NetStateType> {
}

pub struct NetState<T: NetStateType>(HashMap<NetEntityId, T>);

impl NetState<T: NetStateType> {
    
}

pub struct InterpNetStateComponent<T: InterpNetStateType> {
    state_a: T,
    state_b: T
}

pub struct NetStateSystem<Components, Systems, T: NetState, FGet: Fn(EntityData<Components>, &Components) -> &T, FGetMut: Fn(EntityData<Components>, &mut Components) -> &mut T> {
    type Components = Components,
    type Systems = Systems,

    aspect: Aspect<Components>,

    f_get: FGet,
    f_get_mut: FGetMut,
}

pub struct InterpNetStateSystem<Components, Systems, T: InterpNetState> {
    aspect: Aspect<Components>,
}

impl InterpNetStateSystem<Components, Systems, T: InterpNetState> {
    pub fn new(aspect: Aspect<Components>) {
        InterpNetStateSystem {
            aspect: aspect 
        }
    }

    pub fn interpolate(&self, t: f32, entities: &mut DataHelper<Components, System>, interp_net_state: &mut ComponentList<Components, T>) {
        for e in data.entities().filter(aspect) {
            interp_net_state[e] 
        }
    }
}


impl NetStateSystem<Components, Systems, T: NetStateType, FGet: Fn(EntityData<Components>, &Components) -> &T, FGetMut: Fn(EntityData<Components>, &mut Components) -> &mut T> {
    pub fn new(aspect: Aspect<Components>, f_get: FGet, f_get_mut: FGetMut) -> NetStateSystem<Components, Systems, T> {
        NetStateSystem {
            aspect: aspect,
            f_get: f_get,
            f_get_mut: f_get_mut
        }
    }

    pub fn write_entity(&self, w: &mut Write, entity: EntityData<Components>, data: &Components) {
        try!(f_get(entity, data).write(w));
    }

    pub fn read_entity(&self, r: &mut Read, entity: EntityData<Components>, data: &mut Components) {
        try!(f_get(entity, data).write(w));
    }

    fn load_tick

    pub fn interpolate(&self, data: &mut DataHelper<Components, Systems>) {
        for data.entities().filter(aspect) {

        }
    }
}



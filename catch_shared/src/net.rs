use std::collections::HashMap;
use std::io::{Read, Write};

use cereal::{CerealData, CerealError, CerealResult};
use esc::{Aspect, EntityData, DataHelper};
use esc::system::System;

pub type NetEntityId = i32;
pub type PlayerId = i32;
pub type TickNumber = i32;

/// Describes a net entity type

/// Every entity that wants its component state synchronized needs to have this component
pub struct NetEntityComponent {
    pub id: NetEntityId,
}

/// Components that are to be sent over the network should have a
/// meta type implementing this trait
pub trait NetStateType {
}

/// The component state can also be interpolated
pub trait Interp {
    fn interp(&Self, &Self, f32) -> Self;
}

#![plugin(cereal_macros)]
#![feature(custom_derive, plugin)]

#[macro_use]
extern crate ecs;

//#[macro_use]
extern crate cereal;

pub mod net;
pub mod event;
pub mod components;
pub mod tick;
pub mod player;

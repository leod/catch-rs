#![plugin(cereal_macros)]
#![feature(custom_derive, plugin)]

extern crate time;
extern crate cereal;
#[macro_use] extern crate ecs;
extern crate tiled;
extern crate vecmath as vecmath_lib;

pub mod net;
pub mod event;
pub mod components;
pub mod tick;
pub mod player;
pub mod util;
pub mod map;
pub mod math;
pub mod item;

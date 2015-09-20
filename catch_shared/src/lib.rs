#![plugin(cereal_macros)]
#![feature(custom_derive, plugin)]

extern crate time;
extern crate cereal;
#[macro_use] extern crate ecs;
extern crate tiled;
extern crate vecmath as vecmath_lib;

pub mod net;
pub mod components;
pub mod tick;
pub mod player;
#[macro_use] pub mod util;
pub mod map;
pub mod math;
pub mod entities;

pub use map::Map;
pub use tick::{TickState, Tick};
pub use net::{EntityTypes};
pub use player::{Item, PlayerInputKey, PlayerInput, PlayerInfo};

pub type EntityId = u32;
pub type EntityTypeId = u32;

pub type PlayerId = u32;
pub type PlayerInputNumber = u32;
pub type ItemSlot = u32;
pub const NEUTRAL_PLAYER_ID: PlayerId = 0;
pub const NUM_ITEM_SLOTS: ItemSlot = 3;

pub type TickNumber = u32;

#[derive(Debug, Clone, CerealData)]
pub struct GameInfo {
    pub map_name: String,
    pub entity_types: net::EntityTypes,
    pub ticks_per_second: u32,
}

#[derive(Debug, Clone, CerealData)]
pub enum GameEvent {
    PlayerJoin(PlayerId, String),
    PlayerLeave(PlayerId),
    PlayerDied(PlayerId, PlayerId),
    
    CreateEntity(EntityId, EntityTypeId, PlayerId),
    RemoveEntity(EntityId),

    PlaySound(String),

    // This event is only sent to specific players, to indicate
    // that this tick contains the server-side state for the player state
    // after some input by the player was processed on the server
    // Not yet used, since we haven't implemented client-side prediction so far
    CorrectState(TickNumber),

    TakeItem(PlayerId, EntityId),
    //UseItem(PlayerId, ItemType),
}

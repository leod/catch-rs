#![plugin(cereal_macros)]
#![feature(custom_derive, plugin)]

#[macro_use] extern crate log;
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
pub mod movement;
pub mod services;

pub use map::Map;
pub use tick::{TickState, Tick};
pub use player::{Item, PlayerInputKey, PlayerInput, PlayerInfo};
pub use entities::{EntityType, EntityTypes};

pub type EntityId = u32;
pub type EntityTypeId = u32;

pub type PlayerId = u32;
pub type PlayerInputNumber = u32;
pub type ItemSlot = u32;

pub const NEUTRAL_PLAYER_ID: PlayerId = 0;
pub const NUM_ITEM_SLOTS: ItemSlot = 3;

pub type TickNumber = u32;

/// Sent to the clients by the server after connecting
#[derive(Debug, Clone, CerealData)]
pub struct GameInfo {
    pub map_name: String,
    pub entity_types: EntityTypes,
    pub ticks_per_second: u32,
}

#[derive(Debug, Clone, CerealData)]
pub enum GameEvent {
    PlayerJoin(PlayerId, String),
    PlayerLeave(PlayerId),
    PlayerDied {
        player_id: PlayerId,
        position: math::Vec2, 
        responsible_player_id: PlayerId,
    },
    
    CreateEntity(EntityId, EntityTypeId, PlayerId),
    RemoveEntity(EntityId),

    PlaySound(String),

    // This event is only sent to specific players, to indicate
    // that this tick contains the server-side state for the player state
    // after some input by the player was processed on the server
    // Not yet used, since we haven't implemented client-side prediction so far
    CorrectState(TickNumber),

    // The following events are sent to the clients so that they can do some graphical display
    PlayerDash {
        player_id: PlayerId,
        position: math::Vec2,
        orientation: f32,
    },
    PlayerFlip {
        player_id: PlayerId,
        position: math::Vec2,
        orientation: f32,
        speed: f32,
        orientation_wall: f32,
    },
    PlayerTakeItem {
        player_id: PlayerId,
        position: math::Vec2,
    },
    PlayerEquipItem {
        player_id: PlayerId,
        position: math::Vec2,
        item: Item,
    },
    EnemyDied { // This one might not be necessary
        position: math::Vec2,
    },
    ProjectileImpact {
        position: math::Vec2,
    },

    TakeItem(PlayerId, EntityId),
    //UseItem(PlayerId, ItemType),
}

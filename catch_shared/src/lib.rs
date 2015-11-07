#![feature(concat_idents)]

#[macro_use] extern crate log;
extern crate time;
extern crate rustc_serialize;
extern crate bincode;
#[macro_use] extern crate ecs;
extern crate tiled;
extern crate vecmath as vecmath_lib;
extern crate nalgebra as na;

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
pub mod net_components;

pub use map::Map;
pub use tick::{TickState, Tick};
pub use player::{Item, PlayerInputKey, PlayerInput, PlayerInfo, PlayerStats};
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
#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct GameInfo {
    pub map_name: String,
    pub entity_types: EntityTypes,
    pub ticks_per_second: u32,
}

#[derive(Debug, Clone, Copy, RustcEncodable, RustcDecodable)]
pub enum DeathReason {
    Projectile,
    Caught,
    BouncyBall,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum GameEvent {
    // Player list replication
    InitialPlayerList(Vec<(PlayerId, PlayerInfo)>),
    PlayerJoin(PlayerId, PlayerInfo),
    PlayerLeave(PlayerId),
    UpdatePlayerStats(Vec<(PlayerId, PlayerStats)>),

    PlayerDied {
        player_id: PlayerId,
        position: na::Vec2<f32>, 
        responsible_player_id: PlayerId,
        reason: DeathReason,
    },
    
    // Entity replication
    CreateEntity(EntityId, EntityTypeId, PlayerId),
    RemoveEntity(EntityId),

    // Events for graphical display by the clients
    PlayerDash {
        player_id: PlayerId,
        position: na::Vec2<f32>,
        orientation: f32,
    },
    PlayerFlip {
        player_id: PlayerId,
        position: na::Vec2<f32>,
        orientation: f32,
        speed: f32,
        orientation_wall: f32,
    },
    PlayerTakeItem {
        player_id: PlayerId,
        position: na::Vec2<f32>,
    },
    PlayerEquipItem {
        player_id: PlayerId,
        position: na::Vec2<f32>,
        item: Item,
    },
    EnemyDied {
        position: na::Vec2<f32>,
    },
    ProjectileImpact {
        position: na::Vec2<f32>,
    },
}

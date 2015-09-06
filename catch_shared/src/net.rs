use std::collections::HashMap;
use std::fmt;

use player::{PlayerId, PlayerInput, PlayerInputNumber};

pub type EntityId = u32;
pub type EntityTypeId = u32;
pub type TickNumber = u32;

#[derive(Clone, CerealData)]
pub struct GameInfo {
    pub map_name: String,
    pub entity_types: EntityTypes,
    pub ticks_per_second: u32,
}

#[derive(Debug, Clone)]
pub enum Channel {
    Messages,
    Ticks,
}

pub const NUM_CHANNELS: usize = 2;

#[derive(Debug, Clone, CerealData)]
pub enum ClientMessage {
    Pong,
    WishConnect {
        name: String,
    },
    PlayerInput {
        tick: TickNumber,
        input: PlayerInput,
    },
    StartingTick {
        tick: TickNumber,
    }
}

#[derive(Debug, Clone, CerealData)]
pub enum ServerMessage {
    Ping,
    AcceptConnect {
        your_id: PlayerId,
        game_info: GameInfo,
    },

    // Broadcast messages
    PlayerConnect {
        id: PlayerId,
        name: String,
    },
    PlayerDisconnect {
        id: PlayerId,
    },
}

// Components whose state can be synchronized over the net.
// Currently, adding a new ComponentType means you'll need to modify a bunch
// of methods/data types that are kind of all over the place:
// - shared::tick::Tick::{read, write},
// - shared::tick::NetState,
// - client::systems::NetEntitySystem::load_tick_state,
// - client::systems::NetEntitySystem::load_interp_tick_state,
// - server::systems::NetEntitySystem::process
//
// This isn't so pretty, but all the 'dynamic' solutions I could think of weren't either.
// Maybe I need some kind of crazy macro.
#[derive(Clone, CerealData)]
pub enum ComponentType {
    Position,
    Orientation,
    PlayerState
}

pub const COMPONENT_TYPES: &'static [ComponentType] = &[
    ComponentType::Position,
    ComponentType::Orientation,
    ComponentType::PlayerState,
];

#[derive(CerealData, Clone)]
pub struct EntityType {
    pub component_types: Vec<ComponentType>,
}

/// Every entity that wants its component state synchronized needs to have this component
pub struct NetEntity {
    pub id: EntityId,
    pub type_id: EntityTypeId,
    pub owner: PlayerId,
}

pub type EntityTypes = Vec<(String, EntityType)>;

pub fn all_entity_types() -> EntityTypes {
    let mut entity_types: Vec<(String, EntityType)> = Vec::new();

    entity_types.push(("player".to_string(),
        EntityType {
            component_types: [ComponentType::Position,
                              ComponentType::Orientation,
                              ComponentType::PlayerState].to_vec()
        }));

    entity_types
}

impl fmt::Debug for GameInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GameInfo {{ map_name: {}, ... }}", self.map_name)
    }
}


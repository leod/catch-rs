use std::collections::HashMap;
use std::fmt;

use ecs::{ComponentManager, DataHelper, EntityData, BuildData};

use player::{PlayerId, PlayerInput, PlayerInputNumber};
use tick::NetState;

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
// When adding a new net state component X, we need to do the following:
// * Add a new entry X here in ComponentType and COMPONENT_TYPES,
// * Add a new trait HasX in shared::components,
// * Implement that trait for both client::Components and server::Components,
// * Add an entry for X in tick::NetState,
// * Implement StateComponent<T> for StateComponentImpl<X> in shared::components,
// * Add an entry for StateComponentImpl<X> in shared::component_type_traits,
// * Optionally, make sure X is interpolated on the client side
#[derive(Clone, Copy, CerealData)]
pub enum ComponentType {
    Position,
    Orientation,
    PlayerState
}

pub trait StateComponent<T: ComponentManager> {
    fn add(&self, entity: BuildData<T>, c: &mut T);
    fn write(&self, entity: EntityData<T>, id: EntityId, net_state: &mut NetState, c: &T);
    fn read(&self, entity: EntityData<T>, id: EntityId, net_state: &NetState, c: &mut T);
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


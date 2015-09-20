use std::fmt;

use ecs::{ComponentManager, EntityData, BuildData};

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
pub struct TimedPlayerInput {
    pub duration_s: f64,
    pub input: PlayerInput,
}

#[derive(Debug, Clone, CerealData)]
pub enum ClientMessage {
    Pong,
    WishConnect {
        name: String,
    },
    PlayerInput(TimedPlayerInput),
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
// * Implement StateComponent<T> for StateComponentImpl<X> in shared::components (via macro),
// * Add an entry for StateComponentImpl<X> in shared::component_type_traits,
// * Optionally, make sure X is interpolated on the client side
#[derive(Debug, Clone, Copy, PartialEq, Eq, CerealData)]
pub enum ComponentType {
    Position,
    Orientation,
    LinearVelocity,
    Shape,
    PlayerState,
    FullPlayerState,
}

pub trait StateComponent<T: ComponentManager> {
    // Add net component to the component manager for the given entity
    fn add(&self, entity: BuildData<T>, c: &mut T);

    // Stores current component state in a NetState
    fn store(&self, entity: EntityData<T>, id: EntityId, write: &mut NetState, c: &T);

    // Load component state from NetState
    fn load(&self, entity: EntityData<T>, id: EntityId, net_state: &NetState, c: &mut T);
}

pub const COMPONENT_TYPES: &'static [ComponentType] = &[
    ComponentType::Position,
    ComponentType::Orientation,
    ComponentType::LinearVelocity,
    ComponentType::Shape,
    ComponentType::PlayerState,
    ComponentType::FullPlayerState,
];

#[derive(Clone, CerealData)]
pub struct EntityType {
    // Net components of this entity type, i.e. components whose states are sent to clients in
    // ticks
    pub component_types: Vec<ComponentType>,

    // Components that should be sent only to the owner of the object
    // Example: the full state of a player including cooldowns etc. is only needed by the owner
    pub owner_component_types: Vec<ComponentType>,
}

pub type EntityTypes = Vec<(String, EntityType)>;

pub fn all_entity_types() -> EntityTypes {
    vec![("player".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation,
                                    ComponentType::LinearVelocity,
                                    ComponentType::PlayerState],
              owner_component_types: vec![ComponentType::FullPlayerState],
         }),
         ("bouncy_enemy".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation],
              owner_component_types: vec![],
         }),
         ("item".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation],
              owner_component_types: vec![],
         }),
        ]
}

impl fmt::Debug for GameInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GameInfo {{ map_name: {}, ... }}", self.map_name)
    }
}


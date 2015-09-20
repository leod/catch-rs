use std::fmt;

use super::{PlayerInput, TickNumber, PlayerId, GameInfo, EntityType, EntityTypes};

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

pub const COMPONENT_TYPES: &'static [ComponentType] = &[
    ComponentType::Position,
    ComponentType::Orientation,
    ComponentType::LinearVelocity,
    ComponentType::Shape,
    ComponentType::PlayerState,
    ComponentType::FullPlayerState,
];


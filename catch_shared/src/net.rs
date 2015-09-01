use std::collections::HashMap;

use player::{PlayerId, PlayerInput};

pub type EntityId = u32;
pub type EntityTypeId = u32;
pub type TickNumber = u32;

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
        input: PlayerInput
    }
}

#[derive(Debug, Clone, CerealData)]
pub enum ServerMessage {
    Ping,
    AcceptConnect {
        your_id: PlayerId
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

// Components whose state can be synchronized over the net
#[derive(Clone, CerealData)]
pub enum ComponentType {
    Position,
}

pub const COMPONENT_TYPES: &'static [ComponentType] = &[ComponentType::Position];

#[derive(CerealData, Clone)]
pub struct EntityType {
    pub component_types: Vec<ComponentType>,
}

/// Every entity that wants its component state synchronized needs to have this component
pub struct NetEntity {
    pub id: EntityId,
    pub type_id: EntityTypeId,
}

pub type EntityTypes = Vec<(String, EntityType)>;

pub fn all_entity_types() -> EntityTypes {
    let mut entity_types: Vec<(String, EntityType)> = Vec::new();

    entity_types.push(("player".to_string(),
        EntityType {
            component_types: [ComponentType::Position].to_vec()
        }));

    entity_types
}

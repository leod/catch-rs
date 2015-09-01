use std::collections::HashMap;

use player::{PlayerId, PlayerInput};

pub type EntityId = u32;
pub type EntityTypeId = u32;
pub type TickNumber = u32;

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

#[derive(Clone, CerealData)]
pub struct EntityType {
    pub component_types: Vec<ComponentType>,
}

/// Every entity that wants its component state synchronized needs to have this component
pub struct NetEntity {
    pub id: EntityId,
    pub type_id: EntityTypeId,
}

pub type EntityTypes = HashMap<EntityTypeId, EntityType>;
pub type EntityTypeNames = HashMap<String, EntityTypeId>;

pub fn entity_types_by_name() -> Vec<(String, EntityType)> {
    let mut entity_types: Vec<(String, EntityType)> = Vec::new();

    entity_types.push(("player".to_string(),
        EntityType {
            component_types: [ComponentType::Position].to_vec()
        }));

    entity_types
}

pub fn create_entity_type_maps(types_by_name: &Vec<(String, EntityType)>) -> (EntityTypes, EntityTypeNames) {
    let mut id: EntityTypeId = 0;
    let mut types: EntityTypes = HashMap::new();
    let mut type_names: EntityTypeNames = HashMap::new();

    for &(ref type_name, ref entity_type) in types_by_name {
        assert!(type_names.get(type_name).is_none(),
                "Duplicate net entity type name");

        types.insert(id, entity_type.clone());
        type_names.insert(type_name.clone(), id);
        
        id += 1;
    }

    (types, type_names)
}


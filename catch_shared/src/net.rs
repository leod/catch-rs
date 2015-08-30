use std::collections::HashMap;
use std::io::{Read, Write};

use cereal::{CerealData, CerealError, CerealResult};
use ecs::{Aspect, EntityData, DataHelper, ComponentManager};

pub type NetEntityId = i32;
pub type NetEntityTypeId = i32;
pub type PlayerId = i32;

enum NetComponentType {
    Position,
}

const NET_COMPONENT_TYPES: &[NetComponentType] = [NetComponentType::Position];

struct NetEntityType {
    pub id: NetEntityTypeId,
    pub net_component_types: Vec<NetComponentType>,
}

type NetEntityTypes = HashMap<NetEntityTypeId, NetEntityType>;

/// Every entity that wants its component state synchronized needs to have this component
pub struct NetEntityComponent {
    pub id: NetEntityId,
    pub type_id: NetEntityTypeId,
}

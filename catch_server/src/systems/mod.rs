mod net_entity_system;

use ecs::system::{LazySystem, EntitySystem};

pub use self::net_entity_system::NetEntitySystem;
use super::components::{Components};
use super::services::Services;

systems! {
    struct Systems<Components, Services> {
        net_entity_system: EntitySystem<NetEntitySystem> = EntitySystem::new(NetEntitySystem::new(),
            aspect!(<Components> all: [net_entity])),
    }
}

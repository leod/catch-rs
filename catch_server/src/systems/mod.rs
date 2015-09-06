mod net_entity_system;
mod player_movement_system;

use ecs::system::{LazySystem, EntitySystem};

pub use self::net_entity_system::NetEntitySystem;
pub use self::player_movement_system::PlayerMovementSystem;

use super::components::{Components};
use super::services::Services;

systems! {
    struct Systems<Components, Services> {
        net_entity_system: EntitySystem<NetEntitySystem> = EntitySystem::new(NetEntitySystem::new(),
            aspect!(<Components> all: [net_entity])),
        player_movement_system: PlayerMovementSystem = PlayerMovementSystem::new(),
    }
}

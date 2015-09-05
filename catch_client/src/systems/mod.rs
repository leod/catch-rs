mod net_entity_system;
mod draw_player_system;

use ecs::system::{LazySystem, EntitySystem};

pub use self::net_entity_system::NetEntitySystem;
pub use self::draw_player_system::DrawPlayerSystem;
use super::components::*;
use super::services::Services;

systems! {
    struct Systems<Components, Services> {
        net_entity_system: LazySystem<NetEntitySystem> = LazySystem::new(),
        draw_player_system: DrawPlayerSystem = DrawPlayerSystem::new(
            aspect!(<Components> all: [position, orientation, player_state])),
    }
}

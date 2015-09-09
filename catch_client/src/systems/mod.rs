mod net_entity_system;
mod interpolation_system;
mod draw_player_system;

use ecs::system::LazySystem;

pub use self::net_entity_system::NetEntitySystem;
pub use self::interpolation_system::InterpolationSystem;
pub use self::draw_player_system::DrawPlayerSystem;
use super::components::*;
use super::services::Services;

systems! {
    struct Systems<Components, Services> {
        net_entity_system: LazySystem<NetEntitySystem> = LazySystem::new(),
        interpolation_system: InterpolationSystem = InterpolationSystem::new(
            aspect!(<Components> all: [position, interp_position]),
            aspect!(<Components> all: [orientation, interp_orientation])),
        draw_player_system: DrawPlayerSystem = DrawPlayerSystem::new(
            aspect!(<Components> all: [position, orientation, player_state])),
    }
}

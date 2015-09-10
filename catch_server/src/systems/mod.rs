mod net_entity_system;
mod player_movement_system;
mod bouncy_enemy_system;

use ecs::system::EntitySystem;

pub use self::net_entity_system::NetEntitySystem;
pub use self::player_movement_system::PlayerMovementSystem;
pub use self::bouncy_enemy_system::BouncyEnemySystem;

use super::components::{Components};
use super::services::Services;

systems! {
    struct Systems<Components, Services> {
        net_entity_system: EntitySystem<NetEntitySystem> = EntitySystem::new(NetEntitySystem::new(),
            aspect!(<Components> all: [net_entity])),
        player_movement_system: PlayerMovementSystem = PlayerMovementSystem::new(),
        bouncy_enemy_system: BouncyEnemySystem = BouncyEnemySystem::new(
            aspect!(<Components> all: [server_net_entity,
                                       position,
                                       orientation,
                                       linear_velocity,
                                       bouncy_enemy])),
    }
}

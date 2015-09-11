mod net_entity_system;
mod player_movement_system;
mod bouncy_enemy_system;
mod interaction_system;

use ecs::system::EntitySystem;

pub use self::net_entity_system::NetEntitySystem;
pub use self::player_movement_system::PlayerMovementSystem;
pub use self::bouncy_enemy_system::BouncyEnemySystem;
pub use self::interaction_system::InteractionSystem;
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
        interaction_system: InteractionSystem = InteractionSystem::new(
            aspect!(<Components> all: [position, shape, interact]),
            vec![(aspect!(<Components> all: [player_state, position]),
                  aspect!(<Components> all: [bouncy_enemy, position, orientation]),
                  interaction_system::PLAYER_BOUNCY_ENEMY_INTERACTION),
                 (aspect!(<Components> all: [bouncy_enemy, position, orientation, linear_velocity]),
                  aspect!(<Components> all: [bouncy_enemy, position, orientation, linear_velocity]),
                  interaction_system::BOUNCY_ENEMY_BOUNCY_ENEMY_INTERACTION),
                ])
    }
}

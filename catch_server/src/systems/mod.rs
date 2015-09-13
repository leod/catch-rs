mod net_entity_system;
mod player_movement_system;
mod bouncy_enemy_system;
mod interaction_system;

pub use self::net_entity_system::NetEntitySystem;
pub use self::player_movement_system::PlayerMovementSystem;
pub use self::bouncy_enemy_system::BouncyEnemySystem;
pub use self::interaction_system::InteractionSystem;
use super::components::{Components};
use super::services::Services;

systems! {
    struct Systems<Components, Services> {
        net_entity_system: NetEntitySystem = NetEntitySystem::new(),
        player_movement_system: PlayerMovementSystem = PlayerMovementSystem::new(),
        bouncy_enemy_system: BouncyEnemySystem = BouncyEnemySystem::new(
            aspect!(<Components> all: [bouncy_enemy])),
        interaction_system: InteractionSystem = InteractionSystem::new(
            aspect!(<Components> all: [position, shape, interact]),
            vec![(aspect!(<Components> all: [player_state]),
                  aspect!(<Components> all: [bouncy_enemy]),
                  interaction_system::PLAYER_BOUNCY_ENEMY_INTERACTION),
                 (aspect!(<Components> all: [bouncy_enemy]),
                  aspect!(<Components> all: [bouncy_enemy]),
                  interaction_system::BOUNCY_ENEMY_BOUNCY_ENEMY_INTERACTION),
                ])
    }
}

pub mod net_entity_system;
pub mod player_controller_system;
pub mod player_item_system;
pub mod bouncy_enemy_system;
pub mod item_spawn_system;
pub mod rotate_system;
pub mod projectile_system;
pub mod movement_system;
pub mod wall_interactions;
pub mod interaction_system;
pub mod interactions;

use super::components::{Components};
use super::services::Services;
pub use self::net_entity_system::NetEntitySystem;
pub use self::player_controller_system::PlayerControllerSystem;
pub use self::player_item_system::PlayerItemSystem;
pub use self::bouncy_enemy_system::BouncyEnemySystem;
pub use self::item_spawn_system::ItemSpawnSystem;
pub use self::rotate_system::RotateSystem;
pub use self::projectile_system::ProjectileSystem;
pub use self::movement_system::MovementSystem;
pub use self::interaction_system::InteractionSystem;

systems! {
    struct Systems<Components, Services> {
        net_entity_system: NetEntitySystem = NetEntitySystem::new(
            aspect!(<Components> all: [net_entity])),
        player_controller_system: PlayerControllerSystem = PlayerControllerSystem::new(
            aspect!(<Components> all: [player_controller]),
            aspect!(<Components> all: [wall])),
        player_item_system: PlayerItemSystem = PlayerItemSystem::new(),
        bouncy_enemy_system: BouncyEnemySystem = BouncyEnemySystem::new(
            aspect!(<Components> all: [bouncy_enemy])),
        item_spawn_system: ItemSpawnSystem = ItemSpawnSystem::new(
            aspect!(<Components> all: [item_spawn])),
        rotate_system: RotateSystem = RotateSystem::new(
            aspect!(<Components> all: [rotate])),
        projectile_system: ProjectileSystem = ProjectileSystem::new(
            aspect!(<Components> all: [projectile])),
        movement_system: MovementSystem = MovementSystem::new(
            aspect!(<Components> all: [wall]),
            vec![
                 (aspect!(<Components> all: [bouncy_enemy]),
                  Box::new(wall_interactions::BouncyEnemyWallInteraction)),
                 (aspect!(<Components> all: [projectile]),
                  Box::new(wall_interactions::ProjectileWallInteraction)),
                ]),
        interaction_system: InteractionSystem = InteractionSystem::new(
            vec![
                 (aspect!(<Components> all: [player_state]),
                  aspect!(<Components> all: [bouncy_enemy]),
                  Box::new(interactions::PlayerBouncyEnemyInteraction)),
                 (aspect!(<Components> all: [player_state]),
                  aspect!(<Components> all: [item]),
                  Box::new(interactions::PlayerItemInteraction)),
                 (aspect!(<Components> all: [projectile]),
                  aspect!(<Components> all: [bouncy_enemy]),
                  Box::new(interactions::ProjectileBouncyEnemyInteraction)),
                 (aspect!(<Components> all: [projectile]),
                  aspect!(<Components> all: [player_state]),
                  Box::new(interactions::ProjectilePlayerInteraction)),
                ],
            vec![
                 (aspect!(<Components> all: [bouncy_enemy]),
                  Box::new(interactions::BouncyEnemyInteraction)),
                 (aspect!(<Components> all: [player_state]),
                  Box::new(interactions::PlayerPlayerInteraction)),
                ]),
    }
}

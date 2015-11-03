pub mod net_entity_system;
pub mod interpolation_system;
pub mod draw_player_system;
pub mod draw_bouncy_enemy_system;
pub mod draw_item_system;
pub mod draw_projectile_system;
pub mod draw_wall_system;

use ecs::system::LazySystem;

use super::components::*;
use super::services::Services;
pub use self::net_entity_system::NetEntitySystem;
pub use self::interpolation_system::InterpolationSystem;
pub use self::draw_player_system::DrawPlayerSystem;
pub use self::draw_bouncy_enemy_system::DrawBouncyEnemySystem;
pub use self::draw_item_system::DrawItemSystem;
pub use self::draw_projectile_system::DrawProjectileSystem;
pub use self::draw_wall_system::DrawWallSystem;

systems! {
    struct Systems<Components, Services> {
        net_entity_system: LazySystem<NetEntitySystem> = LazySystem::new(),
        interpolation_system: InterpolationSystem = InterpolationSystem::new(
            aspect!(<Components> all: [position, interp_position]),
            aspect!(<Components> all: [orientation, interp_orientation])),
        draw_player_system: DrawPlayerSystem = DrawPlayerSystem::new(
            aspect!(<Components> all: [draw_player])),
        draw_bouncy_enemy_system: DrawBouncyEnemySystem = DrawBouncyEnemySystem::new(
            aspect!(<Components> all: [draw_bouncy_enemy])),
        draw_item_system: DrawItemSystem = DrawItemSystem::new(
            aspect!(<Components> all: [draw_item])),
        draw_projectile_system: DrawProjectileSystem = DrawProjectileSystem::new(
            aspect!(<Components> all: [draw_projectile])),
        draw_wall_system: DrawWallSystem = DrawWallSystem::new(
            aspect!(<Components> all: [draw_wall])),
    }
}

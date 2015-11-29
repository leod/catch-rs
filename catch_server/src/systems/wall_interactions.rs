use ecs::{EntityData, DataHelper};
use na::Vec2;

use shared::movement::{WallInteractionType, WallInteraction};

use components::Components;
use services::Services;
use entities;
use systems::projectile_system;

pub struct ConstWallInteraction(pub WallInteractionType);
impl WallInteraction<Components, Services> for ConstWallInteraction {
    fn apply(&self, _p: Vec2<f32>,
             _enemy: EntityData<Components>, _wall: EntityData<Components>,
             _data: &mut DataHelper<Components, Services>)
             -> WallInteractionType {
        self.0
    }
}

/// Bouncy enemy interaction with wall
pub struct BouncyEnemyWallInteraction;
impl WallInteraction<Components, Services> for BouncyEnemyWallInteraction {
    fn apply(&self, _p: Vec2<f32>,
             _enemy: EntityData<Components>, _wall: EntityData<Components>,
             _data: &mut DataHelper<Components, Services>)
             -> WallInteractionType {
        WallInteractionType::Flip
    }
}

/// Projectile interaction with wall
pub struct ProjectileWallInteraction;
impl WallInteraction<Components, Services> for ProjectileWallInteraction {
    fn apply(&self, p: Vec2<f32>,
             projectile: EntityData<Components>, _wall: EntityData<Components>,
             data: &mut DataHelper<Components, Services>)
             -> WallInteractionType {
        projectile_system::explode(projectile, data);
        WallInteractionType::Stop
    }
}

use ecs::{EntityData, DataHelper};
use na::Vec2;

use shared::GameEvent;
use shared::services::HasEvents;
use shared::movement::{WallInteractionType, WallInteraction};

use components::Components;
use services::Services;
use entities;

/// Bouncy enemy interaction with wall
pub struct BouncyEnemyWallInteraction;
impl WallInteraction<Components, Services> for BouncyEnemyWallInteraction {
    fn apply(&self, p: Vec2<f32>,
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
        let event = &GameEvent::ProjectileImpact {
            position: p,
        };
        data.services.add_event(&event);
        entities::remove_net(**projectile, data);
        WallInteractionType::Stop
    }
}

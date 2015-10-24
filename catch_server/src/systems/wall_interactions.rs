use ecs::{EntityData, DataHelper};

use shared::{GameEvent};
use shared::math;
use shared::movement::{WallInteractionType, WallInteraction};
use shared::services::HasEvents;

use components::Components;
use services::Services;

/// Bouncy enemy interaction with wall
pub struct BouncyEnemyWallInteraction;
impl WallInteraction<Components, Services> for BouncyEnemyWallInteraction {
    fn apply(&self,
             enemy: EntityData<Components>, wall: EntityData<Components>,
             data: &mut DataHelper<Components, Services>)
             -> WallInteractionType {
        WallInteractionType::Flip
    }
}

use std::f64;
use ecs::{EntityData, DataHelper};

use shared::math;
use shared::event::GameEvent;
use shared::player::NEUTRAL_PLAYER_ID;
use components::Components;
use services::Services;
use systems::interaction_system::Interaction;

pub struct PlayerBouncyEnemyInteraction;
pub struct BouncyEnemyInteraction;
pub struct PlayerItemInteraction;

impl Interaction for PlayerBouncyEnemyInteraction {
    fn apply(&self,
             player_e: EntityData<Components>, enemy_e: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        if data.player_state[player_e].vulnerable() {
            // Kill player
            let owner = data.net_entity[player_e].owner;
            data.services.add_event_to_run(&GameEvent::PlayerDied(owner, NEUTRAL_PLAYER_ID));
        }
    }
}

impl Interaction for BouncyEnemyInteraction {
    fn apply(&self,
             a_e: EntityData<Components>, b_e: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        // Flip orientations of both entities and add some velocity in the new direction

        data.orientation[a_e].angle = data.orientation[a_e].angle + f64::consts::PI;
        let direction_a = [data.orientation[a_e].angle.cos(),
                           data.orientation[a_e].angle.sin()];
        data.linear_velocity[a_e].v = math::add(data.linear_velocity[a_e].v,
                                                math::scale(direction_a, 500.0));

        data.orientation[b_e].angle = data.orientation[b_e].angle + f64::consts::PI;
        let direction_b = [data.orientation[b_e].angle.cos(),
                           data.orientation[b_e].angle.sin()];
        data.linear_velocity[b_e].v = math::add(data.linear_velocity[b_e].v,
                                                math::scale(direction_b, 500.0));
    }
}


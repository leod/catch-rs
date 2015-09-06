use ecs;
use ecs::{Process, System, EntityData, DataHelper};
use ecs::system::EntityProcess;

use shared::math;
use shared::player::{PlayerInput};
use shared::event::GameEvent;
use components::*;
use services::Services;

pub struct PlayerMovementSystem;

impl PlayerMovementSystem {
    pub fn new() -> PlayerMovementSystem {
        PlayerMovementSystem
    }

    pub fn run_player_input(&self,
                            entity: ecs::Entity,
                            input: &PlayerInput,
                            data: &mut DataHelper<Components, Services>) {
        data.with_entity_data(&entity, |e, c| {
            /*if input.left_pressed {
                c.position[e].o
            }*/

            // TODO: This is just for testing
            if input.forward_pressed {
                c.position[e].p = math::add(c.position[e].p, [10.0, 0.0]);
            }
        });
    }
}

impl System for PlayerMovementSystem {
    type Components = Components;
    type Services = Services;
}

impl Process for PlayerMovementSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

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
        // TODO: This is just for testing
        const TURN_SPEED: f64 = 0.15;
        const MOVE_SPEED: f64 = 5.0;

        data.with_entity_data(&entity, |e, c| {
            if input.left_pressed {
                c.orientation[e].angle -= TURN_SPEED;
            }
            if input.right_pressed {
                c.orientation[e].angle += TURN_SPEED;
            }
            
            let velocity = [
                c.orientation[e].angle.cos() * MOVE_SPEED,
                c.orientation[e].angle.sin() * MOVE_SPEED
            ];
            
            if input.forward_pressed {
                c.position[e].p = math::add(c.position[e].p, velocity);
            }
            if input.back_pressed {
                c.position[e].p = math::sub(c.position[e].p, velocity);
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

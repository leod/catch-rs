use ecs;
use ecs::{Process, System, EntityData, DataHelper};
use ecs::system::EntityProcess;

use shared::math;
use shared::map::Map;
use shared::player::{PlayerInput};
use shared::event::GameEvent;
use components::*;
use services::Services;

pub struct PlayerMovementSystem;

impl PlayerMovementSystem {
    pub fn new() -> PlayerMovementSystem {
        PlayerMovementSystem
    }

    fn move_straight(&self,
                     e: EntityData<Components>,
                     delta: math::Vec2,
                     map: &Map,
                     data: &mut Components) {
        let p = data.position[e].p;
        let q = math::add(p, delta);

        data.position[e].p = match map.line_segment_intersection(p, q) {
            Some((_, _, n, s)) => // Walk as far as we can
                math::add(p, math::scale(delta, s)),
            None =>
                q
        };
    }

    fn move_sliding(&self,
                    e: EntityData<Components>,
                    delta: math::Vec2,
                    map: &Map,
                    data: &mut Components) {
        let p = data.position[e].p;
        let q = math::add(p, delta);

        match map.line_segment_intersection(p, q) {
            Some((tx, ty, n, s)) => {
                // We walked into a surface with normal n.
                // Find parts of delta parallel and orthogonal to n
                let u = math::scale(n, math::dot(delta, n));
                let v = math::sub(delta, u);

                self.move_straight(e, u, map, data);
                self.move_straight(e, v, map, data);
            }
            None => {
                data.position[e].p = q;
            }
        };
    }

    pub fn run_player_input(&self,
                            entity: ecs::Entity,
                            input: &PlayerInput,
                            map: &Map,
                            data: &mut DataHelper<Components, Services>) {
        // TODO: This is just for testing
        const TURN_SPEED: f64 = 0.3;
        const MOVE_SPEED: f64 = 10.0;

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
                self.move_sliding(e, velocity, map, c);

            }
            if input.back_pressed {
                self.move_sliding(e, math::neg(velocity), map, c);
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

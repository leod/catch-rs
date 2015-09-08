use ecs;
use ecs::{Process, System, EntityData, DataHelper};

use shared::math;
use shared::map::Map;
use shared::player::{PlayerInput};
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
            Some((_, _, _, s)) => { // Walk as far as we can
                let s = (s - 0.00001).max(0.0);
                math::add(p, math::scale(delta, s))
            }
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
            Some((_, _, n, _)) => {
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
        const MOVE_ACCEL: f64 = 6.0;
        const BACK_ACCEL: f64 = 3.0;
        const MOVE_SPEED: f64 = 10.0;
        const MIN_SPEED: f64 = 0.001;

        data.with_entity_data(&entity, |e, c| {
            if input.left_pressed {
                c.orientation[e].angle -= TURN_SPEED;
            }
            if input.right_pressed {
                c.orientation[e].angle += TURN_SPEED;
            }

            let velocity = c.linear_velocity[e].v;
            let angle = c.orientation[e].angle;

            let mut accel = math::scale(velocity, -0.4);

            if input.forward_pressed {
                accel = math::add([angle.cos() * MOVE_ACCEL,
                                   angle.sin() * MOVE_ACCEL],
                                  accel);
            }
            if input.back_pressed {
                accel = math::add([-angle.cos() * BACK_ACCEL,
                                   -angle.sin() * BACK_ACCEL],
                                  accel);
            }

            c.linear_velocity[e].v = math::add(c.linear_velocity[e].v, accel);

            if c.linear_velocity[e].v[0].abs() <= MIN_SPEED {
                c.linear_velocity[e].v[0] = 0.0;
            }
            if c.linear_velocity[e].v[1].abs() <= MIN_SPEED {
                c.linear_velocity[e].v[1] = 0.0;
            }

            self.move_sliding(e, c.linear_velocity[e].v, map, c);
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

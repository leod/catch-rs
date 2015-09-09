use std::f64;

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
                if data.player_state[e].dashing.is_some() && data.player_state[e].dashing.unwrap() < 0.9 {
                    data.player_state[e].dashing = Some(0.9);
                }

                let s = (s - 0.0001).max(0.0);
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

    pub fn move_flicking(&self,
                         e: EntityData<Components>,
                         delta: math::Vec2,
                         map: &Map,
                         data: &mut Components) {
        let p = data.position[e].p;
        let q = math::add(p, delta);

        match map.line_segment_intersection(p, q) {
            Some((_, _, n, s)) => {
                let n_angle = n[1].atan2(n[0]);
                let angle = data.orientation[e].angle;

                data.orientation[e].angle = f64::consts::PI + n_angle - (angle - n_angle);

                let v = data.linear_velocity[e].v;
                let speed = math::square_len(v).sqrt();
                data.linear_velocity[e].v = [
                    data.orientation[e].angle.cos() * (speed + 1.0),
                    data.orientation[e].angle.sin() * (speed + 1.0),
                ];
                    //math::sub(math::scale(n, 2.0 * math::dot(v, n)), v);

                let s = (s - 0.0001).max(0.0);
                data.position[e].p = math::add(p, math::scale(delta, s));
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
        const TURN_SPEED: f64 = 2.0*f64::consts::PI;
        const MOVE_ACCEL: f64 = 500.0;
        const BACK_ACCEL: f64 = 200.0;
        const MIN_SPEED: f64 = 0.001;
        const DASH_SPEED: f64 = 600.0;
        const DASH_DURATION_S: f64 = 0.3;

        let tick_dur_s = data.services.tick_dur_s;

        data.with_entity_data(&entity, |e, c| {
            let angle = c.orientation[e].angle;
            let direction = [angle.cos(), angle.sin()];

            if let Some(dashing) = c.player_state[e].dashing {
                //c.linear_velocity[e].v 
                //let target = math::scale(direction, DASH_SPEED);
                /*c.linear_velocity[e].v = math::add(c.linear_velocity[e].v,
                                                   math::scale(math::sub(target, c.linear_velocity[e].v), 0.4));*/

                //let scale = ((4.0 - 8.0 * dashing).atan() + f64::consts::PI / 2.0) / f64::consts::PI;
                //let scale = ((-4.0 + dashing * 8.0).atan() + f64::consts::PI / 2.0) / f64::consts::PI;
                let t = dashing / DASH_DURATION_S;
                let scale = (t*f64::consts::PI/2.0).cos()*(1.0-(1.0-t).powi(10));
                c.linear_velocity[e].v = math::scale(direction, DASH_SPEED);

                c.player_state[e].dashing = if dashing + tick_dur_s <= DASH_DURATION_S {
                    Some(dashing + tick_dur_s)
                } else {
                    None
                }
            } else {
                if input.left_pressed {
                    c.orientation[e].angle -= TURN_SPEED * tick_dur_s;
                }
                if input.right_pressed {
                    c.orientation[e].angle += TURN_SPEED * tick_dur_s;
                }

                let velocity = c.linear_velocity[e].v;

                let mut accel = math::scale(velocity, -4.0);

                if input.forward_pressed {
                    accel = math::add(math::scale(direction, MOVE_ACCEL), accel);
                }
                if input.back_pressed {
                    accel = math::add(math::scale(direction, -BACK_ACCEL), accel);
                }

                c.linear_velocity[e].v = math::add(c.linear_velocity[e].v, math::scale(accel, tick_dur_s));

                if c.linear_velocity[e].v[0].abs() <= MIN_SPEED {
                    c.linear_velocity[e].v[0] = 0.0;
                }
                if c.linear_velocity[e].v[1].abs() <= MIN_SPEED {
                    c.linear_velocity[e].v[1] = 0.0;
                }

                if input.dash_pressed {
                    c.player_state[e].dashing = Some(0.0);
                }
            }

            if !input.flick_pressed {
                self.move_sliding(e, math::scale(c.linear_velocity[e].v, tick_dur_s), map, c);
            } else {
                self.move_flicking(e, math::scale(c.linear_velocity[e].v, tick_dur_s), map, c);
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

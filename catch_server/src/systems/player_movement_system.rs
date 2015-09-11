use std::f64;

use ecs;
use ecs::{Process, System, EntityData, DataHelper};

use shared::math;
use shared::map::Map;
use shared::net::{ComponentType, TimedPlayerInput};
use shared::player::{PlayerInput, InputKey};
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
                if data.player_state[e].dashing.is_some() &&
                   data.player_state[e].dashing.unwrap() < 0.9 {
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

    pub fn move_flipping(&self,
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
                data.server_net_entity[e].force(ComponentType::Orientation);

                let v = data.linear_velocity[e].v;
                let speed = math::square_len(v).sqrt();
                data.linear_velocity[e].v = [
                    data.orientation[e].angle.cos() * (speed + 1.0),
                    data.orientation[e].angle.sin() * (speed + 1.0),
                ];

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
                            timed_input: &TimedPlayerInput,
                            map: &Map,
                            data: &mut DataHelper<Components, Services>) {
        const TURN_SPEED: f64 = 2.0*f64::consts::PI;
        const MOVE_ACCEL: f64 = 500.0;
        const BACK_ACCEL: f64 = 200.0;
        const STRAFE_ACCEL: f64 = 300.0;
        const MIN_SPEED: f64 = 0.001;
        const DASH_SPEED: f64 = 600.0;
        const DASH_DURATION_S: f64 = 0.3;

        let dur_s = timed_input.duration_s;
        let input = &timed_input.input;

        data.with_entity_data(&entity, |e, c| {
            let angle = c.orientation[e].angle;
            let direction = [angle.cos(), angle.sin()];

            if let Some(inv_s) = c.player_state[e].invulnerable_s {
                let inv_s = inv_s - dur_s;
                c.player_state[e].invulnerable_s =
                    if inv_s <= 0.0 { None }
                    else { Some(inv_s) };
            }

            if let Some(dashing) = c.player_state[e].dashing {
                let t = dashing / DASH_DURATION_S;
                let scale = (t*f64::consts::PI/2.0).cos()*(1.0-(1.0-t).powi(10));
                c.linear_velocity[e].v = math::scale(direction, DASH_SPEED);

                c.player_state[e].dashing =
                    if dashing + dur_s <= DASH_DURATION_S {
                        Some(dashing + dur_s)
                    } else {
                        None
                    };
            } else {
                let mut accel = math::scale(c.linear_velocity[e].v, -4.0);

                if input.has(InputKey::Strafe) {
                    let strafe_direction = [direction[1], -direction[0]];
                    if input.has(InputKey::Left) {
                        accel = math::add(math::scale(strafe_direction, STRAFE_ACCEL), accel);
                    }
                    if input.has(InputKey::Right) {
                        accel = math::add(math::scale(strafe_direction, -STRAFE_ACCEL), accel);
                    }
                } else {
                    if input.has(InputKey::Left) {
                        c.orientation[e].angle -= TURN_SPEED * dur_s;
                    }
                    if input.has(InputKey::Right) {
                        c.orientation[e].angle += TURN_SPEED * dur_s;
                    }
                }
                if input.has(InputKey::Forward) {
                    accel = math::add(math::scale(direction, MOVE_ACCEL), accel);
                }
                if input.has(InputKey::Back) {
                    accel = math::add(math::scale(direction, -BACK_ACCEL), accel);
                }

                c.linear_velocity[e].v = math::add(c.linear_velocity[e].v, math::scale(accel, dur_s));

                if c.linear_velocity[e].v[0].abs() <= MIN_SPEED {
                    c.linear_velocity[e].v[0] = 0.0;
                }
                if c.linear_velocity[e].v[1].abs() <= MIN_SPEED {
                    c.linear_velocity[e].v[1] = 0.0;
                }

                if input.has(InputKey::Dash) {
                    c.player_state[e].dashing = Some(0.0);
                }
            }

            if !input.has(InputKey::Flip) {
                self.move_sliding(e, math::scale(c.linear_velocity[e].v, dur_s), map, c);
            } else {
                self.move_flipping(e, math::scale(c.linear_velocity[e].v, dur_s), map, c);
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

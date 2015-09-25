use std::f64;

use ecs;
use ecs::{Process, System, EntityData, DataHelper};

use shared::math;
use shared::{Map, GameEvent};
use shared::net::{ComponentType, TimedPlayerInput};
use shared::player::PlayerInputKey;

use components::Components;
use services::Services;

const TURN_ACCEL: f64 = 1.5;
const TURN_FRICTION: f64 = 0.25;
const MOVE_ACCEL: f64 = 1000.0;
const MOVE_FRICTION: f64 = 10.0;
const BACK_ACCEL: f64 = 500.0;
const STRAFE_ACCEL: f64 = 900.0;
const MIN_SPEED: f64 = 0.1;
const DASH_SPEED: f64 = 600.0;
const DASH_DURATION_S: f64 = 0.3;

/// System for interpreting player input.
/// When we implement client-side prediction, this module will have to move to catch_shared in
/// some way.
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
            Some(intersection) => { // Walk as far as we can
                if data.player_state[e].dashing.is_some() &&
                   data.player_state[e].dashing.unwrap() < 0.9 {
                    data.player_state[e].dashing = Some(0.9);
                }

                let s = (intersection.t - 0.0001).max(0.0);
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
                    data: &mut Components) -> Vec<GameEvent> {
        let p = data.position[e].p;
        let q = math::add(p, delta);

        match map.line_segment_intersection(p, q) {
            Some(intersection) => {
                // We walked into a surface with normal n.
                // Find parts of delta parallel and orthogonal to n
                let u = math::scale(intersection.n, math::dot(delta, intersection.n));
                let v = math::sub(delta, u);

                self.move_straight(e, u, map, data);
                self.move_straight(e, v, map, data);
            }
            None => {
                data.position[e].p = q;
            }
        };

        Vec::new()
    }

    pub fn move_flipping(&self,
                         e: EntityData<Components>,
                         delta: math::Vec2,
                         map: &Map,
                         data: &mut Components) -> Vec<GameEvent> {
        let p = data.position[e].p;
        let q = math::add(p, delta);

        match map.line_segment_intersection(p, q) {
            Some(intersection) => {
                let n_angle = intersection.n[1].atan2(intersection.n[0]);
                let angle = data.orientation[e].angle;

                data.orientation[e].angle = f64::consts::PI + n_angle - (angle - n_angle);
                data.server_net_entity[e].force(ComponentType::Orientation);

                let v = data.linear_velocity[e].v;
                let speed = math::square_len(v).sqrt();
                data.linear_velocity[e].v = [
                    data.orientation[e].angle.cos() * (speed + 1.0),
                    data.orientation[e].angle.sin() * (speed + 1.0),
                ];

                let s = (intersection.t - 0.0001).max(0.0);
                data.position[e].p = math::add(p, math::scale(delta, s));

                // TODO: Actually at this point we still might have some 't' left to walk

                vec![GameEvent::PlayerFlip {
                         player_id: data.net_entity[e].owner,
                         position: data.position[e].p,
                         orientation: angle,
                         velocity: speed,
                     }]
            }
            None => {
                data.position[e].p = q;

                vec![]
            }
        }
    }

    pub fn run_player_input(&self,
                            entity: ecs::Entity,
                            timed_input: &TimedPlayerInput,
                            map: &Map,
                            data: &mut DataHelper<Components, Services>) {
        let dur_s = timed_input.duration_s;
        let input = &timed_input.input;

        let events = data.with_entity_data(&entity, |e, c| {
            // Cooldowns
            if let Some(dash_cooldown_s) = c.full_player_state[e].dash_cooldown_s {
                let dash_cooldown_s = dash_cooldown_s - dur_s;
                c.full_player_state[e].dash_cooldown_s =
                    if dash_cooldown_s <= 0.0 { None }
                    else { Some(dash_cooldown_s) };
            }
            if let Some(inv_s) = c.player_state[e].invulnerable_s {
                let inv_s = inv_s - dur_s;
                c.player_state[e].invulnerable_s =
                    if inv_s <= 0.0 { None }
                    else { Some(inv_s) };
            }

            let mut events = if !input.has(PlayerInputKey::Flip) {
                self.move_sliding(e, math::scale(c.linear_velocity[e].v, dur_s), map, c)
            } else {
                self.move_flipping(e, math::scale(c.linear_velocity[e].v, dur_s), map, c)
            };

            let angle = c.orientation[e].angle;
            let direction = [angle.cos(), angle.sin()];

            if let Some(dashing) = c.player_state[e].dashing {
                // While dashing, movement input is ignored

                let t = dashing / DASH_DURATION_S;
                let scale = 1.0; //(t*f64::consts::PI/2.0).cos()*(1.0-(1.0-t).powi(10));
                c.linear_velocity[e].v = math::scale(direction, scale*DASH_SPEED);

                c.player_state[e].dashing =
                    if dashing + dur_s <= DASH_DURATION_S {
                        Some(dashing + dur_s)
                    } else {
                        None
                    };
            } else {
                c.orientation[e].angle += c.angular_velocity[e].v;

                let mut accel = math::scale(c.linear_velocity[e].v, -MOVE_FRICTION);

                if input.has(PlayerInputKey::Strafe) {
                    // Strafe left/right
                    c.angular_velocity[e].v = 0.0;

                    let strafe_direction = [direction[1], -direction[0]];
                    if input.has(PlayerInputKey::Left) {
                        accel = math::add(math::scale(strafe_direction, STRAFE_ACCEL), accel);
                    }
                    if input.has(PlayerInputKey::Right) {
                        accel = math::add(math::scale(strafe_direction, -STRAFE_ACCEL), accel);
                    }
                } else {
                    // Turn left/right
                    let mut ang_accel = c.angular_velocity[e].v * -TURN_FRICTION;

                    if input.has(PlayerInputKey::Left) {
                        ang_accel -= TURN_ACCEL * dur_s;
                        //c.orientation[e].angle -= TURN_SPEED * dur_s;
                    }
                    if input.has(PlayerInputKey::Right) {
                        ang_accel += TURN_ACCEL * dur_s;
                        //c.orientation[e].angle += TURN_SPEED * dur_s;
                    }

                    c.angular_velocity[e].v += ang_accel;
                }

                // Move forward/backward
                if input.has(PlayerInputKey::Forward) {
                    accel = math::add(math::scale(direction, MOVE_ACCEL), accel);
                }
                if input.has(PlayerInputKey::Back) {
                    accel = math::add(math::scale(direction, -BACK_ACCEL), accel);
                }

                c.linear_velocity[e].v = math::add(c.linear_velocity[e].v,
                                                   math::scale(accel, dur_s));

                // If velocity is below some limit, set to zero
                if c.linear_velocity[e].v[0].abs() <= MIN_SPEED {
                    c.linear_velocity[e].v[0] = 0.0;
                }
                if c.linear_velocity[e].v[1].abs() <= MIN_SPEED {
                    c.linear_velocity[e].v[1] = 0.0;
                }

                // Start dash if the cooldown is ready
                if input.has(PlayerInputKey::Dash) && 
                   c.full_player_state[e].dash_cooldown_s.is_none() {
                    c.player_state[e].dashing = Some(0.0);
                    c.full_player_state[e].dash_cooldown_s = Some(5.0);
                    c.angular_velocity[e].v = 0.0;

                    events.push(
                        GameEvent::PlayerDash {
                            player_id: c.net_entity[e].owner,
                            position: c.position[e].p,
                            orientation: c.orientation[e].angle,
                        });
                }
            }

            events
        }).unwrap();

        for event in events.iter() {
            data.services.add_event(&event.clone());
        }
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

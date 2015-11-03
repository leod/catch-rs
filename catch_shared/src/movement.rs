use std::f32;

use ecs::{EntityData, DataHelper, ComponentManager, ServiceManager};
use na::{Vec2, Norm, Dot};

use super::{math, GameEvent, PlayerId};
use util::CachedAspect;
use net::TimedPlayerInput;
use player::PlayerInputKey;
use services::HasEvents;
use components::{HasPosition, HasLinearVelocity, HasOrientation, HasAngularVelocity,
                 HasPlayerState, HasFullPlayerState, HasShape, HasWallPosition, WallPosition};

/// What to do when an entity hits a wall while moving
pub enum WallInteractionType {
    /// Slide at wall
    Slide, 

    /// Flip entity orientation at the wall
    Flip,

    /// Just stop moving
    Stop,
}

/// Defines an interaction between an entity and a wall segment
pub trait WallInteraction<Components: ComponentManager,
                          Services: ServiceManager> {
    /// Makes something happen when the entity hits a wall. Returns the action to be taken at the
    /// wall (i.e., flip entity at wall or just slide at the wall)
    fn apply(&self,
             entity: EntityData<Components>, wall: EntityData<Components>,
             data: &mut DataHelper<Components, Services>)
             -> WallInteractionType;
}

pub fn wall_normal(p: &WallPosition) -> Vec2<f32> {
    let d = p.pos_b - p.pos_a;
    Vec2::new(d.y, -d.x).normalize()
}

pub fn wall_orientation(p: &WallPosition) -> f32 {
    let n = wall_normal(p);
    n[1].atan2(n[0])
}

const STEPBACK: f32 = 0.05;

/// Moves an entity while checking for intersections with walls.
/// If there is an intersection, the given `interaction` is called.
pub fn move_entity<Components: ComponentManager,
                   Services: ServiceManager>
                  (e: EntityData<Components>,
                   dur_s: f32,
                   interaction: &WallInteraction<Components, Services>,
                   wall_aspect: &CachedAspect<Components>,
                   c: &mut DataHelper<Components, Services>)
        where Components: HasPosition + HasLinearVelocity + HasShape +
                          HasOrientation + HasWallPosition {
    let delta = c.linear_velocity()[e].v * dur_s;
    let a = c.position()[e].p;
    let b = a + delta;

    c.position_mut()[e].p = match line_segment_walls_intersection(a, b, wall_aspect, c) {
        Some((t, wall)) => {
            // We hit a wall, ask `interaction` what to do
            match interaction.apply(e, wall, c) {
                WallInteractionType::Slide => {
                    // We walked into a surface with normal n.
                    // Find parts of delta parallel and orthogonal to n
                    let n = wall_normal(&c.wall_position()[wall]);
                    let u = n * delta.dot(&n);
                    let v = delta - u;

                    // Move into parallel and orthogonal directions individually
                    let new_a = match line_segment_walls_intersection(a, a + u,
                                                                      wall_aspect, c) {
                        Some((t, _)) => {
                            let s = (t - STEPBACK).max(0.0);
                            a + u * s
                        }
                        None => a + u
                    };
                    let new_a = match line_segment_walls_intersection(new_a, new_a + v,
                                                                      wall_aspect, c) {
                        Some((t, _)) => {
                            let s = (t - STEPBACK).max(0.0);
                            new_a + v * s
                        }
                        None => new_a + v
                    };
                    new_a
                }
                WallInteractionType::Flip => {
                    let n_angle = wall_orientation(&c.wall_position()[wall]);
                    let angle = c.orientation()[e].angle;
                    c.orientation_mut()[e].angle =
                        f32::consts::PI + n_angle - (angle - n_angle);

                    //data.server_net_entity[e].force(ComponentType::Orientation);

                    let v = c.linear_velocity()[e].v;
                    let speed = v.norm();
                    c.linear_velocity_mut()[e].v = Vec2::new(
                        c.orientation()[e].angle.cos() * (speed + 1.0),
                        c.orientation()[e].angle.sin() * (speed + 1.0),
                    );

                    let s = (t - STEPBACK).max(0.0);
                    a + delta * s

                    // TODO: Actually at this point we still might have some 't' left to walk
                }
                WallInteractionType::Stop => {
                    let s = (t - STEPBACK).max(0.0);
                    a + delta * s
                }
            }
        }
        None => b
    };
}

/// Player interaction with wall
pub struct PlayerWallInteraction(PlayerId);
impl<Components: ComponentManager,
     Services: ServiceManager>
    WallInteraction<Components, Services> for PlayerWallInteraction
    where Components: HasPosition + HasOrientation + HasLinearVelocity +
                      HasPlayerState + HasFullPlayerState + HasWallPosition,
          Services: HasEvents {
    fn apply(&self,
             player: EntityData<Components>, wall: EntityData<Components>,
             data: &mut DataHelper<Components, Services>)
             -> WallInteractionType {
        if data.full_player_state()[player].wall_flip {
            let event = GameEvent::PlayerFlip {
                player_id: self.0,
                position: data.position()[player].p,
                orientation: data.orientation()[player].angle,
                speed: data.linear_velocity()[player].v.norm(),
                orientation_wall: wall_orientation(&data.wall_position()[wall]),
            };
            data.services.add_event(&event);

            WallInteractionType::Flip
        } else {
            // If we are dashing and running into a wall, stop dashing soon
            if data.player_state()[player].dashing.is_some() && 
               data.player_state()[player].dashing.unwrap() < 0.9 {
                data.player_state_mut()[player].dashing = Some(0.9);
            }
            WallInteractionType::Slide
        }
    }
}

/// Performs player input on a player-controlled entity, by changing velocities etc.
pub fn run_player_movement_input<Components: ComponentManager,
                                 Services: ServiceManager>
                                (e: EntityData<Components>,
                                 owner: PlayerId,
                                 timed_input: &TimedPlayerInput,
                                 wall_aspect: &CachedAspect<Components>,
                                 c: &mut DataHelper<Components, Services>) 
    where Components: HasPosition + HasLinearVelocity + 
                      HasOrientation + HasAngularVelocity + 
                      HasPlayerState + HasFullPlayerState +
                      HasShape + HasWallPosition,
          Services: HasEvents {
    const TURN_ACCEL: f32 = 1.5;
    const TURN_FRICTION: f32 = 0.25;
    const MOVE_ACCEL: f32 = 1000.0;
    const MOVE_FRICTION: f32 = 10.0;
    const BACK_ACCEL: f32 = 500.0;
    const STRAFE_ACCEL: f32 = 900.0;
    const MIN_SPEED: f32 = 0.1;
    const DASH_SPEED: f32 = 600.0;
    const DASH_DURATION_S: f32 = 0.3;

    let dur_s = timed_input.duration_s;
    let input = &timed_input.input;

    // Cooldowns
    if let Some(dash_cooldown_s) = c.full_player_state()[e].dash_cooldown_s {
        let dash_cooldown_s = dash_cooldown_s - dur_s;
        c.full_player_state_mut()[e].dash_cooldown_s =
            if dash_cooldown_s <= 0.0 { None }
            else { Some(dash_cooldown_s) };
    }
    if let Some(inv_s) = c.player_state()[e].invulnerable_s {
        let inv_s = inv_s - dur_s;
        c.player_state_mut()[e].invulnerable_s =
            if inv_s <= 0.0 { None }
            else { Some(inv_s) };
    }

    c.full_player_state_mut()[e].wall_flip = input.has(PlayerInputKey::Flip);

    // Before changing velocities, move
    let interaction = PlayerWallInteraction(owner);
    move_entity(e, dur_s, &interaction, wall_aspect, c);

    let angle = c.orientation()[e].angle;
    let direction = Vec2::new(angle.cos(), angle.sin());

    if let Some(dashing) = c.player_state()[e].dashing {
        // While dashing, movement input is ignored

        //let t = dashing / DASH_DURATION_S;
        let scale = 1.0; //(t*f32::consts::PI/2.0).cos()*(1.0-(1.0-t).powi(10));
        c.linear_velocity_mut()[e].v = direction * DASH_SPEED * scale;

        c.player_state_mut()[e].dashing =
            if dashing + dur_s <= DASH_DURATION_S {
                Some(dashing + dur_s)
            } else {
                None
            };
    } else {
        c.orientation_mut()[e].angle += c.angular_velocity()[e].v;

        let mut accel = c.linear_velocity_mut()[e].v * -MOVE_FRICTION;

        if input.has(PlayerInputKey::StrafeLeft) {
            c.angular_velocity_mut()[e].v = 0.0;
            let strafe_direction = Vec2::new(direction[1], -direction[0]);
            accel = strafe_direction * STRAFE_ACCEL + accel;
        } else if input.has(PlayerInputKey::StrafeRight) {
            c.angular_velocity_mut()[e].v = 0.0;
            let strafe_direction = Vec2::new(direction[1], -direction[0]);
            accel = -strafe_direction * -STRAFE_ACCEL + accel;
        } else {
            // Turn left/right
            let mut ang_accel = c.angular_velocity()[e].v * -TURN_FRICTION;

            if input.has(PlayerInputKey::Left) {
                ang_accel += TURN_ACCEL * dur_s;
            }
            if input.has(PlayerInputKey::Right) {
                ang_accel -= TURN_ACCEL * dur_s;
            }

            c.angular_velocity_mut()[e].v += ang_accel;
        }

        // Move forward/backward
        if input.has(PlayerInputKey::Forward) {
            accel = direction * MOVE_ACCEL + accel;
        }
        if input.has(PlayerInputKey::Back) {
            accel = -direction * MOVE_ACCEL + accel;
        }

        c.linear_velocity_mut()[e].v = c.linear_velocity()[e].v + accel * dur_s;

        // If velocity is below some limit, set to zero
        if c.linear_velocity()[e].v[0].abs() <= MIN_SPEED {
            c.linear_velocity_mut()[e].v[0] = 0.0;
        }
        if c.linear_velocity()[e].v[1].abs() <= MIN_SPEED {
            c.linear_velocity_mut()[e].v[1] = 0.0;
        }

        // Start dash if the cooldown is ready
        if input.has(PlayerInputKey::Dash) && 
           c.full_player_state()[e].dash_cooldown_s.is_none() {
            c.player_state_mut()[e].dashing = Some(0.0);
            c.full_player_state_mut()[e].dash_cooldown_s = Some(5.0);
            c.angular_velocity_mut()[e].v = 0.0;

            /*let event = GameEvent::PlayerDash {
                player_id: owner,
                position: c.position()[e].p,
                orientation: c.orientation()[e].angle,
            };
            c.services.add_event(&event);*/
        }
    }
}

pub fn line_segment_walls_intersection<'a,
                                       Components: ComponentManager,
                                       Services: ServiceManager>
                                      (a: Vec2<f32>, b: Vec2<f32>,
                                       wall_aspect: &'a CachedAspect<Components>,
                                       data: &mut DataHelper<Components, Services>)
                                       -> Option<(f32, EntityData<'a, Components>)>
        where Components: HasWallPosition {
    let mut closest_i = None;

    for wall in wall_aspect.iter::<'a>() {
        let p = data.wall_position()[wall].clone();
        let i = math::line_segments_intersection(a, b, p.pos_a, p.pos_b);

        if let Some(t) = i {
            closest_i = if let Some((closest_t, closest_e)) = closest_i {
                if t < closest_t { Some((t, wall)) }
                else { Some((closest_t, closest_e)) }
            } else {
                Some((t, wall))
            };
        }
    }

    closest_i
}


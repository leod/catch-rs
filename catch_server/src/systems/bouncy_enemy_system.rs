use std::f32;

use hprof;
use ecs::{Aspect, Process, System, EntityData, DataHelper};

use shared::math;
use shared::map::Map;
use shared::net::ComponentType;
use shared::util::CachedAspect;

use components::{Components};
use services::Services;

const MOVE_ACCEL: f32 = 150.0;
const MOVE_FRICTION: f32 = 4.0;
const ORBIT_SPEED_FACTOR: f32 = 20.0;

pub struct BouncyEnemySystem {
    aspect: CachedAspect<Components>,
}

impl BouncyEnemySystem {
    pub fn new(aspect: Aspect<Components>) -> BouncyEnemySystem {
        BouncyEnemySystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    // TODO: Code duplication
    fn move_flipping(&self,
                     e: EntityData<Components>,
                     delta: math::Vec2,
                     map: &Map,
                     data: &mut Components) {
        let p = data.position[e].p;
        let q = math::add(p, delta);

        match map.line_segment_intersection(p, q) {
            Some(intersection) => {
                let n_angle = intersection.n[1].atan2(intersection.n[0]);
                let angle = data.orientation[e].angle;

                data.orientation[e].angle = f32::consts::PI + n_angle - (angle - n_angle);
                data.server_net_entity[e].force(ComponentType::Orientation);

                let v = data.linear_velocity[e].v;
                let speed = math::square_len(v).sqrt();
                data.linear_velocity[e].v = [
                    data.orientation[e].angle.cos() * (speed + 1.0),
                    data.orientation[e].angle.sin() * (speed + 1.0),
                ];

                let s = (intersection.t - 0.001).max(0.0);
                data.position[e].p = math::add(p, math::scale(delta, s));
            }
            None => {
                data.position[e].p = q;
            }
        };
    }

    pub fn tick(&self, map: &Map, data: &mut DataHelper<Components, Services>) {
        let _g = hprof::enter("bouncy enemy");

        let dur_s = data.services.tick_dur_s;

        for e in self.aspect.iter() {
            let accel = if let Some(orbit) = data.bouncy_enemy[e].orbit {
                self.move_flipping(e, math::scale(data.linear_velocity[e].v, dur_s), map, data);

                if let Some(orbit_position) =
                        data.with_entity_data(&orbit, |e, c| { c.position[e].p }) {
                    let w = math::sub(orbit_position, data.position[e].p);
                    let r = math::square_len(w).sqrt();
                    let f = r;
                    math::add(math::scale(math::normalized(w), f*ORBIT_SPEED_FACTOR),
                              math::scale(data.linear_velocity[e].v, -MOVE_FRICTION))
                } else {
                    data.bouncy_enemy[e].orbit = None;
                    [0.0, 0.0]
                }
            } else {
                self.move_flipping(e, math::scale(data.linear_velocity[e].v, dur_s), map, data);

                let angle = data.orientation[e].angle;
                let direction = [angle.cos(), angle.sin()];

                math::add(math::scale(direction, MOVE_ACCEL),
                          math::scale(data.linear_velocity[e].v, -MOVE_FRICTION))
            };

            data.linear_velocity[e].v = math::add(data.linear_velocity[e].v,
                                                  math::scale(accel, dur_s));
        }
    }
}

impl_cached_system!(Components, Services, BouncyEnemySystem, aspect);

impl Process for BouncyEnemySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

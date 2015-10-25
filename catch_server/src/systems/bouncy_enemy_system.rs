use hprof;
use ecs::{Aspect, Process, System, DataHelper};

use shared::math;
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

    pub fn tick(&self, data: &mut DataHelper<Components, Services>) {
        let _g = hprof::enter("bouncy enemy");

        let dur_s = data.services.tick_dur_s;

        for e in self.aspect.iter() {
            let accel = if let Some(orbit) = data.bouncy_enemy[e].orbit {
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

use hprof;
use ecs::{Aspect, Process, System, DataHelper};
use na::{Vec2, Norm};

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
                    let w = orbit_position - data.position[e].p;
                    let r = w.norm();
                    let f = r;

                    w.normalize() * f * ORBIT_SPEED_FACTOR -
                        data.linear_velocity[e].v * MOVE_FRICTION
                } else {
                    data.bouncy_enemy[e].orbit = None;
                    Vec2::new(0.0, 0.0)
                }
            } else {
                let angle = data.orientation[e].angle;
                let direction = Vec2::new(angle.cos(), angle.sin());

                direction * MOVE_ACCEL - data.linear_velocity[e].v * MOVE_FRICTION
            };

            data.linear_velocity[e].v = data.linear_velocity[e].v + accel * dur_s;
        }
    }
}

impl_cached_system!(Components, Services, BouncyEnemySystem, aspect);

impl Process for BouncyEnemySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

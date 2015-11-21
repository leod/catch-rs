use hprof;
use ecs::{Aspect, Process, System, DataHelper};
use na::{Vec2, Norm};

use shared::util::CachedAspect;

use components::{Components, Shape};
use services::Services;

const MAX_SPEED: f32 = 300.0;
const MOVE_ACCEL: f32 = 150.0;
const MOVE_FRICTION: f32 = 4.0;
const ORBIT_SPEED_FACTOR: f32 = 10.0;
const ORBIT_BUFFER: f32 = 20.0;

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
                if let Some((orbit_position, orbit_shape)) =
                        data.with_entity_data(&orbit, |e, c| {
                            (c.position[e].p, c.shape[e].clone())
                        }) {
                    let w = orbit_position - data.position[e].p;
                    let f = w.norm();
                    let r = match orbit_shape {
                        Shape::Circle { radius } => radius,
                        _ => panic!("not implemented")
                    };
                    let d = if f >= r + ORBIT_BUFFER {
                        w * ORBIT_SPEED_FACTOR * f
                    } else {
                        -w * ORBIT_SPEED_FACTOR * f
                    };
                    d - data.linear_velocity[e].v * MOVE_FRICTION
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

            let speed = data.linear_velocity[e].v.norm();
            if speed > MAX_SPEED {
                data.linear_velocity[e].v = data.linear_velocity[e].v / speed * MAX_SPEED;
            }
        }
    }
}

impl_cached_system!(Components, Services, BouncyEnemySystem, aspect);

impl Process for BouncyEnemySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

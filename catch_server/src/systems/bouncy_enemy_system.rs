use hprof;
use ecs::{Aspect, Process, System, DataHelper};
use na::{Vec2, Norm};

use shared::GameEvent;
use shared::util::CachedAspect;
use shared::services::HasEvents;

use components::Components;
use services::Services;
use entities;

const MAX_SPEED: f32 = 300.0;
const MOVE_ACCEL: f32 = 150.0;
const MOVE_FRICTION: f32 = 4.0;
const ORBIT_SPEED_FACTOR: f32 = 2.0;
const ORBIT_BUFFER: f32 = 14.0;

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
            let accel =
                if let Some(orbit) = data.bouncy_enemy[e].orbit {
                    if let Some((orbit_position, orbit_shape)) =
                            data.with_entity_data(&orbit, |e, c| {
                                (c.position[e].p, c.shape[e].clone())
                            }) {
                        let r = orbit_shape.radius();
                        let d = orbit_position - data.position[e].p;
                        let d = if d.norm() <= 0.0001 {
                            Vec2::new(1.0, 0.0)
                        } else {
                            d
                        };
                        let p_orbit = orbit_position - d.normalize() * (r + ORBIT_BUFFER);
                        let target_d = p_orbit - data.position[e].p;
                        let t = target_d.norm();

                        target_d.normalize() * t * t * t * ORBIT_SPEED_FACTOR -
                            data.linear_velocity[e].v * 1.4

                        /*let f = d.norm();
                        let x = if f >= r + ORBIT_BUFFER {
                            d * ORBIT_SPEED_FACTOR * f
                        } else {
                            -d * ORBIT_SPEED_FACTOR * f
                        };
                        x - data.linear_velocity[e].v * MOVE_FRICTION*/
                    } else {
                        let event = GameEvent::EnemyDied {
                            position: data.position[e].p
                        };
                        data.services.add_event(&event);
                        entities::remove_net(**e, data);
                        continue;
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

use std::f64;

use ecs;
use ecs::{Aspect, Process, System, EntityData, DataHelper};

use shared::math;
use shared::map::Map;
use shared::net::ComponentType;
use shared::util::CachedAspect;
use components::{Components};
use services::Services;

pub struct BouncyEnemySystem {
    aspect: CachedAspect<Components>,
}

impl BouncyEnemySystem {
    pub fn new(aspect: Aspect<Components>) -> BouncyEnemySystem {
        BouncyEnemySystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    // TODO: Code duplication with PlayerMovementSystem...

    fn move_straight(&self,
                     e: EntityData<Components>,
                     delta: math::Vec2,
                     map: &Map,
                     data: &mut Components) {
        let p = data.position[e].p;
        let q = math::add(p, delta);

        data.position[e].p = match map.line_segment_intersection(p, q) {
            Some((_, _, _, s)) => { // Walk as far as we can
                let s = (s - 0.0001).max(0.0);
                math::add(p, math::scale(delta, s))
            }
            None =>
                q
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

    pub fn tick(&self, map: &Map, data: &mut DataHelper<Components, Services>) {
        const MOVE_ACCEL: f64 = 400.0;

        let dur_s = data.services.tick_dur_s;

        for e in self.aspect.iter() {
            let angle = data.orientation[e].angle;
            let direction = [angle.cos(), angle.sin()];

            let accel = math::add(math::scale(direction, MOVE_ACCEL),
                                  math::scale(data.linear_velocity[e].v, -4.0));
            data.linear_velocity[e].v = math::add(data.linear_velocity[e].v,
                                                  math::scale(accel, dur_s));

            self.move_flipping(e, math::scale(data.linear_velocity[e].v, dur_s), map, data);
        }
    }
}

impl System for BouncyEnemySystem {
    type Components = Components;
    type Services = Services;

    fn activated(&mut self, entity: &EntityData<Components>, components: &Components,
                 _: &mut Services) {
        self.aspect.activated(entity, components);
    }

    fn reactivated(&mut self, entity: &EntityData<Components>, components: &Components,
                   _: &mut Services) {
        self.aspect.reactivated(entity, components);
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, components: &Components,
                   _: &mut Services) {
        self.aspect.deactivated(entity, components);
    }
}

impl Process for BouncyEnemySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

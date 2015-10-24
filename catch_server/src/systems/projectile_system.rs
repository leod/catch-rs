use hprof;
use ecs::{Aspect, Process, System, EntityData, DataHelper};

use shared::{GameEvent, Map};
use shared::math;
use shared::net::ComponentType;
use shared::util::CachedAspect;
use shared::services::HasEvents;

use components::{Components};
use services::Services;
use entities;

pub struct ProjectileSystem {
    aspect: CachedAspect<Components>,
}

impl ProjectileSystem {
    pub fn new(aspect: Aspect<Components>) -> ProjectileSystem {
        ProjectileSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn tick(&self, map: &Map, data: &mut DataHelper<Components, Services>) {
        let _g = hprof::enter("projectile");

        let dur_s = data.services.tick_dur_s;

        for e in self.aspect.iter() {
            let p = data.position[e].p;
            let v = math::scale(data.linear_velocity[e].v, dur_s);
            let new_p = math::add(p, v);

            match map.line_segment_intersection(p, new_p) {
                Some(intersection) => {
                    let delta = math::sub(new_p, p);
                    let position = math::add(p, math::scale(delta, intersection.t));
                    data.services.add_event(&GameEvent::ProjectileImpact {
                        position: position,
                    });
                                    
                    entities::remove_net(**e, data);
                }
                None => {
                    data.position[e].p = new_p;
                }
            };
        }
    }
}

impl_cached_system!(Components, Services, ProjectileSystem, aspect);

impl Process for ProjectileSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

use hprof;
use ecs::{Aspect, Process, System, DataHelper};

use shared::util::CachedAspect;

use components::{Components};
use services::Services;

pub struct ProjectileSystem {
    aspect: CachedAspect<Components>,
}

impl ProjectileSystem {
    pub fn new(aspect: Aspect<Components>) -> ProjectileSystem {
        ProjectileSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn tick(&self, data: &mut DataHelper<Components, Services>) {
        let _g = hprof::enter("projectile");

        let _dur_s = data.services.tick_dur_s;

        for _ in self.aspect.iter() {
        }
    }
}

impl_cached_system!(Components, Services, ProjectileSystem, aspect);

impl Process for ProjectileSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

use hprof;
use ecs::{Aspect, Process, DataHelper};

use shared::util::CachedAspect;

use components::Components;
use services::Services;

pub struct RotateSystem {
    aspect: CachedAspect<Components>,
}

impl RotateSystem {
    pub fn new(aspect: Aspect<Components>) -> RotateSystem {
        RotateSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn tick(&self, data: &mut DataHelper<Components, Services>) {
        let _g = hprof::enter("rotate");

        for e in self.aspect.iter() {
            data.orientation[e].angle += data.angular_velocity[e].v * data.services.tick_dur_s;
        }
    }
}

impl_cached_system!(Components, Services, RotateSystem, aspect);

impl Process for RotateSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

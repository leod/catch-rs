use ecs::{Aspect, System, DataHelper, Process};

use shared::util::CachedAspect;
use components::{Components, Interpolatable, Position, Orientation};
use services::Services;

pub struct InterpolationSystem {
    position_aspect: CachedAspect<Components>,
    orientation_aspect: CachedAspect<Components>,
}

impl InterpolationSystem {
    pub fn new(position_aspect: Aspect<Components>,
               orientation_aspect: Aspect<Components>) -> InterpolationSystem {
        InterpolationSystem {
            position_aspect: CachedAspect::new(position_aspect),
            orientation_aspect: CachedAspect::new(orientation_aspect),
        }
    }

    pub fn interpolate(&self, t: f64, data: &mut DataHelper<Components, Services>) {
        for e in self.position_aspect.iter() {
            if let Some((a, b)) = data.interp_position[e].state.clone() {
                data.position[e] = Position::interpolate(&a, &b, t);
            }
        }

        for e in self.orientation_aspect.iter() {
            if let Some((a, b)) = data.interp_orientation[e].state.clone() {
                data.orientation[e] = Orientation::interpolate(&a, &b, t);
            }
        }
    }
}

impl_cached_system!(Components, Services, InterpolationSystem,
                    position_aspect, orientation_aspect);

impl Process for InterpolationSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

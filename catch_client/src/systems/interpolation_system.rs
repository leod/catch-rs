use ecs::{Aspect, System, EntityData, DataHelper, Process};

use shared::math;
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

impl System for InterpolationSystem {
    type Components = Components;
    type Services = Services;

    fn activated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.position_aspect.activated(entity, components);
        self.orientation_aspect.activated(entity, components);
    }

    fn reactivated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.position_aspect.reactivated(entity, components);
        self.orientation_aspect.reactivated(entity, components);
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.position_aspect.deactivated(entity, components);
        self.orientation_aspect.deactivated(entity, components);
    }
}

impl Process for InterpolationSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

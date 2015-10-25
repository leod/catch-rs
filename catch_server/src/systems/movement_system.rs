use hprof;
use ecs::{Aspect, Process, System, DataHelper, EntityData};

use shared::util::CachedAspect;
use shared::movement;

use components::Components;
use services::Services;

pub type WallInteraction = movement::WallInteraction<Components, Services>;

pub struct MovementSystem {
    wall_aspect: CachedAspect<Components>,
    aspects: Vec<(CachedAspect<Components>, Box<WallInteraction>)>,
}

impl MovementSystem {
    pub fn new(wall_aspect: Aspect<Components>,
               aspects: Vec<(Aspect<Components>, Box<WallInteraction>)>)
               -> MovementSystem {
        MovementSystem {
            wall_aspect: CachedAspect::new(wall_aspect),
            aspects:
                aspects.into_iter()
                       .map(|(a, i)| (CachedAspect::new(a), i))
                       .collect(),
        }
    }

    pub fn tick(&self, data: &mut DataHelper<Components, Services>) {
        let _g = hprof::enter("movement");

        for &(ref aspect, ref interaction) in self.aspects.iter() {
            for e in aspect.iter() {
                movement::move_entity(e, data.services.tick_dur_s, &**interaction,
                                      &self.wall_aspect, data);
            }
        }
    }
}

impl System for MovementSystem {
    type Components = Components;
    type Services = Services;

    fn activated(&mut self, entity: &EntityData<Components>, components: &Components,
                 _: &mut Services) {
        self.wall_aspect.activated(entity, components);
        for &mut (ref mut aspect, _) in self.aspects.iter_mut() {
            aspect.activated(entity, components);
        }
    }

    fn reactivated(&mut self, entity: &EntityData<Components>, components: &Components,
                   _: &mut Services) {
        self.wall_aspect.reactivated(entity, components);
        for &mut (ref mut aspect, _) in self.aspects.iter_mut() {
            aspect.reactivated(entity, components);
        }
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, components: &Components,
                   _: &mut Services) {
        self.wall_aspect.deactivated(entity, components);
        for &mut (ref mut aspect, _) in self.aspects.iter_mut() {
            aspect.deactivated(entity, components);
        }
    }
}

impl Process for MovementSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

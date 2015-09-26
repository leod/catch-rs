use ecs::{System, Process, Aspect, EntityData, DataHelper};

use shared::math;
use shared::util::CachedAspect;
use components::{Components, Shape}; 
use services::Services;

/// Defines a conditional interaction between two entities
pub trait Interaction {
    fn condition(&self,
                 _a: EntityData<Components>, _b: EntityData<Components>,
                 _data: &mut DataHelper<Components, Services>) -> bool {
        true
    }

    fn apply(&self,
             a: EntityData<Components>, b: EntityData<Components>,
             data: &mut DataHelper<Components, Services>);
}

/// Each entry of the dispatch table is a pair of entity filters coupled with an interaction
/// that is to be applied when two of these entities overlap
type DispatchTable = Vec<(CachedAspect<Components>,
                          CachedAspect<Components>,
                          Box<Interaction>)>;

pub struct InteractionSystem {
    dispatch_table: DispatchTable,
}

impl InteractionSystem {
    pub fn new(dispatch_table: Vec<(Aspect<Components>, Aspect<Components>, Box<Interaction>)>)
               -> InteractionSystem {
        InteractionSystem {
            dispatch_table:
                dispatch_table.into_iter()
                              .map(|(a, b, i)| (CachedAspect::new(a), CachedAspect::new(b), i))
                              .collect()
        }
    }

    pub fn tick(&self, data: &mut DataHelper<Components, Services>) {
        // n^2 kinda loop over all entity pairs that can interact
        
        for &(ref aspect_a, ref aspect_b, ref interaction) in self.dispatch_table.iter() {
            for entity_a in aspect_a.iter() {
                for entity_b in aspect_b.iter() {
                    if **entity_a == **entity_b {
                        continue;
                    }

                    if interaction.condition(entity_a, entity_b, data) &&
                       self.overlap(entity_a, entity_b, &data.components) {
                        interaction.apply(entity_a, entity_b, data);
                    }
                }
            }
        }
    }

    /// Checks if two entities can interact right now. We simply do this by checking if they
    /// currently overlap. In the future, it might be necessary to consider movement, though.
    fn overlap(&self,
               e_a: EntityData<Components>,
               e_b: EntityData<Components>,
               c: &Components)
               -> bool {
        match (&c.shape[e_a], &c.shape[e_b]) {
            (&Shape::Circle { radius: r_a }, &Shape::Circle { radius: r_b }) => {
                let d = math::square_len(math::sub(c.position[e_a].p, c.position[e_b].p)).sqrt();

                d <= r_a + r_b
            }

            (&Shape::Circle { radius: r }, &Shape::Square { size: s }) => {
                // TODO
                let d = math::square_len(math::sub(c.position[e_a].p, c.position[e_b].p)).sqrt();

                d <= r + s * 2.0
            }

            (&Shape::Circle { radius: r }, &Shape::Rect { width: w, height: h }) => {
                // TODO
                let d = math::square_len(math::sub(c.position[e_a].p, c.position[e_b].p)).sqrt();

                d <= r + w.max(h) * 2.0
            }

            (&Shape::Square { size: _ }, &Shape::Circle { radius: _ }) =>
                self.overlap(e_b, e_a, c),
            (&Shape::Rect { width: _, height: _ }, &Shape::Circle { radius: _ }) =>
                self.overlap(e_b, e_a, c),

            (shape_a, shape_b) =>
                panic!("shape interaction not implemented: {:?}, {:?}", shape_a, shape_b),
        }
    }
}

impl System for InteractionSystem {
    type Components = Components;
    type Services = Services;

    fn activated(&mut self, entity: &EntityData<Components>, components: &Components,
                 _: &mut Services) {
        for &mut (ref mut aspect_a, ref mut aspect_b, _) in self.dispatch_table.iter_mut() {
            aspect_a.activated(entity, components);
            aspect_b.activated(entity, components);
        }
    }

    fn reactivated(&mut self, entity: &EntityData<Components>, components: &Components,
                   _: &mut Services) {
        for &mut (ref mut aspect_a, ref mut aspect_b, _) in self.dispatch_table.iter_mut() {
            aspect_a.reactivated(entity, components);
            aspect_b.reactivated(entity, components);
        }
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, components: &Components,
                   _: &mut Services) {
        for &mut (ref mut aspect_a, ref mut aspect_b, _) in self.dispatch_table.iter_mut() {
            aspect_a.deactivated(entity, components);
            aspect_b.deactivated(entity, components);
        }
    }
}

impl Process for InteractionSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

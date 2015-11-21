use hprof;
use ecs::{System, Process, Aspect, EntityData, DataHelper};
use na::Norm;

use shared::math;
use shared::util::CachedAspect;
use shared::movement::{self, WallInteractionType};

use components::{Components, Shape}; 
use services::Services;
use systems::wall_interactions::ConstWallInteraction;

pub enum InteractionResponse {
    None,
    DisplaceNoOverlap,  
}

/// Defines a conditional interaction between two entities
pub trait Interaction {
    fn condition(&self,
                 _a: EntityData<Components>, _b: EntityData<Components>,
                 _data: &mut DataHelper<Components, Services>) -> bool {
        true
    }

    fn apply(&self,
             a: EntityData<Components>, b: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) -> InteractionResponse;
}

pub struct InteractionSystem {
    /// Walls in the map
    wall_aspect: CachedAspect<Components>,

    /// Interactions between two different entity types
    interactions: Vec<(CachedAspect<Components>, CachedAspect<Components>, Box<Interaction>)>,

    /// Interactions between entities of the same type
    self_interactions: Vec<(CachedAspect<Components>, Box<Interaction>)>,
}

impl InteractionSystem {
    pub fn new(
            wall_aspect: Aspect<Components>,
            interactions: Vec<(Aspect<Components>, Aspect<Components>, Box<Interaction>)>,
            self_interactions: Vec<(Aspect<Components>, Box<Interaction>)>)
            -> InteractionSystem {
        InteractionSystem {
            wall_aspect: CachedAspect::new(wall_aspect),
            interactions:
                interactions.into_iter()
                            .map(|(a, b, i)| (CachedAspect::new(a), CachedAspect::new(b), i))
                            .collect(),
            self_interactions:
                self_interactions.into_iter()
                                 .map(|(a, i)| (CachedAspect::new(a), i))
                                 .collect()
        }
    }

    pub fn tick(&self, data: &mut DataHelper<Components, Services>) {
        // n^2 kinda loop over all entity pairs that can interact
        
        let _g = hprof::enter("interaction");

        for &(ref aspect_a, ref aspect_b, ref interaction) in self.interactions.iter() {
            for entity_a in aspect_a.iter() {
                for entity_b in aspect_b.iter() {
                    InteractionSystem::try_interaction(&**interaction, entity_a, entity_b,
                                                       &self.wall_aspect, data);
                }
            }
        }

        for &(ref aspect, ref interaction) in self.self_interactions.iter() {
            for entity_a in aspect.iter() {
                for entity_b in aspect.iter() {
                    if entity_a.index() <= entity_b.index() {
                        // Don't perform interactions twice
                        continue;
                    }

                    InteractionSystem::try_interaction(&**interaction, entity_a, entity_b,
                                                       &self.wall_aspect, data);
                }
            }
        }
    }

    fn try_interaction(interaction: &Interaction,
                       e_a: EntityData<Components>,
                       e_b: EntityData<Components>,
                       wall_aspect: &CachedAspect<Components>,
                       c: &mut DataHelper<Components, Services>) {
        if interaction.condition(e_a, e_b, c) &&
           InteractionSystem::overlap(e_a, e_b, &c.components) {
            let response = interaction.apply(e_a, e_b, c);

            match response {
                InteractionResponse::None => {
                }
                InteractionResponse::DisplaceNoOverlap => {
                    // Displace the shapes so they no longer overlap
                    let p1 = c.position[e_a].p;
                    let p2 = c.position[e_b].p;
                    let delta = p2 - p1;
                    let cur_dist = delta.norm();
                    let min_dist_no_overlap = c.shape[e_a].radius() + c.shape[e_b].radius() + 0.05;
                    assert!(min_dist_no_overlap > cur_dist); // otherwise why are we here?
                    let delta_no_overlap = delta.normalize() * (min_dist_no_overlap - cur_dist);
                    let interaction = ConstWallInteraction(WallInteractionType::Flip);
                    movement::move_entity(e_a, delta_no_overlap * -0.5, &interaction,
                                          &wall_aspect, c);
                    movement::move_entity(e_b, delta_no_overlap * 0.5, &interaction,
                                          &wall_aspect, c);
                }
            }
        }
    }

    /// Checks if two entities can interact right now. We simply do this by checking if they
    /// currently overlap. In the future, it might be necessary to consider movement, though.
    fn overlap(e_a: EntityData<Components>,
               e_b: EntityData<Components>,
               c: &Components)
               -> bool {
        let p_a = c.position[e_a].p;
        let p_b = c.position[e_b].p;

        match (&c.shape[e_a], &c.shape[e_b]) {
            (&Shape::Circle { radius: r_a }, &Shape::Circle { radius: r_b }) => {
                let d = (p_a - p_b).norm();
                d <= r_a + r_b
            }
            (&Shape::Circle { radius: r }, &Shape::Square { size: s }) => {
                // TODO
                let angle = c.orientation[e_b].angle;
                math::rect_circle_overlap(p_b, s, s, angle, p_a, r)
            }
            (&Shape::Circle { radius: r }, &Shape::Rect { width: w, height: h }) => {
                let angle = c.orientation[e_b].angle;
                math::rect_circle_overlap(p_b, w, h, angle, p_a, r)
            }

            // Try the other way around...
            (&Shape::Square { size: _ }, &Shape::Circle { radius: _ }) =>
                InteractionSystem::overlap(e_b, e_a, c),
            (&Shape::Rect { width: _, height: _ }, &Shape::Circle { radius: _ }) =>
                InteractionSystem::overlap(e_b, e_a, c),

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
        self.wall_aspect.activated(entity, components);
        for &mut (ref mut aspect_a, ref mut aspect_b, _) in self.interactions.iter_mut() {
            aspect_a.activated(entity, components);
            aspect_b.activated(entity, components);
        }
        for &mut (ref mut aspect, _) in self.self_interactions.iter_mut() {
            aspect.activated(entity, components);
        }
    }

    fn reactivated(&mut self, entity: &EntityData<Components>, components: &Components,
                   _: &mut Services) {
        self.wall_aspect.reactivated(entity, components);
        for &mut (ref mut aspect_a, ref mut aspect_b, _) in self.interactions.iter_mut() {
            aspect_a.reactivated(entity, components);
            aspect_b.reactivated(entity, components);
        }
        for &mut (ref mut aspect, _) in self.self_interactions.iter_mut() {
            aspect.reactivated(entity, components);
        }
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, components: &Components,
                   _: &mut Services) {
        self.wall_aspect.deactivated(entity, components);
        for &mut (ref mut aspect_a, ref mut aspect_b, _) in self.interactions.iter_mut() {
            aspect_a.deactivated(entity, components);
            aspect_b.deactivated(entity, components);
        }
        for &mut (ref mut aspect, _) in self.self_interactions.iter_mut() {
            aspect.deactivated(entity, components);
        }
    }
}

impl Process for InteractionSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

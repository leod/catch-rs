use std::f64;

use ecs::{System, Process, Aspect, EntityData, DataHelper};

use shared::math;
use shared::util::CachedAspect;
use shared::event::GameEvent;
use shared::player::NEUTRAL_PLAYER_ID;
use components::{Components, Shape, Position, Orientation};
use services::Services;

pub struct PlayerBouncyEnemyInteraction;
pub struct BouncyEnemyBouncyEnemyInteraction;

impl Interaction for PlayerBouncyEnemyInteraction {
    fn apply(&self,
             player_e: EntityData<Components>, enemy_e: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        if data.player_state[player_e].vulnerable() {
            // Kill player
            let owner = data.net_entity[player_e].owner;
            data.services.add_event_to_run(&GameEvent::PlayerDied(owner, NEUTRAL_PLAYER_ID));
        }
    }
}

impl Interaction for BouncyEnemyBouncyEnemyInteraction {
    fn apply(&self,
             a_e: EntityData<Components>, b_e: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        // Flip orientations of both entities and add some velocity in the new direction

        data.orientation[a_e].angle = data.orientation[a_e].angle + f64::consts::PI;
        let direction_a = [data.orientation[a_e].angle.cos(),
                           data.orientation[a_e].angle.sin()];
        data.linear_velocity[a_e].v = math::add(data.linear_velocity[a_e].v,
                                                math::scale(direction_a, 500.0));

        data.orientation[b_e].angle = data.orientation[b_e].angle + f64::consts::PI;
        let direction_b = [data.orientation[b_e].angle.cos(),
                           data.orientation[b_e].angle.sin()];
        data.linear_velocity[b_e].v = math::add(data.linear_velocity[b_e].v,
                                                math::scale(direction_b, 500.0));
    }
}

/// Defines a conditional interaction between two entities
pub trait Interaction {
    fn condition(&self,
                 a: EntityData<Components>, b: EntityData<Components>,
                 data: &mut DataHelper<Components, Services>) -> bool {
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
            (&Shape::Circle { radius: ref r_a }, &Shape::Circle { radius: ref r_b }) => {
                let d = math::square_len(math::sub(c.position[e_a].p, c.position[e_b].p)).sqrt();

                d <= (*r_a + *r_b).abs()
            }
            //_ => panic!("shape interaction not implemented: {:?}, {:?}", shape_a, shape_b),
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

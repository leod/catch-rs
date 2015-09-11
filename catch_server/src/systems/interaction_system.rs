use std::f64;

use ecs::{System, Process, Aspect, EntityData, DataHelper};

use shared::math;
use shared::util::CachedAspect;
use components::{Components, Shape, Position, Orientation};
use services::Services;

pub struct PlayerBouncyEnemyInteraction;
pub struct BouncyEnemyBouncyEnemyInteraction;

pub const PLAYER_BOUNCY_ENEMY_INTERACTION: &'static PlayerBouncyEnemyInteraction = &PlayerBouncyEnemyInteraction;
pub const BOUNCY_ENEMY_BOUNCY_ENEMY_INTERACTION: &'static BouncyEnemyBouncyEnemyInteraction = &BouncyEnemyBouncyEnemyInteraction;

impl Interaction for PlayerBouncyEnemyInteraction {
    fn apply(&self,
             player_e: EntityData<Components>, enemy_e: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        // Kill player
    }
}

impl Interaction for BouncyEnemyBouncyEnemyInteraction {
    fn apply(&self,
             a_e: EntityData<Components>, b_e: EntityData<Components>,
             data: &mut DataHelper<Components, Services>) {
        data.orientation[a_e].angle = data.orientation[a_e].angle + f64::consts::PI;
        let direction_a = [data.orientation[a_e].angle.cos(),
                           data.orientation[a_e].angle.sin()];
        data.linear_velocity[a_e].v = math::add(data.linear_velocity[a_e].v, math::scale(direction_a, 500.0));

        data.orientation[b_e].angle = data.orientation[b_e].angle + f64::consts::PI;
        let direction_b = [data.orientation[b_e].angle.cos(),
                           data.orientation[b_e].angle.sin()];
        data.linear_velocity[b_e].v = math::add(data.linear_velocity[b_e].v, math::scale(direction_b, 500.0));
    }
}

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

pub struct InteractionSystem {
    pub interact_aspect: CachedAspect<Components>,

    // might make these aspects cached
    pub dispatch_table: Vec<(Aspect<Components>, Aspect<Components>, &'static Interaction)>,
}

impl InteractionSystem {
    pub fn new(interact_aspect: Aspect<Components>,
               dispatch_table: Vec<(Aspect<Components>, Aspect<Components>, &'static Interaction)>) -> InteractionSystem {
        InteractionSystem {
            interact_aspect: CachedAspect::new(interact_aspect),
            dispatch_table: dispatch_table,
        }
    }

    pub fn tick(&self, data: &mut DataHelper<Components, Services>) {
        // n^2 loop over all entity pairs 

        for entity_a in self.interact_aspect.iter() {
            for entity_b in self.interact_aspect.iter() {
                // We don't want to check entity pairs twice
                // (nor do want to check an entity with itself)
                if entity_a.index() >= entity_b.index() {
                    continue;
                }

                // Check for an interaction?
                match self.get_interaction(entity_a, entity_b, &mut data.components) {
                    Some((flip_arguments, interaction)) => {
                        // This pair seems interesting, check for overlap (or later maybe: intersection)

                        let (e_a, e_b) =
                            if flip_arguments { (entity_b, entity_a) }
                            else { (entity_a, entity_b) };

                        if interaction.condition(e_a, e_b, data) &&
                           self.overlap(e_a, e_b, &data.components) {
                            // Go for the interaction then
                            interaction.apply(e_a, e_b, data);
                        }
                    }
                    None => ()
                };
            }
        }
    }

    // Returns the first matching interaction if there is one
    fn get_interaction(&self,
                       entity_a: EntityData<Components>,
                       entity_b: EntityData<Components>,
                       components: &mut Components) -> Option<(bool, &'static Interaction)> {
        for &(ref aspect_a, ref aspect_b, ref interaction) in self.dispatch_table.iter() {
            if aspect_a.check(&entity_a, components) &&
               aspect_b.check(&entity_b, components) {
                return Some((false, *interaction));
            }

            if aspect_a.check(&entity_b, components) &&
               aspect_b.check(&entity_a, components) {
                return Some((true, *interaction));
            }
        }

        None
    }

    // For now we will just check if entity shapes overlap.
    // It might be necessary to consider movement, though.
    fn overlap(&self, e_a: EntityData<Components>, e_b: EntityData<Components>, c: &Components) -> bool {
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

    fn activated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.interact_aspect.activated(entity, components);
    }

    fn reactivated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.interact_aspect.reactivated(entity, components);
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.interact_aspect.deactivated(entity, components);
    }
}

impl Process for InteractionSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

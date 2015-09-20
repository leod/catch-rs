use ecs::{BuildData};

use shared;

use components::{Components, LinearVelocity, BouncyEnemy};

/// Adds server-side components that are not synchronized over the net to an entity
pub fn build(type_name: &str,
             entity: BuildData<Components>,
             data: &mut Components) {
    shared::entities::build(type_name, entity, data);

    if type_name == "player" {
        //data.interact.add(&entity, Interact);
    } else if type_name == "bouncy_enemy" {
        //data.interact.add(&entity, Interact);
        data.linear_velocity.add(&entity, LinearVelocity::default());
        data.bouncy_enemy.add(&entity, BouncyEnemy::default());
    } else {
        panic!("Unknown net entity type: {}", type_name);
    }
}

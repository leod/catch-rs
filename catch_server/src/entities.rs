use std::f64;

use ecs;
use ecs::{BuildData, DataHelper, EntityBuilder};

use shared;
use shared::{PlayerId, GameEvent};

use components::{Components, NetEntity, ServerNetEntity, LinearVelocity, BouncyEnemy, ItemSpawn,
                 AngularVelocity, Rotate};
use services::Services;

/// Create a new networked entity, replicating it to the clients
pub fn build_net(type_name: &str,
                 owner: PlayerId,
                 data: &mut DataHelper<Components, Services>) 
                 -> ecs::Entity {
    build_net_custom(type_name, owner, data, ())
}

/// Create a net entity and add some custom components to it using an EntityBuilder
pub fn build_net_custom<B: EntityBuilder<Components>>
                       (type_name: &str,
                        owner: PlayerId,
                        data: &mut DataHelper<Components, Services>,
                        builder: B) -> ecs::Entity {
    let entity_type_id = data.services.entity_type_id(type_name);
    let entity_type = data.services.entity_types[entity_type_id as usize].1.clone();
    let entity_id = data.services.next_entity_id();

    info!("Building {} net entity {} for {}", type_name, entity_id, owner);

    // Tell the clients about the new entity
    data.services.add_event(
        &GameEvent::CreateEntity(entity_id, entity_type_id, owner));

    // Get the component type traits needed for this entity type.
    // We will then use the trait objects to add net components to our entity
    let all_traits = shared::components::component_type_traits::<Components>();
    let traits = entity_type.component_types.iter()
                            .chain(entity_type.owner_component_types.iter())
                            .map(|t| &all_traits[*t]);

    data.create_entity(|entity: BuildData<Components>, data: &mut Components| {
        data.net_entity.add(&entity, NetEntity {
            id: entity_id,
            type_id: entity_type_id,
            owner: owner,
        });
        data.server_net_entity.add(&entity, ServerNetEntity::default());

        // Add net components to the entity using its component type traits
        for component_type in traits {
            component_type.add(entity, data);
        }

        // Add shared components that don't need to be synchronized
        shared::entities::build_shared(type_name, entity, data);

        // Add server-side only components
        build_server(type_name, entity, data);

        // Possibly add some custom components
        builder.build(entity, data);
    })
}

/// Adds server-side components that are not synchronized over the net to an entity
pub fn build_server(type_name: &str,
                    entity: BuildData<Components>,
                    data: &mut Components) {
    if type_name == "player" {
        //data.interact.add(&entity, Interact);
        //data.rotate.add(&entity, Rotate);
        data.angular_velocity.add(&entity, AngularVelocity::default());
    } else if type_name == "bouncy_enemy" {
        //data.interact.add(&entity, Interact);
        data.linear_velocity.add(&entity, LinearVelocity::default());
        data.bouncy_enemy.add(&entity, BouncyEnemy::default());
    } else if type_name == "item_spawn" {
        data.item_spawn.add(&entity, ItemSpawn::default());
    } else if type_name == "item" {
        data.angular_velocity.add(&entity, AngularVelocity { v: f64::consts::PI });
        data.rotate.add(&entity, Rotate);
    } else {
        panic!("Unknown net entity type: {}", type_name);
    }
}

/// Removes a net entity and tells clients about the removal
pub fn remove_net(entity: ecs::Entity, data: &mut DataHelper<Components, Services>) {
    let id = data.with_entity_data(&entity, |e, c| {
        c.net_entity[e].id
    }).unwrap();

    data.services.add_event(&GameEvent::RemoveEntity(id));
    data.remove_entity(entity);
}

use ecs;
use ecs::{Aspect, System, DataHelper, BuildData, Process};

use shared;
use shared::util::CachedAspect;
use shared::net_components::{NetComponents, ComponentType};
use shared::tick::EntityPair;
use shared::{Tick, GameEvent, PlayerId, EntityId, EntityTypes, EntityTypeId};

use components::{Components, NetEntity, InterpolationState};
use entities;
use services::Services;

pub struct NetEntitySystem {
    aspect: CachedAspect<Components>,
    entity_types: EntityTypes,
    my_id: PlayerId,
}

impl NetEntitySystem {
    pub fn new(aspect: Aspect<Components>, my_id: PlayerId, entity_types: &EntityTypes)
               -> NetEntitySystem {
        NetEntitySystem {
            aspect: CachedAspect::new(aspect),
            entity_types: entity_types.clone(),
            my_id: my_id,
        }
    }

    /// Replicates entities created on the server side via `catch_server::entities::build_net`
    fn create_entity(&mut self,
                     entity_id: EntityId,
                     entity_type_id: EntityTypeId,
                     owner: PlayerId,
                     data: &mut DataHelper<Components, Services>) -> ecs::Entity {
        debug!("creating entity {} of type {} with owner {}", entity_id, entity_type_id, owner);

        assert!(self.entity_types.get(entity_type_id as usize).is_some(),
                "unknown net entity type id");

        let entity = data.create_entity(|entity: BuildData<Components>, data: &mut Components| {
            data.net_entity.add(&entity, NetEntity {
                id: entity_id,
                type_id: entity_type_id,
                owner: owner,
            });

            // Create net components of the entity type locally
            for net_component in &self.entity_types[entity_type_id as usize].1
                                      .component_types {
                NetComponents::add_component(*net_component, entity, data);

                // Add interpolation state components for certain net component types
                match *net_component {
                    ComponentType::Position => {
                        data.interp_position.add(&entity, InterpolationState::none());
                    }
                    ComponentType::Orientation => {
                        data.interp_orientation.add(&entity, InterpolationState::none());
                    }
                    _ => ()
                };
            }

            // If we own the object, potentially add some more net components
            if self.my_id == owner {
                for net_component in &self.entity_types[entity_type_id as usize].1
                                          .owner_component_types {
                    NetComponents::add_component(*net_component, entity, data);
                }
            }

            // Add other shared components
            let type_name = &self.entity_types[entity_type_id as usize].0;
            shared::entities::build_shared(type_name, entity, data);

            // Add client-side components to the entity (e.g. for drawing)
            entities::build_client(type_name, entity, data);
        });

        data.services.net_entities.on_build(entity_id, entity_type_id, owner, entity);

        entity
    }

    fn remove_entity(&mut self,
                     entity_id: EntityId,
                     data: &mut DataHelper<Components, Services>) {
        debug!("removing entity with id {}", entity_id);
        let entity = data.services.net_entities[entity_id];
        data.services.net_entities.on_remove(entity_id);
        data.remove_entity(entity);
    }

    /// Creates entities that are new in a tick and removes those that are to be removed
    pub fn process_entity_events(&mut self, tick: &Tick,
                                 data: &mut DataHelper<Components, Services>) {
        for event in tick.events.iter() {
            match *event {
                GameEvent::CreateEntity(entity_id, entity_type_id, owner) => {
                    self.create_entity(entity_id, entity_type_id, owner, data);
                }
                GameEvent::RemoveEntity(entity_id) => {
                    self.remove_entity(entity_id, data); 
                }
                _ => {}
            }
        }
    }

    /// Loads net state from the given `Tick` into our entities
    pub fn load_tick_state(&mut self, tick: &Tick, c: &mut DataHelper<Components, Services>) {
        for &(net_id, ref net_components) in tick.state.entities.iter() {
            // TODO: Can we avoid these two lookups?
            let entity = c.services.net_entities[net_id];
            c.with_entity_data(&entity, |e, c| {
                let entity_type = &self.entity_types[c.net_entity[e].type_id as usize].1;

                if self.my_id == c.net_entity[e].owner {
                    let it = entity_type.component_types.iter()
                                        .chain(entity_type.owner_component_types.iter())
                                        .map(|c| *c);
                    net_components.load_to_entity(it, e, c);
                } else {
                    let it = entity_type.component_types.iter().map(|c| *c);
                    net_components.load_to_entity(it, e, c);
                };
            });
        }
    }

    /// Loads state that is to be interpolated between `tick_a` and `tick_b`
    pub fn load_interp_tick_state(&mut self, tick_a: &Tick, tick_b: &Tick,
                                  c: &mut DataHelper<Components, Services>) {
        for (net_id, pair) in tick_a.state.iter_pairs(&tick_b.state) {
            match pair {
                EntityPair::Both(state_a, state_b) => {
                    // TODO: Can we avoid these two lookups?
                    let entity = c.services.net_entities[net_id];
                    c.with_entity_data(&entity, |e, c| {
                        let entity_type = &self.entity_types[c.net_entity[e].type_id as usize].1;

                        for component_type in &entity_type.component_types {
                            // Don't interpolate into forced components
                            let mut forced = false;
                            for &(id, forced_component) in &tick_b.state.forced_components {
                                if net_id == id && *component_type == forced_component {
                                    forced = true;
                                }
                            }

                            match *component_type { 
                                ComponentType::Position => {
                                    c.interp_position[e] = if !forced {
                                        InterpolationState::some(
                                            state_a.position.clone().unwrap(),
                                            state_b.position.clone().unwrap())
                                    } else {
                                        InterpolationState::none()
                                    }
                                }
                                ComponentType::Orientation => {
                                    c.interp_orientation[e] = if !forced {
                                        InterpolationState::some(
                                            state_a.orientation.clone().unwrap(),
                                            state_b.orientation.clone().unwrap())
                                    } else {
                                        InterpolationState::none()
                                    }
                                }
                                _ => {}
                            };
                        }
                    });
                }
                _ => {}, 
            }
        }
    }
}

impl_cached_system!(Components, Services, NetEntitySystem, aspect);

impl Process for NetEntitySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

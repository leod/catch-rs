use std::collections::HashMap;

use ecs;
use ecs::{Aspect, System, DataHelper, BuildData, Process};

use shared;
use shared::util::CachedAspect;
use shared::net::ComponentType;
use shared::components::StateComponent;
use shared::{Tick, GameEvent, PlayerId, EntityId, EntityTypes, EntityTypeId};

use components;
use components::{Components, NetEntity, InterpolationState};
use entities;
use services::Services;

pub struct NetEntitySystem {
    aspect: CachedAspect<Components>,

    entity_types: EntityTypes,

    // List of trait objects for each net component type for loading and storing the state
    component_type_traits: components::ComponentTypeTraits<Components>,

    // Map from network entity ids to the local component system's entity id
    entities: HashMap<EntityId, ecs::Entity>,

    // Store for each player the current entity
    player_entities: HashMap<PlayerId, ecs::Entity>,

    my_id: PlayerId,
}

impl NetEntitySystem {
    pub fn new(aspect: Aspect<Components>, my_id: PlayerId, entity_types: &EntityTypes)
               -> NetEntitySystem {
        NetEntitySystem {
            aspect: CachedAspect::new(aspect),
            entity_types: entity_types.clone(),
            component_type_traits: components::component_type_traits(),
            entities: HashMap::new(),
            player_entities: HashMap::new(),
            my_id: my_id,
        }
    }

    pub fn get_my_player_entity(&self) -> Option<ecs::Entity> {
        self.get_player_entity(self.my_id)
    }

    pub fn get_player_entity(&self, player_id: PlayerId) -> Option<ecs::Entity> {
        self.player_entities.get(&player_id).map(|e| *e)
    }

    pub fn get_entity(&self, id: EntityId) -> Option<ecs::Entity> {
        self.entities.get(&id).map(|entity| *entity)
    }

    /// Replicates entities created on the server side via `catch_server::entities::build_net`
    fn create_entity(&mut self,
                     entity_id: EntityId,
                     entity_type_id: EntityTypeId,
                     owner: PlayerId,
                     data: &mut DataHelper<Components, Services>) -> ecs::Entity {
        debug!("creating entity {} of type {} with owner {}", entity_id, entity_type_id, owner);

        assert!(self.entities.get(&entity_id).is_none(), "already have a net entity with that id");
        assert!(self.entity_types.get(entity_type_id as usize).is_some(),
                "Unknown net entity type id");

        let entity = data.create_entity(|entity: BuildData<Components>, data: &mut Components| {
            data.net_entity.add(&entity, NetEntity {
                id: entity_id,
                type_id: entity_type_id,
                owner: owner,
            });

            // Create net components of the entity type locally
            for net_component in &self.entity_types[entity_type_id as usize].1
                                      .component_types {
                self.component_type_traits[*net_component].add(entity, data);

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
                    self.component_type_traits[*net_component].add(entity, data);
                }
            }

            // Add other shared components
            let type_name = &self.entity_types[entity_type_id as usize].0;
            shared::entities::build_shared(type_name, entity, data);

            // Add client-side components to the entity (e.g. for drawing)
            entities::build_client(type_name, entity, data);
        });

        // HACK: detection of player entities
        if self.entity_types[entity_type_id as usize].0 == "player" {
            assert!(self.player_entities.get(&owner).is_none());
            self.player_entities.insert(owner, entity);
        }

        self.entities.insert(entity_id, entity);
        entity
    }

    fn remove_entity(&mut self,
                     entity_id: EntityId,
                     data: &mut DataHelper<Components, Services>) {

        if self.entities.get(&entity_id).is_some() {
            debug!("removing entity with id {}", entity_id);

            let entity = self.entities[&entity_id].clone();

            // Clear references to the entity
            {
                let mut remove_id = None;

                for (player_id, player_entity) in self.player_entities.iter_mut() {
                    if *player_entity == entity {
                        assert!(remove_id.is_none()); 
                        remove_id = Some(*player_id);
                    }
                }

                if let Some(remove_id) = remove_id {
                    self.player_entities.remove(&remove_id);
                }
            }

            data.remove_entity(entity);
            self.entities.remove(&entity_id);
        } else {
            panic!("unkown net entity id: {}", entity_id);
        }
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
        // TODO: If these tick methods turn out to be a bottleneck, I'll need to find a better
        // representation for TickState than a bunch of HashMaps

        // Only load state for those entities that we already have
        for e in self.aspect.iter() {
            let net_id = c.net_entity[e].id;
            let entity_type = &self.entity_types[c.net_entity[e].type_id as usize].1;

            for component_type in &entity_type.component_types {
                self.component_type_traits[*component_type]
                    .load(e, net_id, &tick.state, c);
            }
            if self.my_id == c.net_entity[e].owner {
                for component_type in &entity_type.owner_component_types {
                    self.component_type_traits[*component_type]
                        .load(e, net_id, &tick.state, c);
                }
            }
        }
    }

    /// Loads state that is to be interpolated between `tick_a` and `tick_b`
    pub fn load_interp_tick_state(&mut self, tick_a: &Tick, tick_b: &Tick,
                                  c: &mut DataHelper<Components, Services>) {
        // TODO: This would be more efficient if we stepped through tick_a's and tick_b's entity
        // lists simultaneously (=> O(n))

        // We only load state for those entities that we already have
        for e in self.aspect.iter() {
            let net_id = c.net_entity[e].id;
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
                        c.interp_position[e] = 
                            match (forced,
                                   tick_a.state.position.get(&net_id),
                                   tick_b.state.position.get(&net_id)) {
                                (false, Some(a), Some(b)) =>
                                    InterpolationState::some(a.clone(), b.clone()),
                                _ =>
                                    InterpolationState::none() 
                            };
                    }
                    ComponentType::Orientation => {
                        c.interp_orientation[e] = 
                            match (forced,
                                   tick_a.state.orientation.get(&net_id),
                                   tick_b.state.orientation.get(&net_id)) {
                                (false, Some(a), Some(b)) =>
                                    InterpolationState::some(a.clone(), b.clone()),
                                _ =>
                                    InterpolationState::none() 
                            };
                    }
                    _ => {}
                };
            }
        }
    }
}

impl_cached_system!(Components, Services, NetEntitySystem, aspect);

impl Process for NetEntitySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

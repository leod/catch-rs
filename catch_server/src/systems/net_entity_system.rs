use std::collections::HashMap;

use ecs;
use ecs::{Aspect, Process, System, EntityData, DataHelper};

use shared;
use shared::components::StateComponent;
use shared::{EntityId, EntityTypes, PlayerId, GameEvent, TickState};

use components;
use components::{Components, ComponentTypeTraits};
use services::Services;

pub struct NetEntitySystem {
    aspect: Aspect<Components>,

    entity_types: EntityTypes,
    component_type_traits: ComponentTypeTraits<Components>,

    entities: HashMap<EntityId, ecs::Entity>,
}

impl NetEntitySystem {
    pub fn new(aspect: Aspect<Components>) -> NetEntitySystem {
        NetEntitySystem {
            aspect: aspect,
            entity_types: shared::entities::all_entity_types(),
            component_type_traits: components::component_type_traits(),
            entities: HashMap::new(),
        }
    }

    /// Remove all entities owned by `player_id`
    pub fn remove_player_entities(&mut self,
                                  player_id: PlayerId,
                                  data: &mut DataHelper<Components, Services>) {
        for (_, entity) in self.entities.iter() {
            let owner = data.with_entity_data(entity, |e, c| {
                c.net_entity[e].owner
            }).unwrap();

            if owner == player_id {
                data.remove_entity(entity.clone());
            }
        }
    }

    pub fn get_entity(&self, net_entity_id: EntityId) -> ecs::Entity {
        self.entities[&net_entity_id]
    }

    /// Queue up CreateEntity events for a freshly connected player
    pub fn replicate_entities(&self, player_id: PlayerId,
                              data: &mut DataHelper<Components, Services>) {
        for (net_entity_id, entity) in self.entities.iter() {
            let (entity_type_id, owner) = data.with_entity_data(entity, |e, c| {
                (c.net_entity[e].type_id, c.net_entity[e].owner)
            }).unwrap();
            
            data.services.add_player_event(player_id,
                GameEvent::CreateEntity(*net_entity_id, entity_type_id, owner));
        }
    }

    /// Write the current state into a TickState
    pub fn store_in_tick_state(&self, player_id: PlayerId, tick_state: &mut TickState,
                               data: &mut DataHelper<Components, Services>) {
        let mut forced_components = Vec::new();

        for (net_id, entity) in self.entities.iter() {
            data.with_entity_data(entity, |e, c| {
                let &(_, ref entity_type) =
                    &self.entity_types[c.net_entity[e].type_id as usize];

                for component_type in &entity_type.component_types {
                    self.component_type_traits[*component_type]
                        .store(e, *net_id, tick_state, c);
                }

                // Some components only need to be sent to the owner of the net entity
                if player_id == c.net_entity[e].owner {
                    for component_type in &entity_type.owner_component_types {
                        self.component_type_traits[*component_type]
                            .store(e, *net_id, tick_state, c);
                    }
                }

                // Mark forced components
                for forced_component in &c.server_net_entity[e].forced_components {
                    forced_components.push((*net_id, *forced_component));
                }
                c.server_net_entity[e].forced_components = Vec::new();
            });
        }

        tick_state.forced_components = forced_components;
    }
}

impl System for NetEntitySystem {
    type Components = Components;
    type Services = Services;

    fn activated(&mut self, entity: &EntityData<Components>, components: &Components,
                 _: &mut Services) {
        if self.aspect.check(entity, components) {
            let net_entity = &components.net_entity[*entity];

            debug!("registering net entity {} of type {} with owner {}",
                   net_entity.id, net_entity.type_id, net_entity.owner);

            assert!(self.entities.get(&net_entity.id).is_none(),
                    "already have a net entity with that id");

            self.entities.insert(net_entity.id, ***entity);
        }
    }

    fn reactivated(&mut self, _: &EntityData<Components>, _: &Components, _: &mut Services) {
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, components: &Components,
                   _: &mut Services) {
        if self.aspect.check(entity, components) {
            let net_entity = &components.net_entity[*entity];

            if self.entities.get(&net_entity.id).is_some() {
                self.entities.remove(&net_entity.id);

                debug!("unregistering entity with id {}", net_entity.id);
            } else {
                panic!("unkown net entity id: {}", net_entity.id)
            }

        }
    }
}

impl Process for NetEntitySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

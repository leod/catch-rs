use std::collections::HashMap;

use ecs;
use ecs::{Process, System, BuildData, EntityData, EntityIter, DataHelper};

use shared;
use shared::net;
use shared::components::StateComponent;
use shared::{EntityId, EntityTypeId, EntityTypes, PlayerId, GameEvent, TickState};

use components;
use components::{Components, NetEntity, ServerNetEntity};
use entities;
use services::Services;

pub struct NetEntitySystem {
    id_counter: EntityId,
    entities: HashMap<EntityId, ecs::Entity>,
    entity_types: EntityTypes,
    component_type_traits: components::ComponentTypeTraits<Components>,
}

impl NetEntitySystem {
    pub fn new() -> NetEntitySystem {
        NetEntitySystem {
            id_counter: 0,
            entities: HashMap::new(),
            entity_types: shared::entities::all_entity_types(),
            component_type_traits: components::component_type_traits(),
        }
    }

    pub fn type_id(&self, type_name: String) -> EntityTypeId {
        self.entity_types.iter()
            .enumerate()
            .find(|&(_, &(ref name, _))| name == &type_name)
            .unwrap()
            .0 as EntityTypeId
    }

    fn component_type_trait(&self, component_type: net::ComponentType)
                            -> &Box<StateComponent<Components>> {
        &self.component_type_traits[component_type as usize]
    }

    pub fn create_entity(&mut self,
                         entity_type_id: EntityTypeId,
                         player_id: PlayerId,
                         data: &mut DataHelper<Components, Services>)
                         -> (EntityId, ecs::Entity) {
        self.id_counter += 1;

        println!("Creating entity {} of type {} with owner {}",
                 self.id_counter, entity_type_id, player_id);

        // Tell the clients about it
        data.services.add_event(
            &GameEvent::CreateEntity(self.id_counter, entity_type_id, player_id));

        assert!(self.entities.get(&self.id_counter).is_none(),
                "Already have a net entity with that id");
        assert!(self.entity_types.get(entity_type_id as usize).is_some(),
                "Unknown net entity type id");

        let entity = data.create_entity(|entity: BuildData<Components>,
                                         data: &mut Components| {
            data.net_entity.add(&entity, NetEntity {
                id: self.id_counter,
                type_id: entity_type_id,
                owner: player_id,
            });
            data.server_net_entity.add(&entity, ServerNetEntity::default());

            // Add net component to the entity using its type
            for net_component in &self.entity_types[entity_type_id as usize].1
                                      .component_types {
                self.component_type_trait(*net_component).add(entity, data);
            }
            for net_component in &self.entity_types[entity_type_id as usize].1
                                      .owner_component_types {
                self.component_type_trait(*net_component).add(entity, data);
            }

            // Add server-side only components
            let type_name = &self.entity_types[entity_type_id as usize].0;
            entities::build(type_name, entity, data);
        });

        self.entities.insert(self.id_counter, entity);
        (self.id_counter, entity)
    }

    pub fn remove_entity(&mut self,
                         entity_id: EntityId,
                         data: &mut DataHelper<Components, Services>) {
        if self.entities.get(&entity_id).is_some() {
            data.remove_entity(self.entities[&entity_id]);
            self.entities.remove(&entity_id);

            println!("Removing entity with id {}", entity_id);

            // Tell the clients about it
            data.services.add_event(&GameEvent::RemoveEntity(entity_id));
        } else {
            panic!("Unkown net entity id: {}", entity_id)
        }
    }

    /// Remove all entities owned by `player_id`
    pub fn remove_player_entities(&mut self,
                                  player_id: PlayerId,
                                  data: &mut DataHelper<Components, Services>) {
        let mut remove = Vec::new();
        for (net_id, entity) in self.entities.iter() {
            let owner = data.with_entity_data(entity, |e, c| {
                c.net_entity[e].owner
            }).unwrap();

            if owner == player_id {
                remove.push(*net_id);
            }
        }

        for net_id in remove.iter() {
            self.remove_entity(*net_id, data);
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
                let &(_, ref entity_type) = &self.entity_types[c.net_entity[e].type_id as usize];

                for component_type in &entity_type.component_types {
                    self.component_type_trait(*component_type)
                        .store(e, *net_id, tick_state, c);
                }

                // Some components only need to be sent to the owner of the net entity
                if player_id == c.net_entity[e].owner {
                    for component_type in &entity_type.owner_component_types {
                        self.component_type_trait(*component_type)
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
}

impl Process for NetEntitySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

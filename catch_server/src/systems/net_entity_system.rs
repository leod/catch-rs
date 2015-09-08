use std::collections::HashMap;

use ecs;
use ecs::{System, BuildData, EntityIter, DataHelper};
use ecs::system::EntityProcess;

use shared::net;
use shared::player::PlayerId;
use shared::event::GameEvent;
use components;
use components::{Components, NetEntity};
use services::Services;

pub struct NetEntitySystem {
    id_counter: net::EntityId,
    entities: HashMap<net::EntityId, ecs::Entity>,
    entity_types: net::EntityTypes,
    component_type_traits: components::ComponentTypeTraits<Components>,
}

impl NetEntitySystem {
    pub fn new() -> NetEntitySystem {
        NetEntitySystem {
            id_counter: 0,
            entities: HashMap::new(),
            entity_types: net::all_entity_types(),
            component_type_traits: components::component_type_traits(),
        }
    }

    pub fn type_id(&self, type_name: String) -> net::EntityTypeId {
        self.entity_types.iter()
            .enumerate()
            .find(|&(_, &(ref name, _))| name == &type_name)
            .unwrap()
            .0 as net::EntityTypeId
    }

    fn component_type_trait(&self, component_type: net::ComponentType) -> &Box<net::StateComponent<Components>> {
        &self.component_type_traits[component_type as usize]
    }

    pub fn create_entity(&mut self,
                         entity_type_id: net::EntityTypeId,
                         player_id: PlayerId,
                         data: &mut DataHelper<Components, Services>) -> (net::EntityId, ecs::Entity) {
        self.id_counter += 1;

        println!("Creating entity {} of type {} with owner {}",
                 self.id_counter, entity_type_id, player_id);

        // Tell the clients about it
        data.services.next_tick.as_mut().unwrap().events.push(
            GameEvent::CreateEntity(self.id_counter, entity_type_id, player_id));

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

            for net_component in &self.entity_types[entity_type_id as usize].1.component_types {
                self.component_type_trait(*net_component).add(entity, data);
            }

            let type_name = self.entity_types[entity_type_id as usize].0.clone();

            // TODO: probably don't wanna keep this hardcoded here
            if &type_name == "player" {
            } else {
                panic!("Unknown net entity type: {}", type_name);
            }
        });

        self.entities.insert(self.id_counter, entity);
        (self.id_counter, entity)
    }

    fn remove_entity(&mut self,
                     entity_id: net::EntityId,
                     data: &mut DataHelper<Components, Services>) {
        if self.entities.get(&entity_id).is_some() {
            data.remove_entity(self.entities[&entity_id]);
            self.entities.remove(&entity_id);
            // TODO: consequences
        } else {
            panic!("Unkown net entity id: {}", entity_id)
        }
    }

    pub fn get_entity(&self, net_entity_id: net::EntityId) -> ecs::Entity {
        self.entities[&net_entity_id]
    }

    // Queue up CreateEntity events for a freshly connected player
    pub fn replicate_entities(&self, player_id: PlayerId, data: &mut DataHelper<Components, Services>) {
        for (net_entity_id, entity) in self.entities.iter() {
            let (entity_type_id, owner) = data.with_entity_data(entity, |e, c| {
                (c.net_entity[e].type_id, c.net_entity[e].owner)
            }).unwrap();
            
            data.services.add_player_event(player_id,
                GameEvent::CreateEntity(*net_entity_id, entity_type_id, owner));
        }
    }
}

impl System for NetEntitySystem {
    type Components = Components;
    type Services = Services;
}

// Once the tick has been processed, NetEntitySystem writes the current tick component state into the global Tick
impl EntityProcess for NetEntitySystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        for e in entities {
            let &(_, ref entity_type) = &self.entity_types[data.net_entity[e].type_id as usize];
            let net_id = data.net_entity[e].id;

            for component_type in &entity_type.component_types {
                let next_net_state = &mut data.services.next_tick.as_mut().unwrap().net_state;

                self.component_type_trait(*component_type)
                    .write(e, net_id, next_net_state, &data.components);
            }
        }
    }
}

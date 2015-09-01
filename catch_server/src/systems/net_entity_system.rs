use std::collections::HashMap;

use ecs;
use ecs::{System, EntityData, EntityIter, DataHelper};
use ecs::system::EntityProcess;

use shared::net;
use components::Components;
use services::Services;

pub struct NetEntitySystem {
    entities: HashMap<net::EntityId, ecs::entity::Id>,
    entity_types: net::EntityTypes,
}

impl NetEntitySystem {
    pub fn new() -> NetEntitySystem {
        NetEntitySystem {
            entities: HashMap::new(),
            entity_types: net::all_entity_types(),
        }
    }
}

impl System for NetEntitySystem {
    type Components = Components;
    type Services = Services;

    fn activated(&mut self, entity: &EntityData<Components>, c: &Components, services: &mut Services) {
        if c.net_entity.has(entity) {
            let net_entity = &c.net_entity[*entity];

            assert!(self.entities.get(&net_entity.id).is_none(),
                    "Net entity ID already in use");
            assert!(self.entity_types.get(net_entity.type_id as usize).is_some(),
                    "Net entity with invalid net entity type ID was created");

            self.entities.insert(net_entity.id, entity.id());
        }
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, c: &Components, services: &mut Services) {
        if c.net_entity.has(entity) {
            let net_entity = &c.net_entity[*entity];

            assert!(self.entities.get(&net_entity.id).is_some(),
                    "Net entity with invalid net entity ID");

            self.entities.remove(&net_entity.id);
        }
    }
}

// Once the tick has been processed, NetEntitySystem writes the current tick component state into the global Tick
impl EntityProcess for NetEntitySystem {
    fn process(&mut self, entities: EntityIter<Components>, data: &mut DataHelper<Components, Services>) {
        for e in entities {
            let &(_, ref entity_type) = &self.entity_types[data.net_entity[e].type_id as usize];
            let net_id = data.net_entity[e].id;

            for component_type in &entity_type.component_types {
                match *component_type {
                    net::ComponentType::Position => {
                        let position = data.position[e].clone();
                        data.services.next_tick.as_mut().unwrap().net_state.position.insert(
                            net_id, position);
                    }
                };
            }
        }
    }
}

use std::iter::Iterator;
use std::collections::HashMap;

use ecs;
use ecs::{Aspect, Process, System, EntityData, DataHelper};

use shared;
use shared::net_components::NetComponents;
use shared::{EntityId, EntityTypes, PlayerId, GameEvent, TickState};
use shared::util::CachedAspect;

use entities;
use components::Components;
use services::Services;

pub struct NetEntitySystem {
    aspect: CachedAspect<Components>,
    entity_types: EntityTypes,
}

impl NetEntitySystem {
    pub fn new(aspect: Aspect<Components>) -> NetEntitySystem {
        NetEntitySystem {
            aspect: CachedAspect::new(aspect),
            entity_types: shared::entities::all_entity_types(),
        }
    }

    /// Remove all entities owned by `player_id`
    pub fn remove_player_entities(&mut self,
                                  player_id: PlayerId,
                                  data: &mut DataHelper<Components, Services>) {
        for entity in self.aspect.iter() {
            if data.net_entity[entity].owner == player_id {
                entities::remove_net(**entity, data);
            }
        }
    }

    /// Queue up CreateEntity events for a freshly connected player
    pub fn replicate_entities(&self, player_id: PlayerId,
                              data: &mut DataHelper<Components, Services>) {
        for entity in self.aspect.iter() {
            assert!(!data.server_net_entity[entity].removed);
            let event = GameEvent::CreateEntity(data.net_entity[entity].id,
                                                data.net_entity[entity].type_id,
                                                data.net_entity[entity].owner);
            data.services.add_player_event(player_id, &event);
        }
    }

    /// Write the current state into a TickState
    pub fn store_in_tick_state(&self, player_id: PlayerId, tick_state: &mut TickState,
                               c: &mut DataHelper<Components, Services>) {
        let mut forced_components = Vec::new();

        for e in self.aspect.iter() {
            let &(_, ref entity_type) =
                &self.entity_types[c.net_entity[e].type_id as usize];
            let net_id = c.net_entity[e].id;

            let net_components = 
                if player_id == c.net_entity[e].owner {
                    // Some components only need to be sent to the owner of the net entity
                    let it = entity_type.component_types.iter()
                                        .chain(entity_type.owner_component_types.iter())
                                        .map(|c| *c);
                    NetComponents::from_entity(it, e, c)
                } else {
                    let it = entity_type.component_types.iter().map(|c| *c);
                    NetComponents::from_entity(it, e, c)
                };

            tick_state.entities.push((net_id, net_components));

            // Mark forced components
            for forced_component in &c.server_net_entity[e].forced_components {
                forced_components.push((net_id, *forced_component));
            }
            c.server_net_entity[e].forced_components = Vec::new();
        }
        tick_state.sort();

        tick_state.forced_components = forced_components;
    }
}

impl System for NetEntitySystem {
    type Components = Components;
    type Services = Services;

    fn activated(&mut self, entity: &EntityData<Components>, components: &Components,
                 _: &mut Services) {
        self.aspect.activated(entity, components);
    }

    fn reactivated(&mut self, entity: &EntityData<Components>, components: &Components,
                   _: &mut Services) {
        self.aspect.reactivated(entity, components);
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, components: &Components,
                   _: &mut Services) {
        self.aspect.deactivated(entity, components);
    }
}

impl Process for NetEntitySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

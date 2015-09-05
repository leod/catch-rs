use std::collections::HashMap;

use ecs;
use ecs::{System, EntityData, EntityIter, DataHelper, BuildData, Process};
use ecs::system::EntityProcess;

use shared::net;
use shared::tick::Tick;
use shared::event::GameEvent;
use shared::player::PlayerId;
use components::*;
use services::Services;

pub struct NetEntitySystem {
    entities: HashMap<net::EntityId, ecs::Entity>,
    entity_types: net::EntityTypes,
}

impl NetEntitySystem {
    pub fn new(entity_types: &net::EntityTypes) -> NetEntitySystem {
        NetEntitySystem {
            entities: HashMap::new(),
            entity_types: entity_types.clone(),
        }
    }
    
    fn create_entity(&mut self,
                     entity_id: net::EntityId,
                     entity_type_id: net::EntityTypeId,
                     player_id: PlayerId,
                     data: &mut DataHelper<Components, Services>) -> ecs::Entity {
        println!("Creating entity {} of type {} with owner {}",
                 entity_id, entity_type_id, player_id);

        assert!(self.entities.get(&entity_id).is_none(),
                "Already have a net entity with that id");
        assert!(self.entity_types.get(entity_type_id as usize).is_some(),
                "Unknown net entity type id");

        let entity = data.create_entity(|entity: BuildData<Components>, data: &mut Components| {
            data.net_entity.add(&entity, NetEntity {
                id: entity_id,
                type_id: entity_type_id,
                owner: player_id,
            });

            for net_component in &self.entity_types[entity_type_id as usize].1.component_types {
                match *net_component {
                    net::ComponentType::Position => {
                        data.position.add(&entity, Position::default());
                        data.interp_state_position.add(&entity, InterpolationState::none());
                    }
                    net::ComponentType::Orientation => {
                        data.orientation.add(&entity, Orientation::default());
                        data.interp_state_orientation.add(&entity, InterpolationState::none());
                    }
                    net::ComponentType::PlayerState => {
                        data.player_state.add(&entity, PlayerState::default());
                    }
                };
            }

            let type_name = self.entity_types[entity_type_id as usize].0.clone();

            // TODO: probably don't wanna keep this hardcoded here
            if &type_name == "player" {
                data.draw_player.add(&entity, DrawPlayer);
            } else {
                panic!("Unknown net entity type: {}", type_name);
            }
        });

        self.entities.insert(entity_id, entity);
        entity
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

    // Creates entities that are new in a tick and removes those that are to be removed
    pub fn process_entity_events(&mut self, tick: &Tick, data: &mut DataHelper<Components, Services>) {
        for event in tick.events.iter() {
            match *event {
                GameEvent::CreateEntity(entity_id, entity_type_id, player_id) => {
                    self.create_entity(entity_id, entity_type_id, player_id, data);
                }
                GameEvent::RemoveEntity(entity_id) => {
                    self.remove_entity(entity_id, data); 
                }
                _ => {}
            }
        }
    }

    pub fn load_tick_state(&mut self, tick: &Tick, data: &mut DataHelper<Components, Services>) {
        // TODO: If these tick methods turn out to be a bottleneck, I'll need to find a better
        // representation for TickState than a bunch of HashMaps

        // Only load state for those entities that we already have
        for (net_entity_id, entity) in self.entities.iter() {
            data.with_entity_data(entity, |e, c| {
                let entity_type = &self.entity_types[c.net_entity[e].type_id as usize].1;

                for component_type in &entity_type.component_types {
                    match *component_type { 
                        net::ComponentType::Position => {
                            if let Some(position) = tick.net_state.position.get(&net_entity_id) {
                                c.position[e] = position.clone();
                            }
                        }
                        net::ComponentType::Orientation => {
                            if let Some(orientation) = tick.net_state.orientation.get(&net_entity_id) {
                                c.orientation[e] = orientation.clone();
                            }
                        }
                        net::ComponentType::PlayerState => {
                            if let Some(player_state) = tick.net_state.player_state.get(&net_entity_id) {
                                c.player_state[e] = player_state.clone();
                            }
                        }
                    };
                }
            });
        }
    }

    pub fn load_interp_tick_state(&mut self, tick_a: &Tick, tick_b: &Tick,
                                  data: &mut DataHelper<Components, Services>) {
        // TODO: This would be more efficient if we stepped through tick_a's and tick_b's entity
        // lists simultaneously (=> O(n))

        // Only load state for those entities that we already have
        for (net_entity_id, entity) in self.entities.iter() {
            data.with_entity_data(entity, |e, c| {
                let entity_type = &self.entity_types[c.net_entity[e].type_id as usize].1;

                for component_type in &entity_type.component_types {
                    match *component_type { 
                        net::ComponentType::Position => {
                            c.interp_state_position[e] = 
                                match (tick_a.net_state.position.get(&net_entity_id),
                                       tick_b.net_state.position.get(&net_entity_id)) {
                                    (Some(a), Some(b)) =>
                                        InterpolationState::some(a.clone(), b.clone()),
                                    _ =>
                                        InterpolationState::none() 
                                };
                        }
                        net::ComponentType::Orientation => {
                            c.interp_state_orientation[e] = 
                                match (tick_a.net_state.orientation.get(&net_entity_id),
                                       tick_b.net_state.orientation.get(&net_entity_id)) {
                                    (Some(a), Some(b)) =>
                                        InterpolationState::some(a.clone(), b.clone()),
                                    _ =>
                                        InterpolationState::none() 
                                };
                        }
                        _ => {}
                    };
                }
            });
        }
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

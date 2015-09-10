use std::collections::HashMap;

use ecs;
use ecs::{System, DataHelper, BuildData, Process};

use shared::net;
use shared::tick::Tick;
use shared::event::GameEvent;
use shared::player::PlayerId;
use components;
use components::{Components, NetEntity, InterpolationState, DrawPlayer};
use services::Services;

pub struct NetEntitySystem {
    entity_types: net::EntityTypes,
    component_type_traits: components::ComponentTypeTraits<Components>,

    entities: HashMap<net::EntityId, ecs::Entity>,

    my_id: PlayerId,
    my_player_entity_id: Option<net::EntityId>,
}

impl NetEntitySystem {
    pub fn new(my_id: PlayerId, entity_types: &net::EntityTypes) -> NetEntitySystem {
        NetEntitySystem {
            entity_types: entity_types.clone(),
            component_type_traits: components::component_type_traits(),
            entities: HashMap::new(),
            my_id: my_id,
            my_player_entity_id: None,
        }
    }

    fn component_type_trait(&self, component_type: net::ComponentType) -> &Box<net::StateComponent<Components>> {
        &self.component_type_traits[component_type as usize]
    }

    pub fn my_player_entity_id(&self) -> Option<net::EntityId> {
        self.my_player_entity_id
    }

    pub fn get_entity(&self, id: net::EntityId) -> Option<ecs::Entity> {
        self.entities.get(&id).map(|entity| *entity)
    }
    
    fn create_entity(&mut self,
                     entity_id: net::EntityId,
                     entity_type_id: net::EntityTypeId,
                     owner: PlayerId,
                     data: &mut DataHelper<Components, Services>) -> ecs::Entity {
        println!("Creating entity {} of type {} with owner {}",
                 entity_id, entity_type_id, owner);

        assert!(self.entities.get(&entity_id).is_none(),
                "Already have a net entity with that id");
        assert!(self.entity_types.get(entity_type_id as usize).is_some(),
                "Unknown net entity type id");

        let entity = data.create_entity(|entity: BuildData<Components>, data: &mut Components| {
            data.net_entity.add(&entity, NetEntity {
                id: entity_id,
                type_id: entity_type_id,
                owner: owner,
            });

            for net_component in &self.entity_types[entity_type_id as usize].1.component_types {
                self.component_type_trait(*net_component).add(entity, data);

                // Add components for interpolation state for certain net component types
                match *net_component {
                    net::ComponentType::Position => {
                        data.interp_position.add(&entity, InterpolationState::none());
                    }
                    net::ComponentType::Orientation => {
                        data.interp_orientation.add(&entity, InterpolationState::none());
                    }
                    _ => ()
                };
            }

            let type_name = self.entity_types[entity_type_id as usize].0.clone();

            // TODO: probably don't wanna keep this hardcoded here
            if &type_name == "player" {
                data.draw_player.add(&entity, DrawPlayer::new());
            } else {
                panic!("Unknown net entity type: {}", type_name);
            }
        });

        // TODO: detection of player entities
        if owner == self.my_id {
            self.my_player_entity_id = Some(entity_id);
        }

        self.entities.insert(entity_id, entity);
        entity
    }

    fn remove_entity(&mut self,
                     entity_id: net::EntityId,
                     data: &mut DataHelper<Components, Services>) {
        if self.my_player_entity_id == Some(entity_id) {
            self.my_player_entity_id = None;
        }

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

    pub fn load_tick_state(&mut self, tick: &Tick, data: &mut DataHelper<Components, Services>) {
        // TODO: If these tick methods turn out to be a bottleneck, I'll need to find a better
        // representation for TickState than a bunch of HashMaps

        // Only load state for those entities that we already have
        for (net_entity_id, entity) in self.entities.iter() {
            data.with_entity_data(entity, |e, c| {
                let entity_type = &self.entity_types[c.net_entity[e].type_id as usize].1;

                for component_type in &entity_type.component_types {
                    self.component_type_trait(*component_type)
                        .read(e, *net_entity_id, &tick.net_state, c);
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
                            c.interp_position[e] = 
                                match (tick_a.net_state.position.get(&net_entity_id),
                                       tick_b.net_state.position.get(&net_entity_id)) {
                                    (Some(a), Some(b)) =>
                                        InterpolationState::some(a.clone(), b.clone()),
                                    _ =>
                                        InterpolationState::none() 
                                };
                        }
                        net::ComponentType::Orientation => {
                            c.interp_orientation[e] = 
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

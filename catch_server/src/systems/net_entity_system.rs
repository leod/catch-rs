use std::collections::HashMap;

use ecs;
use ecs::{Aspect, Process, System, BuildData, EntityData, EntityIter, DataHelper};

use shared::net;
use shared::util::CachedAspect;
use shared::player::PlayerId;
use shared::event::GameEvent;
use shared::tick::NetState;
use components;
use components::{Components, NetEntity,
                 Shape, Interact,
                 ServerNetEntity, LinearVelocity,
                 BouncyEnemy};
use services::Services;

pub struct NetEntitySystem {
    aspect: CachedAspect<Components>,

    id_counter: net::EntityId,
    entities: HashMap<net::EntityId, ecs::Entity>,
    entity_types: net::EntityTypes,
    component_type_traits: components::ComponentTypeTraits<Components>,
}

impl NetEntitySystem {
    pub fn new(aspect: Aspect<Components>) -> NetEntitySystem {
        NetEntitySystem {
            aspect: CachedAspect::new(aspect),
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

            for net_component in &self.entity_types[entity_type_id as usize].1.component_types {
                self.component_type_trait(*net_component).add(entity, data);
            }

            let type_name = self.entity_types[entity_type_id as usize].0.clone();

            // TODO: probably don't wanna keep this hardcoded here
            if &type_name == "player" {
                data.shape.add(&entity, Shape::Circle { radius: 9.0 });
                data.interact.add(&entity, Interact);
            } else if &type_name == "bouncy_enemy" {
                data.shape.add(&entity, Shape::Circle { radius: 6.0 });
                data.interact.add(&entity, Interact);
                data.linear_velocity.add(&entity, LinearVelocity::default());
                data.bouncy_enemy.add(&entity, BouncyEnemy::default());
            } else {
                panic!("Unknown net entity type: {}", type_name);
            }
        });

        self.entities.insert(self.id_counter, entity);
        (self.id_counter, entity)
    }

    pub fn remove_entity(&mut self,
                         entity_id: net::EntityId,
                         data: &mut DataHelper<Components, Services>) {
        if self.entities.get(&entity_id).is_some() {
            data.remove_entity(self.entities[&entity_id]);
            self.entities.remove(&entity_id);

            // Tell the clients about it
            data.services.add_event(&GameEvent::RemoveEntity(entity_id));
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

    // Writes the current state into a NetState
    pub fn store_in_net_state(&self, player_id: PlayerId, net_state: &mut NetState, data: &mut DataHelper<Components, Services>) {
        let mut forced_components = Vec::new();

        for e in self.aspect.iter() {
            let &(_, ref entity_type) = &self.entity_types[data.net_entity[e].type_id as usize];
            let net_id = data.net_entity[e].id;

            for component_type in &entity_type.component_types {
                self.component_type_trait(*component_type)
                    .store(e, net_id, net_state, &data.components);
            }

            // Mark forced components
            for forced_component in &data.server_net_entity[e].forced_components {
                forced_components.push((net_id, *forced_component));
            }
            data.server_net_entity[e].forced_components = Vec::new();
        }

        net_state.forced_components = forced_components;
    }
}

impl System for NetEntitySystem {
    type Components = Components;
    type Services = Services;

    fn activated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.aspect.activated(entity, components);
    }

    fn reactivated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.aspect.reactivated(entity, components);
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.aspect.deactivated(entity, components);
    }
}

impl Process for NetEntitySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

use std::collections::{HashMap, hash_map};
use std::ops::Index;

use ecs::{self, ComponentManager, BuildData};

use super::{PlayerId, EntityId, EntityTypeId};
use components::{HasShape, Shape, HasWall, Wall, WallType, Projectile, HasProjectile};
use net_components::ComponentType;

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct EntityType {
    // Net components of this entity type, i.e. components whose states are sent to clients in
    // ticks
    pub component_types: Vec<ComponentType>,

    // Components that should be sent only to the owner of the object
    // Example: the full state of a player including cooldowns etc. is only needed by the owner
    pub owner_component_types: Vec<ComponentType>,
}

/// Adds shared components that are not synchronized over the net to an entity
pub fn build_shared<T: ComponentManager +
                       HasShape +
                       HasWall +
                       HasProjectile>
                   (type_name: &str,
                    entity: BuildData<T>,
                    data: &mut T) {
    if type_name == "player" {
        data.shape_mut().add(&entity, Shape::Circle { radius: 6.0 });
    } else if type_name == "bouncy_enemy" {
        data.shape_mut().add(&entity, Shape::Circle { radius: 10.0 });
    } else if type_name == "player_ball" {
        data.shape_mut().add(&entity, Shape::Circle { radius: 7.0 });
    } else if type_name == "item" {
        data.shape_mut().add(&entity, Shape::Square { size: 5.0 });
    } else if type_name == "item_spawn" {
    } else if type_name == "bullet" {
        data.shape_mut().add(&entity, Shape::Rect { width: 8.0, height: 4.0 });
        data.projectile_mut().add(&entity, Projectile::Bullet);
    } else if type_name == "frag" {
        data.shape_mut().add(&entity, Shape::Rect { width: 5.0, height: 5.0 });
        data.projectile_mut().add(&entity, Projectile::Frag(1.5));
    } else if type_name == "shrapnel" {
        data.projectile_mut().add(&entity, Projectile::Shrapnel);
    } else if type_name == "wall_wood" {
        data.wall_mut().add(&entity, Wall { 
            wall_type: WallType::Wood,
            width: 1.0,
        });
    } else if type_name == "wall_iron" {
        data.wall_mut().add(&entity, Wall { 
            wall_type: WallType::Iron,
            width: 1.0,
        });
    } else {
        panic!("unknown entity type: {}", type_name);
    }
}

/// Maps from net entity ids to the local ecs::Entity handles
pub struct NetEntities {
    entity_types: EntityTypes,
    entities: HashMap<EntityId, ecs::Entity>,

    // Stores each player's currently controlled entity
    player_entities: HashMap<PlayerId, ecs::Entity>,

}

impl Default for NetEntities {
    fn default() -> NetEntities {
        NetEntities {
            entity_types: all_entity_types(),
            entities: HashMap::new(),
            player_entities: HashMap::new(),
        }
    }
}

impl NetEntities {
    pub fn get(&self, id: EntityId) -> Option<ecs::Entity> {
        self.entities.get(&id).map(|e| e.clone())
    }

    pub fn get_player_entity(&self, player_id: PlayerId) -> Option<ecs::Entity> {
        self.player_entities.get(&player_id).map(|e| *e)
    }

    pub fn iter<'a>(&'a self) -> hash_map::Iter<'a, PlayerId, ecs::Entity> {
        self.entities.iter()
    }

    pub fn on_build(&mut self,
                    id: EntityId, type_id: EntityTypeId, owner: PlayerId,
                    entity: ecs::Entity) {
        assert!(self.entities.get(&id).is_none());
        self.entities.insert(id, entity);

        // HACK: detection of player entities
        if self.entity_types[type_id as usize].0 == "player" {
            assert!(self.player_entities.get(&owner).is_none());
            self.player_entities.insert(owner, entity);
        }
    }

    pub fn on_remove(&mut self, id: EntityId) {
        assert!(self.entities.get(&id).is_some());
        let entity = self.entities[&id].clone();
        self.entities.remove(&id);

        // Clear references to the entity
        {
            let mut remove_id = None;

            for (&player_id, &player_entity) in self.player_entities.iter() {
                if player_entity == entity {
                    assert!(remove_id.is_none()); 
                    remove_id = Some(player_id);
                }
            }

            if let Some(remove_id) = remove_id {
                self.player_entities.remove(&remove_id);
            }
        }
    }
}

impl Index<EntityId> for NetEntities {
    type Output = ecs::Entity;

    fn index(&self, id: EntityId) -> &ecs::Entity {
        &self.entities[&id]
    }
}

pub type EntityTypes = Vec<(String, EntityType)>;

pub fn all_entity_types() -> EntityTypes {
    vec![("player".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation,
                                    ComponentType::LinearVelocity,
                                    ComponentType::PlayerState],
              owner_component_types: vec![ComponentType::FullPlayerState],
         }),
         ("bouncy_enemy".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation],
              owner_component_types: vec![],
         }),
         ("player_ball".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation],
              owner_component_types: vec![],
         }),
         ("item_spawn".to_string(), EntityType {
              component_types: vec![ComponentType::Position],
              owner_component_types: vec![],
         }),
         ("item".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation],
              owner_component_types: vec![],
         }),
         ("bullet".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation],
              owner_component_types: vec![],
         }),
         ("frag".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation],
              owner_component_types: vec![],
         }),
         ("shrapnel".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation,
                                    ComponentType::Shape],
              owner_component_types: vec![],
         }),
         ("wall_wood".to_string(), EntityType {
              component_types: vec![ComponentType::WallPosition],
              owner_component_types: vec![],
         }),
         ("wall_iron".to_string(), EntityType {
              component_types: vec![ComponentType::WallPosition],
              owner_component_types: vec![],
         }),
        ]
}

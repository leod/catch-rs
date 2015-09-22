use ecs::{ComponentManager, BuildData};

use net::ComponentType;
use components::{HasShape, Shape};

#[derive(Debug, Clone, CerealData)]
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
                       HasShape>
                   (type_name: &str,
                    entity: BuildData<T>,
                    data: &mut T) {
    if type_name == "player" {
        data.shape_mut().add(&entity, Shape::Circle { radius: 7.5 });
    } else if type_name == "bouncy_enemy" {
        data.shape_mut().add(&entity, Shape::Circle { radius: 4.0 });
    } else if type_name == "item" {
        data.shape_mut().add(&entity, Shape::Square { size: 5.5 });
    } else if type_name == "item_spawn" {
    } else {
        panic!("Unknown entity type: {}", type_name);
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
         ("item_spawn".to_string(), EntityType {
              component_types: vec![ComponentType::Position],
              owner_component_types: vec![],
         }),
         ("item".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation],
              owner_component_types: vec![],
         }),
        ]
}

use net::ComponentType;

#[derive(Debug, Clone, CerealData)]
pub struct EntityType {
    // Net components of this entity type, i.e. components whose states are sent to clients in
    // ticks
    pub component_types: Vec<ComponentType>,

    // Components that should be sent only to the owner of the object
    // Example: the full state of a player including cooldowns etc. is only needed by the owner
    pub owner_component_types: Vec<ComponentType>,
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
         ("item".to_string(), EntityType {
              component_types: vec![ComponentType::Position,
                                    ComponentType::Orientation],
              owner_component_types: vec![],
         }),
        ]
}

use ecs::ComponentList;

use shared::net;
use shared::components::{HasPosition, HasOrientation,
                         HasLinearVelocity, HasPlayerState,
                         HasItemSpawn};
pub use shared::components::{NetEntity, Position,
                             Orientation, LinearVelocity,
                             PlayerState, ItemSpawn,
                             ComponentTypeTraits,
                             component_type_traits};

pub struct ServerNetEntity {
    // Components that should not be interpolated into the current tick
    pub forced_components: Vec<net::ComponentType>,
}

impl Default for ServerNetEntity {
    fn default() -> ServerNetEntity {
        ServerNetEntity {
            forced_components: Vec::new(),
        }
    }
}

impl ServerNetEntity {
    pub fn force(&mut self, component_type: net::ComponentType) {
        self.forced_components.push(component_type);
    }
}

pub struct BouncyEnemy; 

impl Default for BouncyEnemy {
    fn default() -> BouncyEnemy {
        BouncyEnemy
    }
}

components! {
    struct Components {
        #[hot] net_entity: NetEntity,
        #[hot] server_net_entity: ServerNetEntity,

        #[hot] position: Position,
        #[hot] orientation: Orientation,
        #[hot] linear_velocity: LinearVelocity,
        #[cold] player_state: PlayerState,
        #[cold] item_spawn: ItemSpawn,
        #[cold] bouncy_enemy: BouncyEnemy,
    }
}

impl HasPosition for Components {
    fn position(&self) -> &ComponentList<Components, Position> {
        &self.position
    }
    fn position_mut(&mut self) -> &mut ComponentList<Components, Position> {
        &mut self.position
    }
}

impl HasOrientation for Components {
    fn orientation(&self) -> &ComponentList<Components, Orientation> {
        &self.orientation
    }
    fn orientation_mut(&mut self) -> &mut ComponentList<Components, Orientation> {
        &mut self.orientation
    }
}

impl HasLinearVelocity for Components {
    fn linear_velocity(&self) -> &ComponentList<Components, LinearVelocity> {
        &self.linear_velocity
    }
    fn linear_velocity_mut(&mut self) -> &mut ComponentList<Components, LinearVelocity> {
        &mut self.linear_velocity 
    }
}

impl HasPlayerState for Components {
    fn player_state(&self) -> &ComponentList<Components, PlayerState> {
        &self.player_state
    }
    fn player_state_mut(&mut self) -> &mut ComponentList<Components, PlayerState> {
        &mut self.player_state
    }
}

impl HasItemSpawn for Components {
    fn item_spawn(&self) -> &ComponentList<Components, ItemSpawn> {
        &self.item_spawn
    }
    fn item_spawn_mut(&mut self) -> &mut ComponentList<Components, ItemSpawn> {
        &mut self.item_spawn
    }
}

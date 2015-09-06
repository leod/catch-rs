use ecs::ComponentList;

use shared::components::{HasPosition, HasOrientation, HasPlayerState};
pub use shared::components::{NetEntity, Position,
                             Orientation, PlayerState,
                             ComponentTypeTraits,
                             component_type_traits};

components! {
    struct Components {
        #[hot] position: Position,
        #[hot] orientation: Orientation,
        #[hot] net_entity: NetEntity,

        #[cold] player_state: PlayerState,
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

impl HasPlayerState for Components {
    fn player_state(&self) -> &ComponentList<Components, PlayerState> {
        &self.player_state
    }
    fn player_state_mut(&mut self) -> &mut ComponentList<Components, PlayerState> {
        &mut self.player_state
    }
}

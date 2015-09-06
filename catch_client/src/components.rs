use ecs::ComponentList;

use shared::math;
use shared::components::{HasPosition, HasOrientation, HasPlayerState};
pub use shared::components::{NetEntity, Position,
                             Orientation, PlayerState,
                             ComponentTypeTraits,
                             component_type_traits};

pub struct DrawPlayer;

pub trait Interpolatable {
    fn interpolate(&Self, &Self, t: f64) -> Self; 
}

pub struct InterpolationState<T: Interpolatable> {
    state: Option<(T, T)>
}

impl<T: Interpolatable> InterpolationState<T> {
    pub fn some(a: T, b: T) -> InterpolationState<T> {
        InterpolationState {
            state: Some((a, b))
        }
    }

    pub fn none() -> InterpolationState<T> {
        InterpolationState {
            state: None
        }
    }
}

components! {
    struct Components {
        #[hot] net_entity: NetEntity,

        #[hot] position: Position,
        #[hot] orientation: Orientation,

        #[hot] interp_state_orientation: InterpolationState<Orientation>,
        #[hot] interp_state_position: InterpolationState<Position>,

        #[cold] player_state: PlayerState,

        #[cold] draw_player: DrawPlayer,
    }
}

impl Interpolatable for Position {
    fn interpolate(a: &Position, b: &Position, t: f64) -> Position {
        Position {
            p: math::add(math::scale(a.p, 1.0 - t),
                         math::scale(b.p, t))
        }
    }
}

impl Interpolatable for Orientation {
    fn interpolate(a: &Orientation, b: &Orientation, t: f64) -> Orientation {
        Orientation {
            angle: (1.0 - t) * a.angle + b.angle
        }
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

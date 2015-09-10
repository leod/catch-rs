use ecs::ComponentList;

use shared::math;
use shared::components::{HasPosition, HasOrientation,
                         HasLinearVelocity, HasPlayerState,
                         HasItemSpawn};
pub use shared::components::{NetEntity, Position,
                             Orientation, LinearVelocity, 
                             PlayerState, ItemSpawn,
                             ComponentTypeTraits,
                             component_type_traits};

pub struct DrawPlayer {
    pub scale_x: f64
}

pub struct DrawBouncyEnemy;

impl Default for DrawPlayer {
    fn default() -> DrawPlayer {
        DrawPlayer {
            scale_x: 1.0,
        }
    }
}

impl Default for DrawBouncyEnemy {
    fn default() -> DrawBouncyEnemy {
        DrawBouncyEnemy
    }
}

pub trait Interpolatable {
    fn interpolate(&Self, &Self, t: f64) -> Self; 
}

pub struct InterpolationState<T: Interpolatable> {
    pub state: Option<(T, T)>
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
        #[hot] linear_velocity: LinearVelocity,
        #[cold] player_state: PlayerState,
        #[cold] item_spawn: ItemSpawn,

        #[hot] interp_position: InterpolationState<Position>,
        #[hot] interp_orientation: InterpolationState<Orientation>,

        #[cold] draw_player: DrawPlayer,
        #[cold] draw_bouncy_enemy: DrawBouncyEnemy,
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
            angle: (1.0 - t) * a.angle + t * b.angle
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

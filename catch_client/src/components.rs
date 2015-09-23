use ecs::ComponentList;
use graphics;

use shared::math;
use shared::util::PeriodicTimer;
use shared::components::{HasPosition, HasOrientation, HasLinearVelocity, HasShape, HasPlayerState,
                         HasFullPlayerState};
pub use shared::components::{NetEntity, Position, Orientation, LinearVelocity, Shape, PlayerState,
                             FullPlayerState, ComponentTypeTraits, component_type_traits};

pub struct DrawPlayer {
    pub scale_x: f64,
    pub color: graphics::types::Color,
    pub dash_particle_timer: PeriodicTimer,
}

impl Default for DrawPlayer {
    fn default() -> DrawPlayer {
        DrawPlayer {
            scale_x: 1.0,
            color: [0.0, 0.0, 0.0, 1.0],
            dash_particle_timer: PeriodicTimer::new(0.01),
        }
    }
}

#[derive(Default)]
pub struct DrawBouncyEnemy;

pub struct DrawItem {
    pub particle_timer: PeriodicTimer,
}

impl Default for DrawItem {
    fn default() -> DrawItem {
        DrawItem {
            particle_timer: PeriodicTimer::new(0.25),
        }
    }
}

#[derive(Default)]
pub struct DrawShadow;

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

        // Networked components
        #[hot] position: Position,
        #[hot] orientation: Orientation,
        #[hot] linear_velocity: LinearVelocity,
        #[hot] shape: Shape,
        #[cold] player_state: PlayerState,
        #[cold] full_player_state: FullPlayerState, // We will only have our own full player state 

        // Interpolation
        #[hot] interp_position: InterpolationState<Position>,
        #[hot] interp_orientation: InterpolationState<Orientation>,

        // Display
        #[cold] draw_player: DrawPlayer,
        #[cold] draw_bouncy_enemy: DrawBouncyEnemy,
        #[cold] draw_item: DrawItem,
        #[cold] draw_shadow: DrawShadow,
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

impl HasShape for Components {
    fn shape(&self) -> &ComponentList<Components, Shape> {
        &self.shape
    }
    fn shape_mut(&mut self) -> &mut ComponentList<Components, Shape> {
        &mut self.shape
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

impl HasFullPlayerState for Components {
    fn full_player_state(&self) -> &ComponentList<Components, FullPlayerState> {
        &self.full_player_state
    }
    fn full_player_state_mut(&mut self) -> &mut ComponentList<Components, FullPlayerState> {
        &mut self.full_player_state
    }
}

use std::collections::HashMap;

use components::{Position, Orientation, LinearVelocity, Shape, PlayerState, FullPlayerState,
                 WallPosition};
use net::ComponentType;
use super::{EntityId, TickNumber, GameEvent};

/// Stores the state of all net components in a tick
pub type ComponentsNetState<T> = HashMap<EntityId, T>;

#[derive(Default, RustcEncodable, RustcDecodable)]
pub struct TickState {
    // Same order as net::COMPONENT_TYPES

    pub position: ComponentsNetState<Position>, 
    pub orientation: ComponentsNetState<Orientation>,
    pub linear_velocity: ComponentsNetState<LinearVelocity>,
    pub shape: ComponentsNetState<Shape>,
    pub player_state: ComponentsNetState<PlayerState>,
    pub full_player_state: ComponentsNetState<FullPlayerState>,
    pub wall_position: ComponentsNetState<WallPosition>,

    // List of components that should not be interpolated into this tick
    // (e.g. you wouldn't want to interpolate the position of a player that was just teleported)
    pub forced_components: Vec<(EntityId, ComponentType)>,
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct Tick {
    pub tick_number: TickNumber,
    pub events: Vec<GameEvent>,
    pub state: TickState,
}

impl Tick {
    pub fn new(tick_number: TickNumber) -> Tick {
        Tick {
            tick_number: tick_number,
            events: Vec::new(),
            state: TickState::default(),
        }
    }
}

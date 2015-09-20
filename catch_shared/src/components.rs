use std::marker::PhantomData;

use ecs::{ComponentManager, ComponentList, BuildData, EntityData};

use net::{COMPONENT_TYPES, ComponentType};
use super::{EntityId, EntityTypeId, PlayerId, Item, TickState};
use math;
pub use player::PlayerState;

/// Every entity that wants its component state synchronized needs to have this component
pub struct NetEntity {
    pub id: EntityId,
    pub type_id: EntityTypeId,
    pub owner: PlayerId,
}

#[derive(Clone, Default, CerealData)]
pub struct Position {
    pub p: math::Vec2,
}

#[derive(Clone, Default, CerealData)]
pub struct Orientation {
    pub angle: f64,
}

#[derive(Clone, Default, CerealData)]
pub struct LinearVelocity {
    pub v: math::Vec2,
}

#[derive(Clone, CerealData)]
pub enum Shape { 
    Circle {
        radius: f64
    }
}

impl Default for Shape {
    fn default() -> Shape {
        Shape::Circle { radius: 1.0 } // meh
    }
}

// Attached to players on the server and the clients controlling them
// Item states, cooldowns etc.
#[derive(Clone, Default, CerealData)]
pub struct FullPlayerState {
    pub dash_cooldown_s: Option<f64>,

    // An item that the player picked up but hasn't equipped
    pub hidden_item: Option<Item>,
}


#[derive(Clone, Default, CerealData)]
pub struct ItemSpawn;

// Some boilerplate code for each net component type follows...

pub trait HasPosition {
    fn position(&self) -> &ComponentList<Self, Position>;
    fn position_mut(&mut self) -> &mut ComponentList<Self, Position>;
}

pub trait HasOrientation {
    fn orientation(&self) -> &ComponentList<Self, Orientation>;
    fn orientation_mut(&mut self) -> &mut ComponentList<Self, Orientation>;
}

pub trait HasLinearVelocity {
    fn linear_velocity(&self) -> &ComponentList<Self, LinearVelocity>;
    fn linear_velocity_mut(&mut self) -> &mut ComponentList<Self, LinearVelocity>;
}

pub trait HasShape {
    fn shape(&self) -> &ComponentList<Self, Shape>;
    fn shape_mut(&mut self) -> &mut ComponentList<Self, Shape>;
}

pub trait HasPlayerState {
    fn player_state(&self) -> &ComponentList<Self, PlayerState>;
    fn player_state_mut(&mut self) -> &mut ComponentList<Self, PlayerState>;
}

pub trait HasFullPlayerState {
    fn full_player_state(&self) -> &ComponentList<Self, FullPlayerState>;
    fn full_player_state_mut(&mut self) -> &mut ComponentList<Self, FullPlayerState>;
}

pub trait StateComponent<T: ComponentManager> {
    // Add net component to the component manager for the given entity
    fn add(&self, entity: BuildData<T>, c: &mut T);

    // Stores current component state in a TickState
    fn store(&self, entity: EntityData<T>, id: EntityId, write: &mut TickState, c: &T);

    // Load component state from TickState
    fn load(&self, entity: EntityData<T>, id: EntityId, net_state: &TickState, c: &mut T);
}

struct StateComponentImpl<C>(PhantomData<C>);

macro_rules! state_component_impl {
    ($trait_ty: ident, $ty: ident, $field: ident, $field_mut: ident) => {
        impl<T: ComponentManager> StateComponent<T> for StateComponentImpl<$ty>
            where T: $trait_ty {
            fn add(&self, entity: BuildData<T>, c: &mut T) {
                c.$field_mut().add(&entity, $ty::default());
            }
            fn store(&self, entity: EntityData<T>, id: EntityId, net_state: &mut TickState,
                     c: &T) {
                net_state.$field.insert(id, c.$field()[entity].clone());
            }
            fn load(&self, entity: EntityData<T>, id: EntityId, net_state: &TickState, c: &mut T) {
                if let Some($field) = net_state.$field.get(&id) {
                    c.$field_mut()[entity] = $field.clone();
                }
            }
        }
    };
}

state_component_impl!(HasPosition, Position, position, position_mut);
state_component_impl!(HasOrientation, Orientation, orientation, orientation_mut);
state_component_impl!(HasLinearVelocity, LinearVelocity, linear_velocity, linear_velocity_mut);
state_component_impl!(HasShape, Shape, shape, shape_mut);
state_component_impl!(HasPlayerState, PlayerState, player_state, player_state_mut);
state_component_impl!(HasFullPlayerState, FullPlayerState, full_player_state, full_player_state_mut);

pub type ComponentTypeTraits<T> = Vec<Box<StateComponent<T>>>;

pub fn component_type_traits<T: ComponentManager +
                                HasPosition +
                                HasOrientation +
                                HasLinearVelocity +
                                HasShape +
                                HasPlayerState +
                                HasFullPlayerState>() -> ComponentTypeTraits<T> {
    let mut traits = ComponentTypeTraits::<T>::new();

    for component_type in COMPONENT_TYPES.iter() {
        match *component_type {
            ComponentType::Position => 
                traits.push(Box::new(StateComponentImpl::<Position>(PhantomData))),
            ComponentType::Orientation =>
                traits.push(Box::new(StateComponentImpl::<Orientation>(PhantomData))),
            ComponentType::LinearVelocity =>
                traits.push(Box::new(StateComponentImpl::<LinearVelocity>(PhantomData))),
            ComponentType::Shape =>
                traits.push(Box::new(StateComponentImpl::<Shape>(PhantomData))),
            ComponentType::PlayerState =>
                traits.push(Box::new(StateComponentImpl::<PlayerState>(PhantomData))),
            ComponentType::FullPlayerState =>
                traits.push(Box::new(StateComponentImpl::<FullPlayerState>(PhantomData))),
        };
    }

    traits
}

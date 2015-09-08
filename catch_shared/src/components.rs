#![feature(concat_idents)]

use std::marker::PhantomData;

use ecs::{ComponentManager, ComponentList, BuildData, EntityData};

use net::{StateComponent, EntityId, EntityTypeId, COMPONENT_TYPES, ComponentType};
use player::PlayerId;
use tick::NetState;
use math;

/// Every entity that wants its component state synchronized needs to have this component
pub struct NetEntity {
    pub id: EntityId,
    pub type_id: EntityTypeId,
    pub owner: PlayerId,
}

#[derive(CerealData, Clone)]
pub struct Position {
    pub p: math::Vec2,
}

#[derive(CerealData, Clone)]
pub struct Orientation {
    pub angle: f64,
}

#[derive(CerealData, Clone)]
pub struct LinearVelocity {
    pub v: math::Vec2,
}

#[derive(CerealData, Clone)]
pub struct PlayerState {
    pub color: u32,
    pub dashing: Option<f64>,
}

impl Default for Position {
    fn default() -> Position {
        Position {
            p: [0.0, 0.0]
        }
    }
}

impl Default for Orientation {
    fn default() -> Orientation {
        Orientation {
            angle: 0.0
        }
    }
}

impl Default for LinearVelocity {
    fn default() -> LinearVelocity {
        LinearVelocity {
            v: [0.0, 0.0]
        }
    }
}

impl Default for PlayerState {
    fn default() -> PlayerState {
        PlayerState {
            color: 0,
            dashing: None,
        }
    }
}

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

pub trait HasPlayerState {
    fn player_state(&self) -> &ComponentList<Self, PlayerState>;
    fn player_state_mut(&mut self) -> &mut ComponentList<Self, PlayerState>;
}

struct StateComponentImpl<C>(PhantomData<C>);

macro_rules! state_component_impl {
    ($trait_ty: ident, $ty: ident, $field: ident, $field_mut: ident) => {
        impl<T: ComponentManager> StateComponent<T> for StateComponentImpl<$ty> where T: $trait_ty {
            fn add(&self, entity: BuildData<T>, c: &mut T) {
                c.$field_mut().add(&entity, $ty::default());
            }
            fn write(&self, entity: EntityData<T>, id: EntityId, net_state: &mut NetState, c: &T) {
                net_state.$field.insert(id, c.$field()[entity].clone());
            }
            fn read(&self, entity: EntityData<T>, id: EntityId, net_state: &NetState, c: &mut T) {
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
state_component_impl!(HasPlayerState, PlayerState, player_state, player_state_mut);

pub type ComponentTypeTraits<T> = Vec<Box<StateComponent<T>>>;

pub fn component_type_traits<T: ComponentManager +
                                HasPosition +
                                HasOrientation +
                                HasPlayerState +
                                HasLinearVelocity>() -> ComponentTypeTraits<T> {
    let mut traits = ComponentTypeTraits::<T>::new();

    for component_type in COMPONENT_TYPES.iter() {
        match *component_type {
            ComponentType::Position => 
                traits.push(Box::new(StateComponentImpl::<Position>(PhantomData))),
            ComponentType::Orientation =>
                traits.push(Box::new(StateComponentImpl::<Orientation>(PhantomData))),
            ComponentType::LinearVelocity =>
                traits.push(Box::new(StateComponentImpl::<LinearVelocity>(PhantomData))),
            ComponentType::PlayerState =>
                traits.push(Box::new(StateComponentImpl::<PlayerState>(PhantomData))),
        };
    }

    traits
}

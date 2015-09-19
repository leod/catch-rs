use std::marker::PhantomData;

use ecs::{ComponentManager, ComponentList, BuildData, EntityData};

pub use player::{PlayerState, FullPlayerState};
use net::{StateComponent, EntityId, EntityTypeId, COMPONENT_TYPES, ComponentType};
use player::PlayerId;
use item::Item;
use tick::NetState;
use math;

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

#[derive(Clone, Default, CerealData)]
pub struct ItemSpawn {
    pub item: Option<Item>,
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

pub trait HasFullPlayerState {
    fn full_player_state(&self) -> &ComponentList<Self, FullPlayerState>;
    fn full_player_state_mut(&mut self) -> &mut ComponentList<Self, FullPlayerState>;
}

pub trait HasItemSpawn {
    fn item_spawn(&self) -> &ComponentList<Self, ItemSpawn>;
    fn item_spawn_mut(&mut self) -> &mut ComponentList<Self, ItemSpawn>;
}

struct StateComponentImpl<C>(PhantomData<C>);

macro_rules! state_component_impl {
    ($trait_ty: ident, $ty: ident, $field: ident, $field_mut: ident) => {
        impl<T: ComponentManager> StateComponent<T> for StateComponentImpl<$ty>
            where T: $trait_ty {
            fn add(&self, entity: BuildData<T>, c: &mut T) {
                c.$field_mut().add(&entity, $ty::default());
            }
            fn store(&self, entity: EntityData<T>, id: EntityId, net_state: &mut NetState,
                     c: &T) {
                net_state.$field.insert(id, c.$field()[entity].clone());
            }
            fn load(&self, entity: EntityData<T>, id: EntityId, net_state: &NetState, c: &mut T) {
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
state_component_impl!(HasFullPlayerState, FullPlayerState, full_player_state, full_player_state_mut);
state_component_impl!(HasItemSpawn, ItemSpawn, item_spawn, item_spawn_mut);

pub type ComponentTypeTraits<T> = Vec<Box<StateComponent<T>>>;

pub fn component_type_traits<T: ComponentManager +
                                HasPosition +
                                HasOrientation +
                                HasLinearVelocity +
                                HasPlayerState +
                                HasFullPlayerState +
                                HasItemSpawn>() -> ComponentTypeTraits<T> {
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
            ComponentType::FullPlayerState =>
                traits.push(Box::new(StateComponentImpl::<FullPlayerState>(PhantomData))),
            ComponentType::ItemSpawn =>
                traits.push(Box::new(StateComponentImpl::<ItemSpawn>(PhantomData))),
        };
    }

    traits
}

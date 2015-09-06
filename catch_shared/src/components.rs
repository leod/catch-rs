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
pub struct PlayerState {
    pub color: u32
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

impl Default for PlayerState {
    fn default() -> PlayerState {
        PlayerState {
            color: 0,
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

pub trait HasPlayerState {
    fn player_state(&self) -> &ComponentList<Self, PlayerState>;
    fn player_state_mut(&mut self) -> &mut ComponentList<Self, PlayerState>;
}

struct StateComponentImpl<C>(PhantomData<C>);

impl<T: ComponentManager> StateComponent<T> for StateComponentImpl<Position> where T: HasPosition {
    fn add(&self, entity: BuildData<T>, c: &mut T) {
        c.position_mut().add(&entity, Position::default());
    }
    fn write(&self, entity: EntityData<T>, id: EntityId, net_state: &mut NetState, c: &T) {
        net_state.position.insert(id, c.position()[entity].clone());
    }
    fn read(&self, entity: EntityData<T>, id: EntityId, net_state: &NetState, c: &mut T) {
        if let Some(position) = net_state.position.get(&id) {
            c.position_mut()[entity] = position.clone();
        }
    }
}

impl<T: ComponentManager> StateComponent<T> for StateComponentImpl<Orientation> where T: HasOrientation {
    fn add(&self, entity: BuildData<T>, c: &mut T) {
        c.orientation_mut().add(&entity, Orientation::default());
    }
    fn write(&self, entity: EntityData<T>, id: EntityId, net_state: &mut NetState, c: &T) {
        net_state.orientation.insert(id, c.orientation()[entity].clone());
    }
    fn read(&self, entity: EntityData<T>, id: EntityId, net_state: &NetState, c: &mut T) {
        if let Some(orientation) = net_state.orientation.get(&id) {
            c.orientation_mut()[entity] = orientation.clone();
        }
    }
}

impl<T: ComponentManager> StateComponent<T> for StateComponentImpl<PlayerState> where T: HasPlayerState {
    fn add(&self, entity: BuildData<T>, c: &mut T) {
        c.player_state_mut().add(&entity, PlayerState::default());
    }
    fn write(&self, entity: EntityData<T>, id: EntityId, net_state: &mut NetState, c: &T) {
        net_state.player_state.insert(id, c.player_state()[entity].clone());
    }
    fn read(&self, entity: EntityData<T>, id: EntityId, net_state: &NetState, c: &mut T) {
        if let Some(player_state) = net_state.player_state.get(&id) {
            c.player_state_mut()[entity] = player_state.clone();
        }
    }
}

pub type ComponentTypeTraits<T> = Vec<Box<StateComponent<T>>>;

pub fn component_type_traits<T: ComponentManager +
                                HasPosition +
                                HasOrientation +
                                HasPlayerState>() -> ComponentTypeTraits<T> {
    let mut traits = ComponentTypeTraits::<T>::new();

    for component_type in COMPONENT_TYPES.iter() {
        match *component_type {
            ComponentType::Position => 
                traits.push(Box::new(StateComponentImpl::<Position>(PhantomData))),
            ComponentType::Orientation =>
                traits.push(Box::new(StateComponentImpl::<Orientation>(PhantomData))),
            ComponentType::PlayerState =>
                traits.push(Box::new(StateComponentImpl::<PlayerState>(PhantomData))),
        };
    }

    traits
}

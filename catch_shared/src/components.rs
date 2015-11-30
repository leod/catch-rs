use na::Vec2;
use ecs::{ComponentManager, ComponentList};

use super::{EntityId, EntityTypeId, PlayerId};
pub use player::{PlayerState, FullPlayerState};

/// Every entity that wants its component state synchronized needs to have this component
pub struct NetEntity {
    pub id: EntityId,
    pub type_id: EntityTypeId,
    pub owner: PlayerId,
}

#[derive(PartialEq, Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct Position {
    pub p: Vec2<f32>,
}

impl Default for Position {
    fn default() -> Position {
        Position {
            p: Vec2::new(0.0, 0.0)
        }
    }
}

#[derive(PartialEq, Debug, Clone, Default, RustcEncodable, RustcDecodable)]
pub struct Orientation {
    pub angle: f32, // radians
}

#[derive(PartialEq, Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct LinearVelocity {
    pub v: Vec2<f32>,
}

impl Default for LinearVelocity {
    fn default() -> LinearVelocity {
        LinearVelocity {
            v: Vec2::new(0.0, 0.0)
        }
    }
}

#[derive(PartialEq, Debug, Clone, Default, RustcEncodable, RustcDecodable)]
pub struct AngularVelocity {
    pub v: f32,
}

#[derive(PartialEq, Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum Shape { 
    Circle {
        radius: f32,
    },
    Square {
        size: f32,
    },
    Rect {
        width: f32,
        height: f32
    }
}

impl Default for Shape {
    fn default() -> Shape {
        Shape::Circle { radius: 1.0 } // meh
    }
}

impl Shape {
    pub fn radius(&self) -> f32 {
        match *self {
            Shape::Circle { radius } => radius,
            Shape::Square { size } => size / 2.0,
            Shape::Rect { width, height } => width.max(height) / 2.0,
        }
    }
}

#[derive(PartialEq, Debug, Clone, RustcEncodable, RustcDecodable)]
pub enum Projectile {
    Bullet,
    Frag(f32),
    Shrapnel
}

impl Projectile {
    pub fn lethal_to_owner(&self) -> bool {
        match *self {
            Projectile::Bullet => false,
            Projectile::Frag(_) => false,
            Projectile::Shrapnel => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WallType {
    Iron,
    Wood
}

impl Default for WallType {
    fn default() -> WallType {
        WallType::Iron
    }
}

#[derive(Debug, Clone, Default)]
pub struct Wall {
    pub wall_type: WallType,
    pub width: f32,
}

#[derive(PartialEq, Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct WallPosition {
    pub pos_a: Vec2<f32>,
    pub pos_b: Vec2<f32>,
}

impl Default for WallPosition {
    fn default() -> WallPosition {
        WallPosition {
            pos_a: Vec2::new(0.0, 0.0),
            pos_b: Vec2::new(0.0, 0.0)
        }
    }
}

// Some boilerplate code for each net component type follows...

pub trait HasPosition: Sized + ComponentManager {
    fn position(&self) -> &ComponentList<Self, Position>;
    fn position_mut(&mut self) -> &mut ComponentList<Self, Position>;
}

pub trait HasOrientation: Sized + ComponentManager {
    fn orientation(&self) -> &ComponentList<Self, Orientation>;
    fn orientation_mut(&mut self) -> &mut ComponentList<Self, Orientation>;
}

pub trait HasLinearVelocity: Sized + ComponentManager {
    fn linear_velocity(&self) -> &ComponentList<Self, LinearVelocity>;
    fn linear_velocity_mut(&mut self) -> &mut ComponentList<Self, LinearVelocity>;
}

pub trait HasAngularVelocity: Sized + ComponentManager {
    fn angular_velocity(&self) -> &ComponentList<Self, AngularVelocity>;
    fn angular_velocity_mut(&mut self) -> &mut ComponentList<Self, AngularVelocity>;
}

pub trait HasShape: Sized + ComponentManager {
    fn shape(&self) -> &ComponentList<Self, Shape>;
    fn shape_mut(&mut self) -> &mut ComponentList<Self, Shape>;
}

pub trait HasPlayerState: Sized + ComponentManager {
    fn player_state(&self) -> &ComponentList<Self, PlayerState>;
    fn player_state_mut(&mut self) -> &mut ComponentList<Self, PlayerState>;
}

pub trait HasFullPlayerState: Sized + ComponentManager {
    fn full_player_state(&self) -> &ComponentList<Self, FullPlayerState>;
    fn full_player_state_mut(&mut self) -> &mut ComponentList<Self, FullPlayerState>;
}

pub trait HasWallPosition: Sized + ComponentManager {
    fn wall_position(&self) -> &ComponentList<Self, WallPosition>;
    fn wall_position_mut(&mut self) -> &mut ComponentList<Self, WallPosition>;
}

pub trait HasWall: Sized + ComponentManager {
    fn wall(&self) -> &ComponentList<Self, Wall>;
    fn wall_mut(&mut self) -> &mut ComponentList<Self, Wall>;
}

pub trait HasProjectile: Sized + ComponentManager {
    fn projectile(&self) -> &ComponentList<Self, Projectile>;
    fn projectile_mut(&mut self) -> &mut ComponentList<Self, Projectile>;
}

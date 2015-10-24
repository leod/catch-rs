use ecs;
use ecs::ComponentList;

use shared::net::{self, TimedPlayerInput};
use shared::player::Item;
use shared::components::{HasPosition, HasOrientation, HasLinearVelocity, HasShape, HasPlayerState,
                         HasFullPlayerState, HasWallPosition, HasAngularVelocity, HasWall};
pub use shared::components::{NetEntity, Position, Orientation, LinearVelocity, Shape, PlayerState,
                             Projectile, FullPlayerState, AngularVelocity, Wall, WallPosition,
                             ComponentTypeTraits, component_type_traits};

/// Server-side information about net entities
#[derive(Default)]
pub struct ServerNetEntity {
    // Components that should not be interpolated by clients into the current tick
    pub forced_components: Vec<net::ComponentType>,

    // Used to prevent ecs entity removal events being queued multiple times for the same entity
    pub removed: bool,
}

impl ServerNetEntity {
    pub fn force(&mut self, component_type: net::ComponentType) {
        self.forced_components.push(component_type);
    }
}

/// Player input for an entity
#[derive(Default)]
pub struct PlayerController {
    pub inputs: Vec<TimedPlayerInput>,
}

/// Server-side information about bouncy enemies
#[derive(Default)]
pub struct BouncyEnemy {
    pub orbit: Option<ecs::Entity>,
}

/// Tag component for InteractionSystem
pub struct Interact;

/// Tag component for RotateSystem
pub struct Rotate;

/// Server-side information about item spawns
#[derive(Default)]
pub struct ItemSpawn {
    pub spawned_entity: Option<ecs::Entity>,
    pub cooldown_s: Option<f32>,
}

components! {
    struct Components {
        #[hot] net_entity: NetEntity,
        #[hot] server_net_entity: ServerNetEntity,

        // Networked components
        #[hot] position: Position,
        #[hot] orientation: Orientation,
        #[hot] linear_velocity: LinearVelocity,
        #[hot] shape: Shape,
        #[cold] player_state: PlayerState,
        #[cold] full_player_state: FullPlayerState,
        #[hot] wall_position: WallPosition,
        #[hot] wall: Wall,

        #[cold] angular_velocity: AngularVelocity,

        #[cold] player_controller: PlayerController,
        #[cold] bouncy_enemy: BouncyEnemy,
        #[cold] item: Item,
        #[cold] item_spawn: ItemSpawn,
        #[cold] rotate: Rotate,
        #[cold] projectile: Projectile,
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

impl HasAngularVelocity for Components {
    fn angular_velocity(&self) -> &ComponentList<Components, AngularVelocity> {
        &self.angular_velocity
    }
    fn angular_velocity_mut(&mut self) -> &mut ComponentList<Components, AngularVelocity> {
        &mut self.angular_velocity 
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

impl HasWallPosition for Components {
    fn wall_position(&self) -> &ComponentList<Components, WallPosition> {
        &self.wall_position
    }
    fn wall_position_mut(&mut self) -> &mut ComponentList<Components, WallPosition> {
        &mut self.wall_position
    }
}

impl HasWall for Components {
    fn wall(&self) -> &ComponentList<Components, Wall> {
        &self.wall
    }
    fn wall_mut(&mut self) -> &mut ComponentList<Components, Wall> {
        &mut self.wall
    }
}

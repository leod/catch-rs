use std::f32;

use ecs;
use ecs::{Aspect, Process, System, EntityData, DataHelper};

use shared::{math, movement, Map, GameEvent};
use shared::net::{ComponentType, TimedPlayerInput};
use shared::util::CachedAspect;
use shared::player::PlayerInputKey;

use components::Components;
use services::Services;

/// System for interpreting player input on the server side
pub struct PlayerControllerSystem {
    player_aspect: CachedAspect<Components>,
    wall_aspect: CachedAspect<Components>,
}

impl PlayerControllerSystem {
    pub fn new(player_aspect: Aspect<Components>,
               wall_aspect: Aspect<Components>) -> PlayerControllerSystem {
        PlayerControllerSystem {
            player_aspect: CachedAspect::new(player_aspect),
            wall_aspect: CachedAspect::new(wall_aspect),
        }
    }

    pub fn run_queued_inputs(&self, data: &mut DataHelper<Components, Services>) {
        for player in self.player_aspect.iter() {
            let inputs = data.player_controller[player].inputs.clone();
            data.player_controller[player].inputs.clear();

            let owner = data.net_entity[player].owner;

            for input in &inputs {
                movement::run_player_movement_input(player, owner, input, &self.wall_aspect, data);
            }
        }
    }
}

impl_cached_system!(Components, Services, PlayerControllerSystem, player_aspect, wall_aspect);

impl Process for PlayerControllerSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

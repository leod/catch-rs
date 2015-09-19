use ecs;
use ecs::{Process, System, EntityData, DataHelper};

use shared::map::Map;
use shared::net::TimedPlayerInput;
use shared::player::{PlayerInput, InputKey};
use components::Components;
use services::Services;

pub struct PlayerItemSystem;

impl PlayerItemSystem {
    pub fn new() -> PlayerItemSystem {
        PlayerItemSystem
    }

    pub fn run_player_input(&self,
                            entity: ecs::Entity,
                            timed_input: &TimedPlayerInput,
                            map: &Map,
                            data: &mut DataHelper<Components, Services>) {
        data.with_entity_data(&entity, |e, c| {

        });
    }
}

impl System for PlayerItemSystem {
    type Components = Components;
    type Services = Services;
}

impl Process for PlayerItemSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

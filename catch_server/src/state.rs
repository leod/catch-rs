use std::collections::HashMap;

use ecs;

use shared::player::{PlayerId, PlayerInfo};
use systems::Systems;

pub struct Player {
    pub info: PlayerInfo,
}

pub struct GameState {
    pub world: ecs::World<Systems>, 

    players: HashMap<PlayerId, Player>,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            world: ecs::World::new(),
            players: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, info: PlayerInfo) {
        let id = info.id;
        assert!(self.players.get(&id).is_none());

        let player = Player {
            info: info,
        };

        self.players.insert(id, player);
    }

    pub fn remove_player(&mut self, id: PlayerId) {
        assert!(self.players.get(&id).is_some());
        self.players.remove(&id);
    }

    pub fn get_player_info(&self, id: PlayerId) -> &PlayerInfo {
        &self.players[&id].info
    }
}

use std::collections::HashMap;

use ecs;

use shared::net;
use shared::math;
use shared::net::{GameInfo, TickNumber};
use shared::tick::Tick;
use shared::player::{PlayerId, PlayerInfo, PlayerInput};
use systems::{Systems, NetEntitySystem};

pub struct GameState {
    pub world: ecs::World<Systems>, 
    pub tick_number: Option<TickNumber>,

    players: HashMap<PlayerId, PlayerInfo>,
}

impl GameState {
    pub fn new(game_info: &GameInfo) -> GameState {
        let mut world = ecs::World::<Systems>::new();
        world.systems.net_entity_system.init(NetEntitySystem::new(&game_info.entity_types));

        GameState {
            world: world,
            tick_number: None,
            players: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, info: PlayerInfo) {
        let id = info.id;
        assert!(self.players.get(&id).is_none());
        self.players.insert(id, info);
    }

    pub fn remove_player(&mut self, id: PlayerId) {
        assert!(self.players.get(&id).is_some());
        self.players.remove(&id);
    }

    pub fn get_player_info(&self, id: PlayerId) -> &PlayerInfo {
        &self.players[&id]
    }

    pub fn on_local_player_input(&mut self, input: &PlayerInput) {
        // TODO: Client-side prediction
    }

    pub fn run_tick(&mut self, tick: &Tick) {
        let net_entity_system = self.world.systems.net_entity_system.inner.as_mut().unwrap();
        // Create new entities, remove dead ones
        net_entity_system.process_entity_events(tick, &mut self.world.data);

        // Load net state
        net_entity_system.load_tick_state(tick, &mut self.world.data);

        // TODO: Process other events, interpolation
    }
}

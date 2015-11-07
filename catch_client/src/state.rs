use std::collections::HashMap;

use ecs;
use hprof;

use shared::{GameEvent, GameInfo, TickNumber, PlayerId, PlayerInfo, PlayerInput, Tick, Map};
use systems::{Systems, NetEntitySystem};
use components::Components;

pub struct GameState {
    pub game_info: GameInfo,
    pub map: Map,

    pub world: ecs::World<Systems>, 
    pub tick_number: Option<TickNumber>,

    players: HashMap<PlayerId, PlayerInfo>,
}

impl GameState {
    pub fn new(my_id: PlayerId, game_info: &GameInfo) -> GameState {
        let mut world = ecs::World::<Systems>::new();
        world.systems.net_entity_system.init(
            NetEntitySystem::new(aspect!(<Components> all: [net_entity]),
                                 my_id, &game_info.entity_types));

        GameState {
            game_info: game_info.clone(),
            map: Map::load(&game_info.map_name).unwrap(),
            world: world,
            tick_number: None,
            players: HashMap::new(),
        }
    }

    pub fn get_player_info(&self, id: PlayerId) -> &PlayerInfo {
        &self.players[&id]
    }

    pub fn players(&self) -> &HashMap<PlayerId, PlayerInfo> {
        &self.players
    }

    pub fn on_local_player_input(&mut self, _input: &PlayerInput) {
        // TODO: Client-side prediction
    }

    pub fn run_tick(&mut self, tick: &Tick) {
        let _g = hprof::enter("run tick");

        {
            let _g = hprof::enter("entity events");

            // Add or remove players, update player stats
            for i in 0..tick.events.len() {
                let event = tick.events[i].clone();
                self.process_game_event(event);
            }

            // Create new entities, remove dead ones
            self.world.systems.net_entity_system.inner.as_mut().unwrap()
                .process_entity_events(tick, &mut self.world.data);

            // Let all the systems know about any new ecs entities
            self.world.flush_queue();
        }

        {
            let _g = hprof::enter("load tick state");

            // Load net state
            self.world.systems.net_entity_system.inner.as_mut().unwrap()
                .load_tick_state(tick, &mut self.world.data);
        }
    }

    pub fn load_interp_tick_state(&mut self, tick_a: &Tick, tick_b: &Tick) {
        let _g = hprof::enter("load interp");

        self.world.systems.net_entity_system.inner.as_mut().unwrap()
            .load_interp_tick_state(tick_a, tick_b, &mut self.world.data);
    }

    fn add_player(&mut self, id: PlayerId, info: PlayerInfo) {
        assert!(self.players.get(&id).is_none());
        self.players.insert(id, info);
    }

    fn remove_player(&mut self, id: PlayerId) {
        assert!(self.players.get(&id).is_some());
        self.players.remove(&id);
    }

    fn process_game_event(&mut self, event: GameEvent) {
        match event { 
            GameEvent::InitialPlayerList(players) => {
                info!("received initial player list: {:?}", players);

                if !self.players.is_empty() {
                    warn!("received superfluous inital player list, ignoring");
                    return;
                }

                for (id, info) in players {
                    self.add_player(id, info);
                }
            }
            GameEvent::PlayerJoin(id, info) => {
                info!("player {} joined: {:?}", id, info);
                self.add_player(id, info);
            }
            GameEvent::PlayerLeave(id) => {
                info!("player {} left", id);
                self.remove_player(id);
            }
            GameEvent::UpdatePlayerStats(stats_list) => {
                for (id, stats) in stats_list {
                    self.players.get_mut(&id).unwrap().stats = stats;
                }
            }
            _ => ()
        }
    }
}

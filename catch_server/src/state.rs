use std::collections::HashMap;

use ecs;

use shared::net;
use shared::math;
use shared::map::Map;
use shared::net::{TickNumber, GameInfo};
use shared::tick::Tick;
use shared::event::GameEvent;
use shared::player::{PlayerId, PlayerInfo, PlayerInput};
use systems::Systems;

pub struct Player {
    // Has this player been sent its first tick yet?
    pub is_new: bool,

    pub info: PlayerInfo,
    pub next_input: Option<(TickNumber, PlayerInput)>,
    pub controlled_entity: Option<net::EntityId>
}

impl Player {
    fn new(info: PlayerInfo) -> Player {
        Player {
            is_new: true,
            info: info,
            next_input: None,
            controlled_entity: None,
        }
    }
}

pub struct GameState {
    pub game_info: GameInfo,
    pub map: Map,

    pub world: ecs::World<Systems>, 
    pub tick_number: TickNumber,

    players: HashMap<PlayerId, Player>,
}

impl GameState {
    pub fn new(game_info: &GameInfo) -> GameState {
        GameState {
            game_info: game_info.clone(),
            map: Map::load(&game_info.map_name).unwrap(),
            world: ecs::World::new(),
            tick_number: 0,
            players: HashMap::new(),
        }
    }

    pub fn tick_number(&self) -> TickNumber {
        self.tick_number 
    }

    pub fn add_player(&mut self, info: PlayerInfo) {
        let id = info.id;
        assert!(self.players.get(&id).is_none());

        self.players.insert(id, Player::new(info));
    }

    fn spawn_player(&mut self, id: PlayerId) {
        assert!(self.players[&id].controlled_entity.is_none(),
                "Can't spawn a player that is already controlling an entity");

        let net_entity_type_id = self.world.systems.net_entity_system.type_id("player".to_string());
        let (net_entity_id, _) = 
            self.world.systems.net_entity_system
                .create_entity(net_entity_type_id, id, &mut self.world.data);
        self.players.get_mut(&id).unwrap().controlled_entity = Some(net_entity_id);
    }

    pub fn remove_player(&mut self, id: PlayerId) {
        assert!(self.players.get(&id).is_some());
        self.players.remove(&id);
    }

    pub fn get_player_info(&self, id: PlayerId) -> &PlayerInfo {
        &self.players[&id].info
    }

    pub fn on_player_input(&mut self, id: PlayerId, input_client_tick: TickNumber, input: &PlayerInput) {
        // TODO: Should we be able to queue multiple inputs for each player?
        // Currently, the idea is for the clients to send one PlayerInput per tick.
        // Is it enough for the server to be able to execute one PlayerInput per tick?
        // It's probably a better idea to process PlayerInput as soon as it arrives at the
        // server...

        if self.players[&id].next_input.is_some() {
            println!("Warning: already have player input for {}", id);
        }

        self.players.get_mut(&id).as_mut().unwrap().next_input = Some((input_client_tick, input.clone()));
    }

    pub fn run_player_input(&mut self, player_id: PlayerId, net_entity_id: net::EntityId,
                            input_client_tick: net::TickNumber, input: &PlayerInput) {
        let entity = self.world.systems.net_entity_system.get_entity(net_entity_id);

        self.world.systems.player_movement_system
            .run_player_input(entity, input, &self.map, &mut self.world.data);

        // Tell the player in that their input has been processed.
        // TODO: Should this be done on a level thats finer than ticks?!
        // The following GameEvent will be sent with the next tick the server starts!
        self.world.services.add_player_event(player_id,
            GameEvent::CorrectState(input_client_tick));
    }

    // For now, the resulting tick data will be written in Services::next_tick
    pub fn tick(&mut self) {
        self.tick_number += 1;
        self.world.services.prepare_for_tick(self.tick_number, self.players.keys().map(|i| *i));

        // Replicate entities to new players
        {
            let mut new_players = Vec::new();
            for (player_id, player) in self.players.iter_mut() {
                if player.is_new {
                    new_players.push(*player_id);
                    player.is_new = false;
                }
            }
            for player_id in new_players {
                self.world.systems.net_entity_system
                    .replicate_entities(player_id, &mut self.world.data);
            }
        }

        // Spawn player entities if needed
        {
            let mut respawn = Vec::new();
            for (player_id, player) in self.players.iter_mut() {
                if !player.info.alive {
                    respawn.push(*player_id);
                    player.info.alive = true;
                }
            }
            for player_id in respawn {
                self.spawn_player(player_id); 
            }
        }

        // Let all the systems know about any new entities
        self.world.flush_queue();

        // Run input of players
        {
            let mut input = Vec::new();
            for (player_id, player) in self.players.iter() {
                match (player.controlled_entity, &player.next_input) {
                    (Some(net_entity_id),
                     &Some((ref input_client_tick, ref player_input))) => 
                         input.push((*player_id, net_entity_id, *input_client_tick, player_input.clone())),
                    _ => {}
                }
            }
            for (player_id, net_entity_id, input_client_tick, player_input) in input {
                self.run_player_input(player_id, net_entity_id, input_client_tick, &player_input);
            }
        }

        for (_, player) in self.players.iter_mut() {
            player.next_input = None;
        }

        process!(self.world, net_entity_system);
    }
}

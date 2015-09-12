use std::f64;
use std::collections::HashMap;

use ecs;
use rand;

use shared::net;
use shared::math;
use shared::map::{LayerId, Map};
use shared::net::{TickNumber, GameInfo, TimedPlayerInput};
use shared::event::GameEvent;
use shared::player::{PlayerId, PlayerInfo, PlayerInput};
use systems::Systems;

pub struct Player {
    // Has this player been sent its first tick yet?
    pub is_new: bool,

    pub info: PlayerInfo,
    pub next_input: Vec<TimedPlayerInput>,

    pub controlled_entity: Option<net::EntityId>,
    pub respawn_time: Option<f64>, 
}

pub struct SpawnPoint {
    position: math::Vec2,
    size: math::Vec2,
    last_used_time_s: Option<f64>,
}

impl Player {
    fn new(info: PlayerInfo) -> Player {
        assert!(!info.alive);
        Player {
            is_new: true,
            info: info,
            next_input: Vec::new(),
            controlled_entity: None,
            respawn_time: Some(0.0),
        }
    }
}

pub struct GameState {
    pub game_info: GameInfo,
    pub map: Map,
    pub spawn_points: Vec<SpawnPoint>,

    pub world: ecs::World<Systems>, 
    pub tick_number: TickNumber,

    pub time_s: f64,

    players: HashMap<PlayerId, Player>,
}

impl GameState {
    pub fn new(game_info: &GameInfo) -> GameState {
        let map = Map::load(&game_info.map_name).unwrap();

        let spawn_points = map.objects.iter()
               .filter(|object| &object.type_str == "player_spawn")
               .map(|object| SpawnPoint {
                        position: [object.x, object.y],
                        size: [object.width, object.height],
                        last_used_time_s: None,
                    })
               .collect();

        GameState {
            game_info: game_info.clone(),
            map: map,
            spawn_points: spawn_points,
            world: ecs::World::new(),
            tick_number: 0,
            time_s: 0.0,
            players: HashMap::new(),
        }
    }

    fn create_map_objects(&mut self) {
        for object in self.map.objects.iter() {
        }
    }

    // For adding test entities and stuff
    fn init_first_tick(&mut self) {
        self.create_map_objects();

        let num_bouncies = 20;

        for i in 0..num_bouncies {
            let net_entity_type_id = self.world.systems.net_entity_system.type_id("bouncy_enemy".to_string());
            let (_, entity) = 
                self.world.systems.net_entity_system
                    .create_entity(net_entity_type_id, 0, &mut self.world.data);

            // Pick a random non-blocked tile
            let mut rx = 0;
            let mut ry = 0;
            loop {
                rx = rand::random::<usize>() % self.map.width();
                ry = rand::random::<usize>() % self.map.height();

                if self.map.get_tile(LayerId::Block, rx, ry).is_none() {
                    break;
                }
            }

            let position = [(rx * self.map.tile_width()) as f64 + self.map.tile_width() as f64 / 2.0,
                            (ry * self.map.tile_height()) as f64 + self.map.tile_height() as f64 / 2.0];

            self.world.with_entity_data(&entity, |e, c| {
                c.position[e].p = position;
                c.orientation[e].angle = rand::random::<f64>() * f64::consts::PI * 2.0;
            });
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
        let (net_entity_id, entity) = 
            self.world.systems.net_entity_system
                .create_entity(net_entity_type_id, id, &mut self.world.data);
        self.players.get_mut(&id).unwrap().controlled_entity = Some(net_entity_id);

        let position = {
            let spawn_point = &self.spawn_points[rand::random::<usize>() % self.spawn_points.len()];
            [spawn_point.position[0] + rand::random::<f64>() * spawn_point.size[0],
             spawn_point.position[1] + rand::random::<f64>() * spawn_point.size[1]]
        };

        self.world.with_entity_data(&entity, |e, c| {
            c.position[e].p = position;
            c.player_state[e].invulnerable_s = Some(2.5);
        });
    }

    pub fn remove_player(&mut self, id: PlayerId) {
        assert!(self.players.get(&id).is_some());
        self.world.systems.net_entity_system
            .remove_player_entities(id, &mut self.world.data);
        self.players.remove(&id);
    }

    pub fn get_player_info(&self, id: PlayerId) -> &PlayerInfo {
        &self.players[&id].info
    }

    pub fn on_player_input(&mut self,
                           id: PlayerId,
                           input: &TimedPlayerInput) {
        if self.players[&id].next_input.len() > 0 {
            //println!("Already have player input for {}, queuing", id);
        }

        self.players.get_mut(&id).as_mut().unwrap()
            .next_input.push(input.clone());
    }

    pub fn run_player_input(&mut self,
                            player_id: PlayerId,
                            net_entity_id: net::EntityId,
                            input: &TimedPlayerInput) {
        let entity = self.world.systems.net_entity_system.get_entity(net_entity_id);

        self.world.systems.player_movement_system
            .run_player_input(entity, input, &self.map, &mut self.world.data);

        // Tell the player in that their input has been processed.
        // TODO: Should this be done on a level thats finer than ticks?!
        // The following GameEvent will be sent with the next tick the server starts!
        /*self.world.services.add_player_event(player_id,
            GameEvent::CorrectState(input_client_tick));*/
    }

    // For now, the resulting tick data will be written in Services::next_tick
    pub fn tick(&mut self) {
        self.tick_number += 1;
        self.world.services.tick_dur_s = (1.0 / (self.game_info.ticks_per_second as f64)); 
        self.world.services.prepare_for_tick(self.tick_number, self.players.keys().map(|i| *i));

        if self.tick_number == 1 {
            self.init_first_tick();
        }

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
                    assert!(player.controlled_entity.is_none());

                    if let Some(time) = player.respawn_time {
                        let time = time - self.world.services.tick_dur_s;

                        player.respawn_time = if time <= 0.0 {
                            respawn.push(*player_id);
                            player.info.alive = true;

                            None   
                        } else {
                            Some(time)
                        };
                    }

                }
            }
            for player_id in respawn {
                self.spawn_player(player_id); 
            }
        }

        // Flush ecs queue: let all the systems know about any new entities
        self.world.flush_queue();

        // Run input of players
        {
            let mut input = Vec::new();
            for (player_id, player) in self.players.iter() {
                match player.controlled_entity {
                    Some(net_entity_id) => {
                        for player_input in &player.next_input {
                            input.push((*player_id,
                                        net_entity_id,
                                        player_input.clone()));
                        }
                    }
                    _ => {}
                }
            }
            for (player_id, net_entity_id, player_input) in input {
                self.run_player_input(player_id,
                                      net_entity_id,
                                      &player_input);
            }
        }

        for (_, player) in self.players.iter_mut() {
            player.next_input.clear();
        }

        // Let server entities have their time
        self.world.systems.bouncy_enemy_system.tick(&self.map, &mut self.world.data);
        self.world.systems.interaction_system.tick(&mut self.world.data);

        // Process generated events
        // TODO: There might be a subtle problem with orderings here
        for i in 0..self.world.services.next_events.len() {
            match self.world.services.next_events[i].clone() {
                GameEvent::PlayerDied(player_id, cause_player_id) => {
                    if !self.get_player_info(player_id).alive {
                        println!("Killing a dead player! HAH!");
                    } else {
                        let entity_id = {
                            let player = self.players.get_mut(&player_id).unwrap();
                            let entity_id = player.controlled_entity.unwrap();
                            player.info.alive = false;
                            player.controlled_entity = None;
                            player.respawn_time = Some(5.0);
                            entity_id 
                        };

                        // This also tells the clients about the removal:
                        self.world.systems.net_entity_system
                            .remove_entity(entity_id, &mut self.world.data);
                    }
                },
                _ => ()
            };
        }
        self.world.services.next_events.clear();

        self.time_s += self.world.services.tick_dur_s;
    }
}

use std::f64;
use std::collections::HashMap;

use ecs;
use rand;

use shared::math;
use shared::{TickNumber, GameInfo, GameEvent, PlayerId, PlayerInfo};
use shared::map::{LayerId, Map};
use shared::net::TimedPlayerInput;

use systems::Systems;
use services::Services;
use entities;

pub struct Player {
    // Has this player been sent its first tick yet?
    pub is_new: bool,

    pub info: PlayerInfo,
    pub next_input: Vec<TimedPlayerInput>,

    pub controlled_entity: Option<ecs::Entity>,
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

        let services = Services::new(game_info.entity_types.clone());

        GameState {
            game_info: game_info.clone(),
            map: map,
            spawn_points: spawn_points,
            world: ecs::World::with_services(services),
            tick_number: 0,
            time_s: 0.0,
            players: HashMap::new(),
        }
    }

    fn create_map_objects(&mut self) {
        for object in self.map.objects.iter() {
            if &object.type_str == "item_spawn" {
                let entity = entities::build_net("item_spawn", 0, &mut self.world.data);
                self.world.with_entity_data(&entity, |e, c| {
                    c.position[e].p = [object.x, object.y];
                });
            } else if &object.type_str == "player_spawn" {
            } else {
                warn!("Ignoring unknown entity type {} in map", object.type_str);
            }
        }
    }

    // For adding test entities and stuff
    fn init_first_tick(&mut self) {
        self.create_map_objects();

        let num_bouncies = 20;

        for _ in 0..num_bouncies {
            let entity = entities::build_net("bouncy_enemy", 0, &mut self.world.data);

            // Pick a random non-blocked tile
            let mut rx;
            let mut ry;
            loop {
                rx = rand::random::<usize>() % self.map.width();
                ry = rand::random::<usize>() % self.map.height();

                if self.map.get_tile(LayerId::Block, rx, ry).is_none() {
                    break;
                }
            }

            let position = [(rx * self.map.tile_width()) as f64 +
                            self.map.tile_width() as f64 / 2.0,
                            (ry * self.map.tile_height()) as f64 +
                            self.map.tile_height() as f64 / 2.0];

            self.world.with_entity_data(&entity, |e, c| {
                c.position[e].p = position;
                c.orientation[e].angle = rand::random::<f64>() * f64::consts::PI * 2.0;
            });
        }

        self.world.flush_queue();
    }

    pub fn tick_number(&self) -> TickNumber {
        self.tick_number 
    }

    pub fn add_player(&mut self, info: PlayerInfo) {
        let id = info.id;
        assert!(self.players.get(&id).is_none());

        self.players.insert(id, Player::new(info));
    }

    fn spawn_player(&mut self, id: PlayerId) -> ecs::Entity {
        assert!(self.players[&id].controlled_entity.is_none(),
                "Can't spawn a player that is already controlling an entity");
        assert!(!self.players[&id].info.alive);

        let entity = entities::build_net("player", id, &mut self.world.data);

        self.players.get_mut(&id).unwrap().controlled_entity = Some(entity);
        self.players.get_mut(&id).unwrap().info.alive = true;

        let position = {
            let spawn_point = &self.spawn_points[rand::random::<usize>() %
                                                 self.spawn_points.len()];
            [spawn_point.position[0] + rand::random::<f64>() * spawn_point.size[0],
             spawn_point.position[1] + rand::random::<f64>() * spawn_point.size[1]]
        };

        self.world.with_entity_data(&entity, |e, c| {
            c.position[e].p = position;
            c.player_state[e].invulnerable_s = Some(2.5);
        });
        
        entity
    }

    fn process_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::PlayerDied(player_id, _cause_player_id) => {
                info!("Killing player {}", player_id);

                if !self.get_player_info(player_id).alive {
                    info!("Killing a dead player! HAH!");
                } else {
                    let entity = {
                        let player = self.players.get_mut(&player_id).unwrap();
                        let entity = player.controlled_entity.unwrap();

                        player.info.alive = false;
                        player.controlled_entity = None;
                        player.respawn_time = Some(5.0);

                        entity
                    };

                    entities::remove_net(entity, &mut self.world.data);
                }
            },
            _ => (),
        }
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
                            entity: ecs::Entity,
                            input: &TimedPlayerInput) {
        self.world.systems.player_movement_system
            .run_player_input(entity, input, &self.map, &mut self.world.data);
        self.world.systems.player_item_system
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
        self.world.services.tick_dur_s = 1.0 / (self.game_info.ticks_per_second as f64); 
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
                    Some(entity) => {
                        for player_input in &player.next_input {
                            input.push((*player_id, entity, player_input.clone()));
                        }
                    }
                    _ => {}
                }
            }
            for (player_id, entity, player_input) in input {
                self.run_player_input(player_id, entity, &player_input);
            }
        }

        for (_, player) in self.players.iter_mut() {
            player.next_input.clear();
        }

        // Let server entities have their time
        self.world.systems.bouncy_enemy_system.tick(&self.map, &mut self.world.data);
        self.world.systems.item_spawn_system.tick(&mut self.world.data);
        self.world.systems.rotate_system.tick(&mut self.world.data);

        self.world.systems.interaction_system.tick(&mut self.world.data);
        

        // Process generated events
        self.world.flush_queue();

        // TODO: There might be a subtle problem with orderings here
        // (events might be processed in a different order on some clients)
        for i in 0..self.world.services.next_events.len() {
            let event = self.world.services.next_events[i].clone();
            self.process_event(event);
            self.world.flush_queue();
        }
        self.world.services.next_events.clear();

        self.time_s += self.world.services.tick_dur_s;
    }
}

use std::f32;
use std::collections::HashMap;

use ecs;
use rand;
use hprof;

use shared::math;
use shared::{TickNumber, GameInfo, GameEvent, PlayerId, PlayerInfo, Item};
use shared::map::{LayerId, Map};
use shared::net::TimedPlayerInput;

use systems::Systems;
use services::Services;
use entities;

const RESPAWN_TIME_S: f32 = 5.0;

pub struct Player {
    // Has this player been sent its first tick yet?
    is_new: bool,

    // If true, player (and owned entities) will be removed next tick
    remove: bool,

    info: PlayerInfo,
    next_input: Vec<TimedPlayerInput>,

    // Entity controlled by the player, if alive
    entity: Option<ecs::Entity>,

    respawn_time: Option<f32>, 
}

pub struct SpawnPoint {
    position: math::Vec2,
    size: math::Vec2,
    last_used_time_s: Option<f32>,
}

impl Player {
    fn new(info: PlayerInfo) -> Player {
        Player {
            is_new: true,
            remove: false,
            info: info,
            next_input: Vec::new(),
            entity: None,
            respawn_time: Some(0.0),
        }
    }

    fn alive(&self) -> bool {
        self.entity.is_some()
    }
}

pub struct GameState {
    game_info: GameInfo,
    map: Map,
    spawn_points: Vec<SpawnPoint>,
    pub world: ecs::World<Systems>, 
    pub tick_number: TickNumber,
    time_s: f32,
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
                let entity = entities::build_net(&object.type_str, 0, &mut self.world.data);
                self.world.with_entity_data(&entity, |e, c| {
                    c.position[e].p = [object.x, object.y];
                });
            } else if &object.type_str == "bouncy_enemy" {
                let entity = entities::build_net(&object.type_str, 0, &mut self.world.data);
                self.world.with_entity_data(&entity, |e, c| {
                    c.position[e].p = [object.x, object.y];
                    c.orientation[e].angle = rand::random::<f32>() * f32::consts::PI * 2.0;
                });
            } else if &object.type_str == "player_spawn" {
            } else {
                warn!("ignoring unknown entity type {} in map", object.type_str);
            }
        }
    }

    // For adding test entities and stuff
    fn init_first_tick(&mut self) {
        self.create_map_objects();

        /*let num_bouncies = 50;

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
        }*/

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
        assert!(self.players[&id].entity.is_none(),
                "Can't spawn a player that is already controlling an entity");

        let entity = entities::build_net("player", id, &mut self.world.data);

        self.players.get_mut(&id).unwrap().entity = Some(entity);

        // Pick a random spawn point
        let position = {
            let spawn_point = &self.spawn_points[rand::random::<usize>() %
                                                 self.spawn_points.len()];
            [spawn_point.position[0] + rand::random::<f32>() * spawn_point.size[0],
             spawn_point.position[1] + rand::random::<f32>() * spawn_point.size[1]]
        };

        // If we don't have a catcher right now, this player is lucky
        let is_catcher = self.current_catcher() == None; 

        self.world.with_entity_data(&entity, |e, c| {
            c.position[e].p = position;
            c.player_state[e].invulnerable_s = Some(2.5);
            c.player_state[e].is_catcher = is_catcher;

            // We'll equip a gun for now
            c.player_state[e].equip(0, Item::Weapon { charges: 20 }); 
        });

        /*// Spawn a complementary orbit ball
        let orbit_entity = entities::build_net("bouncy_enemy", id, &mut self.world.data);

        self.world.with_entity_data(&orbit_entity, |e, c| {
            c.position[e].p = math::add(position, [10.0, 0.0]);
            c.bouncy_enemy[e].orbit = Some(entity);
        });*/
        
        entity
    }

    pub fn remove_player(&mut self, id: PlayerId) {
        self.players.get_mut(&id).unwrap().remove = true;
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

    fn current_catcher(&mut self) -> Option<PlayerId> {
        for (player_id, player) in self.players.iter() {
            if let Some(entity) = player.entity {
                if self.world.with_entity_data(&entity, |e, c| c.player_state[e].is_catcher)
                       .unwrap() {
                    return Some(*player_id);
                }
            }
        }
        return None;
    }

    fn run_player_input(&mut self,
                        player_id: PlayerId,
                        entity: ecs::Entity,
                        input: &TimedPlayerInput) {
        self.world.systems.player_movement_system
            .run_player_input(entity, input, &self.map, &mut self.world.data);
        self.world.systems.player_item_system
            .run_player_input(entity, input, &self.map, &mut self.world.data);
    }

    /// Advances the state of the server by one tick.
    /// Events generated during the tick are stored for each player separately in the services.
    pub fn tick(&mut self) {
        self.check_integrity();

        self.tick_number += 1;
        self.world.services.tick_dur_s = 1.0 / (self.game_info.ticks_per_second as f32); 
        self.world.services.prepare_for_tick(self.tick_number, self.players.keys().map(|i| *i));

        self.tick_replicate_entities_to_new_players();
        if self.tick_number == 1 { self.init_first_tick(); }
        self.tick_spawn_player_entities_if_needed();
        self.tick_remove_disconnected_players();
        self.tick_run_player_input();

        // Let all the systems know about any new/removed ecs entities
        self.world.flush_queue();

        // Let server entities have their time
        {
            let _g = hprof::enter("entities");

            self.world.systems.bouncy_enemy_system.tick(&self.map, &mut self.world.data);
            self.world.systems.projectile_system.tick(&self.map, &mut self.world.data);
            self.world.systems.item_spawn_system.tick(&mut self.world.data);
            self.world.systems.rotate_system.tick(&mut self.world.data);
            self.world.systems.interaction_system.tick(&mut self.world.data);
        }
        
        // Process events generated in this tick
        self.world.flush_queue();

        for i in 0..self.world.services.next_events.len() {
            // TODO: There might be a subtle problem with orderings here
            // (events might be processed in a different order on some clients)

            let event = self.world.services.next_events[i].clone();
            self.tick_process_event(event);
            self.world.flush_queue();
        }
        self.world.services.next_events.clear();

        self.time_s += self.world.services.tick_dur_s;
    }

    fn tick_replicate_entities_to_new_players(&mut self) {
        let mut new_players = Vec::new();
        for (player_id, player) in self.players.iter_mut() {
            if player.is_new {
                info!("replicating net state to player {}", player_id);
                new_players.push(*player_id);
                player.is_new = false;
            }
        }
        for player_id in new_players {
            self.world.systems.net_entity_system
                .replicate_entities(player_id, &mut self.world.data);
        }
    }

    fn tick_spawn_player_entities_if_needed(&mut self) {
        let mut respawn = Vec::new();
        for (player_id, player) in self.players.iter_mut() {
            if !player.alive() {
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

    fn tick_remove_disconnected_players(&mut self) {
        let mut remove = Vec::new();
        for (&player_id, player) in self.players.iter_mut() {
            if player.remove {
                info!("removing player {}", player_id);
                remove.push(player_id);
            }
        }

        for &id in remove.iter() {
            let is_catcher = if let Some(entity) = self.players[&id].entity {
                self.world.with_entity_data(&entity, |e, c| {
                    c.player_state[e].is_catcher
                }).unwrap()
            } else {
                false
            };

            self.world.systems.net_entity_system
                .remove_player_entities(id, &mut self.world.data);
            self.players.remove(&id); 

            // If the disconnected player was the catcher, choose a random new one
            let alive_players = self.players.iter()
                                    .filter(|&(&other_id, player)| player.alive())
                                    .map(|(&id, _)| id)
                                    .collect::<Vec<_>>();
            if !alive_players.is_empty() {
                let chosen_one = alive_players[rand::random::<usize>() % alive_players.len()];

                self.world.with_entity_data(&self.players[&chosen_one].entity.unwrap(), |e, c| {
                    assert!(!c.player_state[e].is_catcher);
                    c.player_state[e].is_catcher = true;
                });
            }
        }

    }

    fn tick_run_player_input(&mut self) {
        let mut input = Vec::new();
        for (player_id, player) in self.players.iter() {
            if let Some(entity) = player.entity {
                for player_input in &player.next_input {
                    input.push((*player_id, entity, player_input.clone()));
                }
            }
        }

        for (player_id, entity, player_input) in input {
            self.run_player_input(player_id, entity, &player_input);
        }

        for (_, player) in self.players.iter_mut() {
            player.next_input.clear();
        }
    }

    fn tick_process_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::PlayerDied {
                player_id,
                position,
                responsible_player_id,
            } => {
                info!("killing player {}", player_id);

                if !self.players[&player_id].alive() {
                    info!("killing a dead player! HAH!");
                } else {
                    let player_entity = self.players[&player_id].entity.unwrap();

                    // If this player is the catcher, we need to determine a new catcher
                    let is_catcher = self.world.with_entity_data(&player_entity, |e, c| {
                        let is_catcher = c.player_state[e].is_catcher;
                        c.player_state[e].is_catcher = false;
                        is_catcher
                    }).unwrap();

                    if is_catcher {
                        let responsible_entity = 
                            self.players.get(&responsible_player_id).map(|player| player.entity);

                        if let Some(Some(responsible_entity)) = responsible_entity {
                            // If we were killed by another player, that one becomes the catcher
                            self.world.with_entity_data(&responsible_entity, |e, c| {
                                c.player_state[e].is_catcher = true;
                            });
                        } else {
                            // Otherwise, find the player that is the closest to the dead catcher
                            let player_ids = self.players.keys().filter(|id| **id != player_id);

                            let mut closest: Option<(ecs::Entity, f32)> = None; 
                            for &id in player_ids {
                                if let Some(entity) = self.players[&id].entity {
                                    let d = self.world.with_entity_data(&entity, |e, c| {
                                        math::square_len(math::sub(position,
                                                                   c.position[e].p)).sqrt()
                                    }).unwrap();
                                    if closest.is_none() || closest.unwrap().1 > d {
                                        closest = Some((entity, d));
                                    }
                                }
                            }
                            
                            if let Some((closest_player_entity, _)) = closest {
                                self.world.with_entity_data(&closest_player_entity, |e, c| {
                                    assert!(!c.player_state[e].is_catcher);
                                    c.player_state[e].is_catcher = true;
                                });
                            } else {
                                // If we are here, this should mean that nobody is alive
                                for (id, player) in self.players.iter() {
                                    assert!(*id == player_id || player.entity.is_none());
                                }
                            }
                        }
                    }

                    // Kill the player
                    {
                        let player = self.players.get_mut(&player_id).unwrap();
                        player.entity = None;
                        player.respawn_time = Some(RESPAWN_TIME_S);
                    };

                    entities::remove_net(player_entity, &mut self.world.data);
                }
            },
            _ => (),
        }
    }

    fn check_integrity(&mut self) {
        // When we have at least one player that is alive, there should be exactly one catcher
        let mut num_alive = 0;
        let mut num_catchers = 0;

        for (_, player) in self.players.iter() {
            if let Some(entity) = player.entity {
                num_alive += 1;
                self.world.with_entity_data(&entity, |e, c| {
                    if c.player_state[e].is_catcher {
                        num_catchers += 1;
                    }
                });
            }
        }

        if num_alive > 0 {
            assert!(num_catchers == 1, "There should be exactly one catcher!");
        }
    }
}

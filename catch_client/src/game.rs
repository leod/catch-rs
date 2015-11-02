use std::f32;
use std::thread;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;

use ecs;
use rand;
use time;
use hprof;
use na::{Vec2, Norm};

use glium::glutin;
use glium::{Display, Surface};
use glium_text::{TextSystem, FontTexture, TextDisplay};

use shared::NUM_ITEM_SLOTS;
use shared::{Item, GameEvent, PlayerId};
use shared::net::{ClientMessage, TimedPlayerInput};
use shared::tick::Tick;

use client::Client;
use state::GameState;
use player_input::{PlayerInput, InputMap};
use draw_map::DrawMap;
use particles::Particles;
use sounds::Sounds;

pub struct Game {
    quit: bool,

    client: Client,
    game_state: GameState,

    player_input_map: InputMap,
    player_input: PlayerInput,

    interpolation_ticks: usize,
    current_tick: Option<Tick>,
    tick_progress: f32,
    time_factor: f32,

    display: Display,

    draw_map: DrawMap,
    particles: Particles,
    sounds: Sounds,

    /*text_system: TextSystem,
    font: FontTexture,*/

    cam_pos: Vec2<f32>,
    fps: f32,

    print_prof: bool,
}

impl Game {
    // The given client is expected to be connected already
    pub fn new(connected_client: Client,
               player_input_map: InputMap,
               display: Display) -> Game {
        let game_state = GameState::new(connected_client.my_id(),
                                        connected_client.game_info());
        let draw_map = DrawMap::load(&game_state.map).unwrap();

        let sounds = Sounds::load().unwrap();

        Game {
            quit: false,

            client: connected_client,

            game_state: game_state,

            player_input_map: player_input_map,
            player_input: PlayerInput::new(),

            interpolation_ticks: 2,
            current_tick: None,
            tick_progress: 0.0,
            time_factor: 0.0,

            display: display,

            draw_map: draw_map,
            particles: Particles::new(),
            sounds: sounds,

            cam_pos: Vec2::new(0.0, 0.0),
            fps: 0.0,

            print_prof: false,
        }
    }

    pub fn run(&mut self) {
        self.wait_first_ticks();

        let mut simulation_time_s = 0.0;
        let mut frame_start_s = time::precise_time_s() as f32;
        while !self.quit {
            hprof::start_frame();

            self.client_service();
            self.read_input();
            self.send_input(simulation_time_s);
            self.manage_ticks(simulation_time_s);
            self.interpolate();
            self.draw(simulation_time_s);

            self.fps = 1.0 / simulation_time_s;
            self.display.get_window().map(|w| w.set_title(&format!("{}", self.fps as usize)));

            hprof::end_frame();

            if self.print_prof {
                hprof::profiler().print_timing();
                self.print_prof = false;
            }

            let new_frame_start_s = time::precise_time_s() as f32;
            simulation_time_s = new_frame_start_s - frame_start_s;
            frame_start_s = new_frame_start_s;
        }
    }

    fn wait_first_ticks(&mut self) {
        info!("waiting to receive first ticks from server... ");

        while self.client.num_ticks() < self.interpolation_ticks {
            self.client_service();
        }

        info!("done! have {} ticks", self.client.num_ticks());

        while self.client.num_ticks() >=2 { // catch up
            debug!("starting initial tick {}", self.client.get_next_tick().1.tick_number);
            self.start_tick();
        }
    }

    /// Starts the next tick in the queue, loading its state and running its events.
    /// The function assumes that we have at least 2 ticks queued, so that we can interpolate.
    fn start_tick(&mut self) {
        let _g = hprof::enter("start tick");

        assert!(self.client.num_ticks() >= 2);

        let tick = self.client.pop_next_tick().1;

        {
            let _g = hprof::enter("events");

            for event in tick.events.iter() {
                debug!("tick {}: {:?}", tick.tick_number, event);
                self.process_game_event(event);
            }
        }

        let next_tick = &self.client.get_next_tick().1;

        self.game_state.run_tick(&tick);
        self.game_state.load_interp_tick_state(&tick, next_tick);
        self.current_tick = Some(tick);
    }

    fn client_service(&mut self) {
        let _g = hprof::enter("client service");

        loop {
            match self.client.service().unwrap() {
                Some(_) => continue,
                None => break
            };
        }
    }

    fn read_input(&mut self) {
        let _g = hprof::enter("read input");

        for event in self.display.poll_events() {
            match event {
                glutin::Event::KeyboardInput(state, _, Some(key)) => {
                    if state == glutin::ElementState::Pressed {
                        if key == glutin::VirtualKeyCode::Escape {
                            info!("got escape input, quitting game");
                            self.quit = true;
                            return;
                        } else if key == glutin::VirtualKeyCode::L {
                            thread::sleep_ms(200);
                            continue;
                        } else if key == glutin::VirtualKeyCode::P {
                            self.print_prof = true;
                            continue;
                        }
                    }

                    self.player_input_map.update_player_input(state, key, &mut self.player_input);
                }
                _ => (),
            };
        }
    }

    fn send_input(&mut self, simulation_time_s: f32) {
        let _g = hprof::enter("send input");

        self.client.send(&ClientMessage::PlayerInput(
            TimedPlayerInput {
                duration_s: simulation_time_s,
                input: self.player_input.clone(),
            }
        ));
    }

    fn manage_ticks(&mut self, simulation_time_s: f32) {
        let _g = hprof::enter("manage ticks");

        assert!(self.current_tick.is_some());

        if self.tick_progress < 1.0 {
            self.time_factor = {
                if self.client.num_ticks() > 2 {
                    debug!("speeding up playback (num queued ticks: {}, progress: {})",
                           self.client.num_ticks(), self.tick_progress);

                    1.25 
                } else if self.client.num_ticks() < 2 && self.tick_progress > 0.5 {
                    debug!("slowing down tick playback (num queued ticks: {}, progress: {})",
                           self.client.num_ticks(), self.tick_progress);
                    0.75 // Is this a stupid idea?
                } else {
                    1.0
                }
            };

            self.tick_progress += self.time_factor * 
                                  simulation_time_s *
                                  self.client.game_info().ticks_per_second as f32;
        }

        if self.tick_progress >= 1.0 {
            // Load the next tick state if we can interpolate into the following tick
            if self.client.num_ticks() >= 2 {
                self.start_tick();
                self.tick_progress -= 1.0;
            } else {
                debug!("waiting to receive next tick (num queued ticks: {})",
                       self.client.num_ticks());
            }
        }
    }

    /// Produce graphics such as particles and audio from game events
    fn process_game_event(&mut self, event: &GameEvent) {
        match event {
            &GameEvent::PlayerDied {
                player_id,
                position,
                responsible_player_id: _
            } => {
                let entity = self.get_player_entity(player_id).unwrap();
                let color = self.game_state.world.with_entity_data(&entity, |e, c| {
                    [c.draw_player[e].color[0],
                     c.draw_player[e].color[1],
                     c.draw_player[e].color[2]]
                }).unwrap();

                let num = 100;
                for _ in 0..num {
                    self.particles.spawn_cone(0.6, color, color, 3.5 * rand::random::<f32>() + 2.0,
                                              position, 0.0, f32::consts::PI * 2.0,
                                              70.0 + rand::random::<f32>() * 40.0,
                                              rand::random::<f32>() * 8.0, 1.0);
                }
            }
            &GameEvent::PlayerDash {
                player_id: _,
                position,
                orientation: _,
            } => {
                self.sounds.play("dash", position);
            }
            &GameEvent::PlayerFlip {
                player_id: _,
                position,
                orientation: _,
                speed,
                orientation_wall,
            } => {
                let num = (3.0 * speed.sqrt()) as usize;
                for _ in 0..num {
                    self.particles.spawn_cone(0.5,
                                              [0.0, 0.0, 0.0],
                                              [0.0, 0.0, 0.0],
                                              1.5,
                                              position,
                                              orientation_wall,
                                              f32::consts::PI,
                                              20.0 + rand::random::<f32>() * 20.0,
                                              0.0,
                                              1.0);
                }
            }
            &GameEvent::PlayerTakeItem {
                player_id: _,
                position,
            } => {
                self.sounds.play("take_item", position);

                let num = 100;
                let color = [0.05, 0.5, 1.0];
                for _ in 0..num {
                    self.particles.spawn_cone(0.4, color, color, 1.5, position, 0.0,
                                              f32::consts::PI * 2.0,
                                              200.0 + rand::random::<f32>() * 20.0, 0.0, 1.0);
                }
            }
            &GameEvent::PlayerEquipItem {
                player_id: _,
                position,
                item: _,
            } => {
                self.sounds.play("equip_item", position);

                /*let num = 100;
                let color = [0.05, 0.5, 1.0];
                for i in 0..num {
                    self.particles.spawn_cone(0.4, color, color, 1.5, position, 0.0,
                                              f32::consts::PI * 2.0,
                                              200.0 + rand::random::<f32>() * 20.0, 0.0, 1.0);
                }*/
            }
            &GameEvent::EnemyDied {
                position,
            } => {
                let num = 100;
                let color = [1.0, 0.0, 0.0];
                for _ in 0..num {
                    self.particles.spawn_cone(0.5, color, color, 2.5 * rand::random::<f32>() + 1.0,
                                              position, 0.0, f32::consts::PI * 2.0,
                                              70.0 + rand::random::<f32>() * 20.0,
                                              rand::random::<f32>() * 5.0, 1.0);
                }
            }
            &GameEvent::ProjectileImpact {
                position,
            } => {
                let num = 30;
                let color = [0.3, 0.3, 0.3];
                for _ in 0..num {
                    self.particles.spawn_cone(0.25, color, color,
                                              1.0 * rand::random::<f32>() + 0.5, position, 0.0,
                                              f32::consts::PI * 2.0,
                                              30.0 + rand::random::<f32>() * 15.0,
                                              rand::random::<f32>() * 5.0, 1.0);
                }
            }
            _ => ()
        };
    }

    fn interpolate(&mut self) {
        let _g = hprof::enter("interpolate");

        let t = if self.tick_progress >= 1.0 { 1.0 } else { self.tick_progress };

        self.game_state.world.systems.interpolation_system
            .interpolate(t, &mut self.game_state.world.data);
    }

    fn draw(&mut self, simulation_time_s: f32) {
        let _g = hprof::enter("draw");

        let mut target = self.display.draw();
        target.clear_color(0.3, 0.3, 0.3, 1.0);

        target.finish().unwrap();

        /*gl.draw(viewport, |c, gl| {
            graphics::clear([0.3, 0.3, 0.3, 0.0], gl);

            let pos = self.get_my_player_position().unwrap_or(self.cam_pos);
            self.cam_pos = math::add(self.cam_pos,
                                     math::scale(math::sub(pos, self.cam_pos),
                                     0.15));

            let half_width = draw_width as f32 / 2.0;
            let half_height = draw_height as f32 / 2.0;
            let zoom = 3.0;

            // Clip camera position to map size in pixels
            if self.cam_pos[0] < half_width / zoom {
                self.cam_pos[0] = half_width / zoom; 
            } else if self.cam_pos[0] + half_width / zoom >
                      self.game_state.map.width_pixels() as f32 {
                self.cam_pos[0] = self.game_state.map.width_pixels() as f32 - half_width / zoom;
            }
            if self.cam_pos[1] < half_height / zoom {
                self.cam_pos[1] = half_height / zoom; 
            } else if self.cam_pos[1] + half_height / zoom >
                      self.game_state.map.height_pixels() as f32 {
                self.cam_pos[1] = self.game_state.map.height_pixels() as f32 - half_height / zoom;
            }

            {
                let c = c.trans(half_width as f64, half_height as f64)
                         .zoom(zoom as f64)
                         .trans(-self.cam_pos[0] as f64, -self.cam_pos[1] as f64);

                /*// What part of the map is visible?
                let cam_tx_min = ((self.cam_pos[0]*zoom - half_width) /
                                   (zoom * self.game_state.map.tile_width() as f32))
                                 .floor() as isize;
                let cam_ty_min = ((self.cam_pos[1]*zoom - half_height) /
                                   (zoom * self.game_state.map.tile_height() as f32))
                                 .floor() as isize;
                let cam_tx_size = (draw_width as f32 /
                                   (zoom * self.game_state.map.tile_width() as f32))
                                  .ceil() as isize;
                let cam_ty_size = (draw_height as f32 /
                                   (zoom * self.game_state.map.tile_height() as f32))
                                  .ceil() as isize;*/

                {
                    let _g = hprof::enter("map");

                    self.draw_map.draw(&self.game_state.map, c, gl);
                }
                {
                    let _g = hprof::enter("entities");

                    self.game_state.world.systems.draw_wall_system
                        .draw(&mut self.game_state.world.data, simulation_time_s,
                              &mut self.particles, c, gl);
                    self.game_state.world.systems.draw_item_system
                        .draw(&mut self.game_state.world.data, simulation_time_s,
                              &mut self.particles, c, gl);
                    self.game_state.world.systems.draw_projectile_system
                        .draw(&mut self.game_state.world.data, simulation_time_s,
                              &mut self.particles, c, gl);
                    self.game_state.world.systems.draw_player_system
                        .draw(&mut self.game_state.world.data, simulation_time_s,
                              &mut self.particles, c, gl);
                    self.game_state.world.systems.draw_bouncy_enemy_system
                        .draw(&mut self.game_state.world.data, c, gl);
                }
                {
                    let _g = hprof::enter("update particles");
                    self.particles.update(simulation_time_s);
                }
                {
                    let _g = hprof::enter("draw particles");
                    self.particles.draw(c, gl);
                }
            }

            let _g = hprof::enter("text");
            self.draw_player_text(c, gl);
            self.draw_debug_text(c, gl);
        });

        let _g = hprof::enter("swap");
        self.window.swap_buffers();*/
    }

    fn draw_debug_text<S: Surface>(&mut self, target: &mut S) {
        let color = [1.0, 0.0, 1.0, 1.0];

        let s = &format!("fps: {:.1}", self.fps);
        self.draw_text(color, 10.0, 30.0, s, target);

        let s = &format!("# queued ticks: {}", self.client.num_ticks());
        self.draw_text(color, 10.0, 65.0, s, target);

        let s = &format!("tick progress: {:.1}", self.tick_progress);
        self.draw_text(color, 10.0, 100.0, s, target);

        let s = &format!("time factor: {:.1}", self.time_factor);
        self.draw_text(color, 10.0, 135.0, s, target);

        let s = &format!("num particles: {}", self.particles.num());
        self.draw_text(color, 10.0, 170.0, s, target);

        if let Some(entity) = self.get_my_player_entity() {
            let speed =
                self.game_state.world.with_entity_data(&entity, |e, c| {
                    c.linear_velocity[e].v.norm()
                }).unwrap();

            let s = &format!("player speed: {:.1}", speed);
            self.draw_text(color, 10.0, 205.0, s, target);
        }
    }

    fn draw_player_text<S: Surface>(&mut self, target: &mut S) {
        if let Some(entity) = self.get_my_player_entity() {
            let (dash_cooldown_s, hidden_item, player_state) =
                self.game_state.world.with_entity_data(&entity, |e, c| {
                    (c.full_player_state[e].dash_cooldown_s,
                     c.full_player_state[e].hidden_item.clone(),
                     c.player_state[e].clone())
                }).unwrap();

            let y1 = 900.0; //self.window.draw_size().height as f32 - 100.0;
            let y2 = y1 + 35.0; 
            let y3 = y2 + 35.0;
            let color1 = [0.0, 0.0, 1.0, 1.0];
            let color2 = [0.3, 0.3, 0.3, 1.0];

            self.draw_text(if dash_cooldown_s.is_none() { color1 } else { color2 },
                           20.0, y1, "dash", target);
            if let Some(t) = dash_cooldown_s {
                self.draw_text(color1, 25.0, y2, &format!("{:.1}", t), target);
            }

            let slot_names = vec!["Q", "W", "E"]; // TODO
            let mut cursor_x = 200.0;

            for (item_slot, slot_name) in (0..NUM_ITEM_SLOTS).zip(slot_names.iter()) {
                cursor_x += 180.0;

                if let Some(equipped_item) = player_state.get_item(item_slot) {
                    let color = if equipped_item.cooldown_s.is_none() { color1 } else { color2 };

                    self.draw_text(color, cursor_x, y1, slot_name, target);

                    let text = &self.item_text(&equipped_item.item);
                    self.draw_text(color, cursor_x, y2, &text, target);

                    if let Some(t) = equipped_item.cooldown_s {
                        self.draw_text(color1, cursor_x + 5.0, y3, &format!("{:.1}", t),
                                       target);
                    }
                } else {
                    self.draw_text(color2, cursor_x, y1, slot_name, target);
                }
            }

            if let Some(item) = hidden_item {
                let text = self.item_text(&item);
                self.draw_text(color1, 200.0, y1, "item", target);
                self.draw_text(color1, 200.0, y2, &text, target);
            } else {
                self.draw_text(color2, 200.0, y1, "item", target);
            }
        }
    }

    fn draw_text<S: Surface>(&mut self, color: [f32; 4], x: f32, y: f32, s: &str,
                             target: &mut S) {
    }

    fn item_text(&self, item: &Item) -> String {
        match item {
            &Item::Weapon { charges } =>
                format!("weapon {}", charges),
            &Item::SpeedBoost { duration_s: _ } =>
                format!("speed boost"),
            &Item::BlockPlacer { charges: _ } =>
                format!("block placer"),
            &Item::BallSpawner { charges: _ } =>
                format!("ball spawner"),
        }
    }

    fn get_player_entity(&mut self, player_id: PlayerId) -> Option<ecs::Entity> {
        self.game_state.world.systems.net_entity_system.inner
            .as_ref().unwrap()
            .get_player_entity(player_id)
    }
    
    fn get_my_player_entity(&mut self) -> Option<ecs::Entity> {
        self.game_state.world.systems.net_entity_system.inner
            .as_ref().unwrap()
            .get_my_player_entity()
    }

    fn get_my_player_position(&mut self) -> Option<Vec2<f32>> {
        self.get_my_player_entity().map(|entity| {
            self.game_state.world.with_entity_data(&entity, |e, c| {
                c.position[e].p
            }).unwrap()
        })
    }
}


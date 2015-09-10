use std::thread;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;

use time;
use graphics;
use graphics::Transformed;
use graphics::Viewport;
use glutin_window::GlutinWindow;
use piston_window::{PistonWindow, Text};
use piston::window::Window;
use piston::input::{Input, Button, Key};
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;

use shared::math;
use shared::net::{ClientMessage, TickNumber, TimedPlayerInput};
use shared::map::Map;
use shared::tick::Tick;
use shared::util::PeriodicTimer;

use client::Client;
use state::GameState;
use player_input::{PlayerInput, InputMap};
use draw_map::DrawMap;

type GameWindow = PistonWindow;

pub struct Game {
    quit: bool,

    client: Client,

    game_state: GameState,

    player_input_map: InputMap,
    player_input: PlayerInput,

    interpolation_ticks: usize,
    current_tick: Option<Tick>,
    tick_progress: f64,
    time_factor: f64,

    window: GameWindow,
    draw_map: DrawMap,

    cam_pos: math::Vec2,
    glyphs: GlyphCache<'static>,
    fps: f64,
}

impl Game {
    // The given client is expected to be connected already
    pub fn new(connected_client: Client,
               player_input_map: InputMap,
               window: GlutinWindow) -> Game {
        let game_state = GameState::new(connected_client.get_my_id(),
                                        connected_client.get_game_info());
        let draw_map = DrawMap::load(&game_state.map).unwrap();
        let window = GameWindow::new(Rc::new(RefCell::new(window)),
                                     Rc::new(RefCell::new(())));
        let font = "../data/ProggyClean.ttf";
        let glyphs = GlyphCache::new(Path::new(font)).unwrap();

        Game {
            quit: false,
            client: connected_client,
            player_input_map: player_input_map,
            player_input: PlayerInput::new(),
            interpolation_ticks: 2,
            current_tick: None,
            tick_progress: 0.0,
            time_factor: 0.0,
            game_state: game_state,
            window: window,
            draw_map: draw_map,
            cam_pos: [0.0, 0.0],
            glyphs: glyphs,
            fps: 0.0,
        }
    }

    pub fn run(&mut self, gl: &mut GlGraphics) {
        let mut simulation_time_s = 0.0;

        self.wait_first_ticks();

        while !self.quit {
            let frame_start_s = time::precise_time_s();

            self.client_service();
            self.read_input();
            self.send_input(simulation_time_s);
            self.manage_ticks(simulation_time_s);
            self.interpolate();
            self.draw(gl);

            //thread::sleep_ms(10);

            let frame_end_s = time::precise_time_s();
            simulation_time_s = frame_end_s - frame_start_s;

            self.fps = 1.0 / simulation_time_s;
        }
    }

    fn wait_first_ticks(&mut self) {
        print!("Waiting to receive first ticks from server... ");

        while self.client.num_ticks() < self.interpolation_ticks {
            self.client_service();
        }

        println!("done! Have {} ticks", self.client.num_ticks());

        let tick = self.client.pop_next_tick().1;
        println!("Starting tick {}", tick.tick_number);
        self.game_state.run_tick(&tick);

        let next_tick = &self.client.get_next_tick().1;
        println!("Interpolating to tick {}", next_tick.tick_number);
        self.game_state.load_interp_tick_state(&tick, next_tick);

        assert!(self.current_tick.is_none());
        self.current_tick = Some(tick);
    }

    fn client_service(&mut self) {
        loop {
            match self.client.service().unwrap() {
                Some(_) => continue,
                None => break
            };
        }
    }

    fn read_input(&mut self) {
        while let Some(input) = (&mut self.window as &mut Window<Event=Input>).poll_event() {
            match input {
                Input::Press(Button::Keyboard(Key::Escape)) => {
                    self.quit = true;
                    return;
                }
                Input::Press(Button::Keyboard(Key::P)) => {
                    thread::sleep_ms(200);
                }
                _ => 
                    self.player_input_map
                        .update_player_input(&input, &mut self.player_input)
            };
        }
    }

    fn send_input(&mut self, simulation_time_s: f64) {
        self.client.send(&ClientMessage::PlayerInput(
            TimedPlayerInput {
                duration_s: simulation_time_s,
                input: self.player_input.clone(),
            }
        ));
    }

    fn manage_ticks(&mut self, simulation_time_s: f64) {
        assert!(self.current_tick.is_some());

        if self.tick_progress < 1.0 {
            self.time_factor = {
                if self.client.num_ticks() > 2 {
                    1.25 
                } else if self.client.num_ticks() < 2 && self.tick_progress > 0.5 {
                    println!("Slowing down");
                    0.75 // Is this a stupid idea?
                } else {
                    1.0
                }
            };

            self.tick_progress += self.time_factor * 
                                  simulation_time_s *
                                  self.client.get_game_info().ticks_per_second as f64;
        }

        if self.tick_progress >= 1.0 {
            // Load the next ticks if we have them
            if self.client.num_ticks() >= 2 {
                let (_, tick) = self.client.pop_next_tick();
                self.game_state.run_tick(&tick);

                let next_tick = &self.client.get_next_tick().1;

                self.game_state.load_interp_tick_state(&tick, next_tick);

                self.current_tick = Some(tick);

                self.tick_progress -= 1.0;
            } else {
                println!("Waiting for new tick");
            }
        }
    }

    fn interpolate(&mut self) {
        self.game_state.world.systems.interpolation_system
            .interpolate(self.tick_progress, &mut self.game_state.world.data);
    }

    fn draw(&mut self, gl: &mut GlGraphics) {
        let draw_width = self.window.draw_size().width;
        let draw_height = self.window.draw_size().height;

        let viewport = Viewport {
            rect: [0, 0, draw_width as i32, draw_height as i32],
            draw_size: [draw_width, draw_height],
            window_size: [self.window.size().width,
                          self.window.size().height]
        };

        gl.draw(viewport, |c, gl| {
            graphics::clear([0.0, 0.0, 0.0, 0.0], gl);

            let pos = self.my_player_position().unwrap_or([0.0, 0.0]);
            self.cam_pos = math::add(self.cam_pos,
                                     math::scale(math::sub(pos, self.cam_pos),
                                     0.15));

            let half_width = draw_width as f64 / 2.0;
            let half_height = draw_height as f64 / 2.0;
            let zoom = 2.0;

            // Clip camera position to map size in pixels
            if self.cam_pos[0] < half_width / zoom {
                self.cam_pos[0] = half_width / zoom; 
            }
            if self.cam_pos[0] + half_width / zoom > self.game_state.map.width_pixels() as f64 {
                self.cam_pos[0] = self.game_state.map.width_pixels() as f64 - half_width / zoom;
            }
            if self.cam_pos[1] < half_height / zoom {
                self.cam_pos[1] = half_height / zoom; 
            }
            if self.cam_pos[1] + half_height / zoom > self.game_state.map.height_pixels() as f64{
                self.cam_pos[1] = self.game_state.map.height_pixels() as f64 - half_height / zoom;
            }

            {
                let c = c.trans(half_width, half_height)
                         .zoom(zoom)
                         .trans(-self.cam_pos[0], -self.cam_pos[1]);

                self.draw_map.draw(&self.game_state.map, c, gl);
                self.game_state.world.systems
                    .draw_player_system
                    .draw(&mut self.game_state.world.data, c, gl);
            }

            self.debug_text(c, gl);
        });

        self.window.swap_buffers();
    }

    fn debug_text(&mut self, c: graphics::Context, gl: &mut GlGraphics) {
        let s = &format!("fps: {:.1}", self.fps); self.draw_text(10.0, 30.0, s, c, gl);
        let s = &format!("# queued ticks: {}", self.client.num_ticks()); self.draw_text(10.0, 65.0, s, c, gl);
        let s = &format!("tick progress: {:.1}", self.tick_progress); self.draw_text(10.0, 100.0, s, c, gl);
        let s = &format!("time factor: {:.1}", self.time_factor); self.draw_text(10.0, 135.0, s, c, gl);
    }

    fn draw_text(&mut self, x: f64, y: f64, s: &str, c: graphics::Context, gl: &mut GlGraphics) {
        let color = [1.0, 0.0, 1.0, 1.0];
        Text::new_color(color, 30).draw(
            s,
            &mut self.glyphs,
            &c.draw_state,
            c.transform.trans(x, y),
            gl);
    }

    fn my_player_position(&mut self) -> Option<math::Vec2> {
        match self.game_state.world.systems.net_entity_system.inner
                                   .as_ref().unwrap()
                                   .my_player_entity_id() {
            Some(player_entity_id) => {
                let player_entity = self.game_state.world.systems
                    .net_entity_system.inner.as_ref().unwrap()
                    .get_entity(player_entity_id)
                    .unwrap();

                self.game_state.world.with_entity_data(&player_entity, |e, c| {
                    c.position[e].p
                })
            }
            None => None
        }
    }
}


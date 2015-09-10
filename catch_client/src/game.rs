use std::rc::Rc;
use std::cell::RefCell;
use std::thread;

use time;
use graphics;
use graphics::Transformed;
use graphics::Viewport;
use glutin_window::GlutinWindow;
use piston_window::PistonWindow;
use piston::window::Window;
use piston::input::{Input, Button, Key};
use opengl_graphics::GlGraphics;

use shared::math;
use shared::net::{ClientMessage, TickNumber, TimedPlayerInput};
use shared::map::Map;
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

    tick_number: Option<TickNumber>,

    window: GameWindow,
    draw_map: DrawMap,

    cam_pos: math::Vec2,
}

impl Game {
    // The given client is expected to be connected already
    pub fn new(connected_client: Client,
               player_input_map: InputMap,
               window: GlutinWindow) -> Game {
        let game_state = GameState::new(connected_client.get_my_id(),
                                        connected_client.get_game_info());
        let draw_map = DrawMap::load(&game_state.map).unwrap();

        Game {
            quit: false,
            client: connected_client,
            player_input_map: player_input_map,
            player_input: PlayerInput::new(),
            tick_number: None,
            game_state: game_state,
            window: GameWindow::new(Rc::new(RefCell::new(window)),
                                    Rc::new(RefCell::new(()))),
            draw_map: draw_map,
            cam_pos: [0.0, 0.0]
        }
    }

    pub fn run(&mut self, gl: &mut GlGraphics) {
        let mut simulation_time_s = 0.0;

        while !self.quit {
            let frame_start_s = time::precise_time_s();

            self.client_service();
            self.read_input();
            self.send_input(simulation_time_s);
            self.manage_ticks();
            self.draw(gl);

            //thread::sleep_ms(10);

            let frame_end_s = time::precise_time_s();
            simulation_time_s = frame_end_s - frame_start_s;
        }
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

    fn manage_ticks(&mut self) {
        if self.client.num_ticks() > 0 {
            // Play some very rough catch up for a start...
            // For the future, the idea here is to increase the playback speed of the
            // received ticks if we notice that we are falling behind too much.
            // We only need to make sure we know what "too much" is, and if it is
            // sufficient to query client.num_ticks() for that.
            while self.client.num_ticks() > 0 {
                let (time_recv, tick) = self.client.pop_next_tick();

                println!("Starting tick {}, {:?} delay, {} ticks queued",
                         tick.tick_number,
                         (time::get_time() - time_recv).num_milliseconds(),
                         self.client.num_ticks());

                self.game_state.run_tick(&tick);
                self.tick_number = Some(tick.tick_number);

                self.client.send(&ClientMessage::StartingTick {
                    tick: tick.tick_number
                });
            }
        }
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
            graphics::clear([1.0, 0.0, 0.0, 0.0], gl);

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

            let c = c.trans(half_width,
                            half_height)
                     .zoom(zoom)
                     .trans(-self.cam_pos[0], -self.cam_pos[1]);

            self.draw_map.draw(&self.game_state.map, c, gl);
            self.game_state.world.systems
                .draw_player_system
                .draw(&mut self.game_state.world.data, c, gl);
        });

        self.window.swap_buffers();
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


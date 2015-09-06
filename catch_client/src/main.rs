extern crate renet as enet;
extern crate cereal;
#[macro_use] extern crate ecs;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate image;

extern crate catch_shared as shared;

mod client;
mod player_input;
mod draw_map;
mod components;
mod services;
mod systems;
mod state;

use piston::window::WindowSettings;
use piston::input::*;
use piston::event_loop::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL};
use graphics::Transformed;

use shared::player::PlayerInput;
use shared::net::ClientMessage;
use shared::map::Map;
use client::Client;
use player_input::InputMap;
use draw_map::DrawMap;
use state::GameState;

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let window = GlutinWindow::new(
        WindowSettings::new(
            "catching game",
            [1024, 768]
        )
        .opengl(opengl)
        .exit_on_esc(true)
    ).unwrap();

    // Connect
    enet::initialize().unwrap();

    let mut client = Client::connect(5000, "127.0.0.1".to_string(), 2338, "leo".to_string())
        .unwrap();
    client.finish_connecting(5000).unwrap();

    println!("Connected to server! My id: {}", client.get_my_id());

    let map = Map::load(&client.get_game_info().map_name).unwrap();
    let draw_map = DrawMap::load(&map).unwrap();

    // Create a new game and run it.
    let player_input_map = InputMap::new();
    let mut player_input = PlayerInput::new();
    let mut tick_number = None;
    let mut game_state = GameState::new(client.get_my_id(), client.get_game_info());

    let mut gl = GlGraphics::new(opengl);

    for e in window.events().ups(client.get_game_info().ticks_per_second as u64).max_fps(60) {
        match e {
            Event::Render(render_args) => {
                gl.draw(render_args.viewport(), |c, gl| {
                    // Clear the screen.
                    graphics::clear([0.0, 0.0, 0.0, 0.0], gl);

                    let trans = match game_state.world.systems.net_entity_system.inner.as_mut().unwrap().my_player_entity_id() {
                        Some(player_entity_id) => {
                            let player_entity = game_state.world.systems.net_entity_system.inner.as_mut().unwrap().get_entity(player_entity_id).unwrap();

                            game_state.world.with_entity_data(&player_entity, |e, c| {
                                c.position[e].p
                            }).unwrap()
                        }
                        None => [0.0, 0.0]
                    };

                    let c = c.trans(-trans[0], -trans[1]);
                    draw_map.draw(&map, c, gl);
                    game_state.world.systems.draw_player_system.draw(&mut game_state.world.data, c, gl);
                });
            }
            Event::Update(update_args) => {
                loop {
                    match client.service().unwrap() {
                        Some(message) => continue,
                        None => break
                    };
                }

                if client.num_ticks() > 0 {
                    let tick = client.pop_next_tick();
                    //println!("Starting tick {}", tick.tick_number);
                    game_state.run_tick(&tick);
                    tick_number = Some(tick.tick_number);
                }

                if let Some(tick) = tick_number {
                    //println!("Sending input {:?}", &player_input);
                    client.send(&ClientMessage::PlayerInput {
                        tick: tick,
                        input: player_input.clone()
                    });
                }

            }
            Event::Input(input) => {
                player_input_map.update_player_input(&input, &mut player_input);
            }
            _ => ()
        };
    }
}


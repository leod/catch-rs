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

use piston::window::WindowSettings;
use piston::input::*;
use piston::event_loop::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{ GlGraphics, OpenGL };

use shared::player::PlayerInput;
use shared::net::ClientMessage;
use shared::map::Map;
use client::Client;
use player_input::InputMap;
use draw_map::DrawMap;

pub struct App {
    client: Client,
    map: Map,
    draw_map: DrawMap,
}

impl App {
    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics) {
        gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            graphics::clear([0.0, 0.0, 0.0, 0.0], gl);

            self.draw_map.draw(&self.map, c, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.client.service().unwrap();
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let window = GlutinWindow::new(
        WindowSettings::new(
            "spinning-square",
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
    let mut app = App {
        client: client,
        map: map,
        draw_map: draw_map,
    };

    let player_input_map = InputMap::new();
    let mut player_input = PlayerInput::new();

    let mut gl = GlGraphics::new(opengl);

    for e in window.events().ups(100).max_fps(60) {
        match e {
            Event::Render(render_args) =>
                app.render(&render_args, &mut gl),
            Event::Update(update_args) => {
                app.client.send(&ClientMessage::PlayerInput(player_input.clone()));
                app.update(&update_args);
            }
            Event::Input(input) =>
                player_input_map.update_player_input(&input, &mut player_input),
            _ => ()
        };
    }
}


extern crate renet as enet;
extern crate cereal;
#[macro_use] extern crate ecs;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate image;
extern crate time;
extern crate gl;

extern crate catch_shared as shared;

mod client;
mod player_input;
mod draw_map;
mod components;
mod services;
mod systems;
mod state;
mod game;

use piston::window::WindowSettings;
use glutin_window::GlutinWindow;
use opengl_graphics::{OpenGL, GlGraphics};

use client::Client;
use player_input::InputMap;
use game::Game;

fn main() {
    let opengl = OpenGL::V3_2;

    let window = GlutinWindow::new(
        WindowSettings::new(
            "catching game",
            [1280, 1024]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .vsync(true)
        //.fullscreen(true)
    ).unwrap();

    // Connect
    enet::initialize().unwrap();

    let mut client = Client::connect(5000,
                                     "127.0.0.1".to_string(),
                                     2338,
                                     "leo".to_string()).unwrap();
    client.finish_connecting(5000).unwrap();

    println!("Connected to server! My id: {}", client.get_my_id());

    let mut gl = GlGraphics::new(opengl);

    let mut game = Game::new(client,
                             InputMap::new(),
                             window);
    game.run(&mut gl);
}


extern crate renet as enet;
extern crate cereal;
#[macro_use] extern crate ecs;
extern crate piston;
extern crate piston_window;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate image;
extern crate time;
extern crate gl;
extern crate getopts;

#[macro_use] extern crate catch_shared as shared;

mod client;
mod player_input;
mod draw_map;
mod components;
mod services;
mod systems;
mod state;
mod game;

use std::env;

use getopts::Options;
use piston::window::WindowSettings;
use glutin_window::GlutinWindow;
use opengl_graphics::{OpenGL, GlGraphics};

use client::Client;
use player_input::InputMap;
use game::Game;

fn main() {
    let args: Vec<String> = env::args().collect();
    //let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "connect", "set server address to connect to", "ADDRESS");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!(e.to_string())
    };
    let address = match matches.opt_str("c") {
        Some(s) => s,
        None => "127.0.0.1".to_string()
    };

    let opengl = OpenGL::V3_2;
    let window = GlutinWindow::new(
        WindowSettings::new("catching game", [1024, 768])
        .opengl(opengl)
        .exit_on_esc(true)
        .vsync(true)
        .fullscreen(false)
    ).unwrap();

    // Connect
    enet::initialize().unwrap();
    println!("Connecting to {}", address);
    let mut client = Client::connect(5000,
                                     address,
                                     2338,
                                     "leo".to_string()).unwrap();
    client.finish_connecting(5000).unwrap();

    println!("Connected to server! My id: {}", client.get_my_id());


    let mut game = Game::new(client,
                             InputMap::new(),
                             window);
    let mut gl = GlGraphics::new(opengl);
    game.run(&mut gl);
}


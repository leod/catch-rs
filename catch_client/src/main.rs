#[macro_use] extern crate log;
extern crate env_logger;
extern crate renet as enet;
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
extern crate color;
extern crate rand;
extern crate rodio;
extern crate cpal;
extern crate hprof;
extern crate rustc_serialize;
extern crate bincode;

#[macro_use] extern crate catch_shared as shared;

mod client;
mod player_input;
mod draw_map;
mod components;
mod entities;
mod services;
mod systems;
mod state;
mod game;
mod particles;
mod sounds;
mod dummy;

use std::env;

use getopts::Options;
use piston::window::WindowSettings;
use glutin_window::GlutinWindow;
use opengl_graphics::{OpenGL, GlGraphics};

use client::Client;
use player_input::InputMap;
use game::Game;
use dummy::DummyClient;

fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("c", "connect", "set server address to connect to", "ADDRESS");
    opts.optflag("", "dummy", "create a dummy client without graphical display");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!(e.to_string())
    };
    let address = match matches.opt_str("c") {
        Some(s) => s,
        None => "127.0.0.1".to_string()
    };
    let dummy = matches.opt_present("dummy");

    // Connect
    enet::initialize().unwrap();
    let port = 9988;
    info!("connecting to {}:{}", address, port);
    let mut client = Client::connect(5000,
                                     address,
                                     9988,
                                     "leo".to_string()).unwrap();
    client.finish_connecting(5000).unwrap();

    info!("connected to server! My id: {}", client.my_id());
    info!("game info: {:?}", client.game_info());

    if !dummy {
        let opengl = OpenGL::V3_2;
        let window = GlutinWindow::new(
            WindowSettings::new("catching game", [800, 600])
            .opengl(opengl)
            .exit_on_esc(true)
            .vsync(true)
            .fullscreen(false)
        ).unwrap();

        let mut game = Game::new(client,
                                 InputMap::new(),
                                 window);
        let mut gl = GlGraphics::new(opengl);
        game.run(&mut gl);
    } else {
        let mut dummy = DummyClient::new(client);
        dummy.run();
    }
}


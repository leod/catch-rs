#[macro_use] extern crate log;
extern crate env_logger;
extern crate renet as enet;
#[macro_use] extern crate ecs;
extern crate time;
#[macro_use] extern crate glium;
extern crate glium_text;
extern crate getopts;
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

use glium::DisplayBuild;

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
        let display = glium::glutin::WindowBuilder::new()
            .with_dimensions(1024, 768)
            .with_title(format!("Catching game"))
            .build_glium()
            .unwrap();

        let mut game = Game::new(client,
                                 InputMap::new(),
                                 display);
        game.run();
    } else {
        let mut dummy = DummyClient::new(client);
        dummy.run();
    }
}


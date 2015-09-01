#![feature(libc)]

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate renet as enet;
extern crate cereal;
#[macro_use] extern crate ecs;

extern crate catch_shared as shared;

mod client;

use std::cell::RefCell;
use std::rc::Rc;

use piston::window::WindowSettings;
use piston::input::*;
use piston::event_loop::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{ GlGraphics, OpenGL };

use shared::net::*;
use client::Client;

pub struct App {
    client: Client,

    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64   // Rotation for the square.
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        let (x, y) = ((args.draw_width / 2) as f64,
                      (args.draw_height / 2) as f64);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);

            let transform = c.transform.trans(x, y)
                                       .rot_rad(rotation)
                                       .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.client.service();

        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let window = GlutinWindow::new(
        WindowSettings::new(
            "spinning-square",
            [200, 200]
        )
        .opengl(opengl)
        .exit_on_esc(true)
    ).unwrap();

    // Connect
    enet::initialize();

    let mut client = Client::connect(5000, "127.0.0.1".to_string(), 2338, "leo".to_string())
        .unwrap();
    let my_player_id = client.finish_connecting(5000).unwrap();

    println!("Connected to server! My id: {}", my_player_id);

    // Create a new game and run it.
    let mut app = App {
        client: client,
        gl: GlGraphics::new(opengl),
        rotation: 0.0
    };

    for e in window.events() {
        match e {
            Event::Render(render_args) => app.render(&render_args),
            Event::Update(update_args) => app.update(&update_args),
            _ => ()
        };
    }
}


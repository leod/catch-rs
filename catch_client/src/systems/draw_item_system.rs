use std::f32;

use ecs::{Aspect, System, DataHelper, Process};

use rand;
use graphics;
use graphics::context::Context;
use graphics::Transformed;
use opengl_graphics::GlGraphics;

use shared::util::CachedAspect;

use components::{Components, Shape};
use services::Services;
use particles::Particles;

pub struct DrawItemSystem {
    aspect: CachedAspect<Components>,
}

impl DrawItemSystem {
    pub fn new(aspect: Aspect<Components>) -> DrawItemSystem {
        DrawItemSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn draw(&mut self, data: &mut DataHelper<Components, Services>, time_s: f32,
                particles: &mut Particles, c: graphics::Context, gl: &mut GlGraphics) {
        for entity in self.aspect.iter() {
            let p = data.position[entity].p;
            let size = match data.shape[entity] {
                Shape::Square { size } => size as f64,
                _ => panic!("item should be square"),
            };

            data.draw_item[entity].particle_timer.add(time_s);
            while data.draw_item[entity].particle_timer.next() {
                let color = [0.1, 0.9, 0.1];
                particles.spawn_cone(1.0, // duration in seconds
                                     color, color,
                                     1.5, // size
                                     p, // position
                                     data.orientation[entity].angle - f32::consts::PI,
                                     f32::consts::PI * 2.0,
                                     20.0 + rand::random::<f32>() * 10.0, // speed
                                     f32::consts::PI,
                                     0.0 // friction
                                     );
            }

            let transform = c.trans(p[0] as f64, p[1] as f64)
                             .rot_rad(data.orientation[entity].angle as f64)
                             .transform;

            graphics::rectangle([0.0, 1.0, 0.0, 1.0],
                                [-size/2.0, -size/2.0, size, size],
                                transform,
                                gl);
        }
    }
}

impl_cached_system!(Components, Services, DrawItemSystem, aspect);

impl Process for DrawItemSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

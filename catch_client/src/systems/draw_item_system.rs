use std::f32;

use ecs::{Aspect, System, DataHelper, Process};
use na::{Vec4, Mat2, Mat4};

use rand;

use shared::util::CachedAspect;

use components::{Components, Shape};
use services::Services;
use particles::Particles;
use draw::{DrawElement, DrawList, DrawAttributes};

pub struct DrawItemSystem {
    aspect: CachedAspect<Components>,
}

impl DrawItemSystem {
    pub fn new(aspect: Aspect<Components>) -> DrawItemSystem {
        DrawItemSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn spawn_particles(&mut self, data: &mut DataHelper<Components, Services>, time_s: f32,
                           particles: &mut Particles) {
        for entity in self.aspect.iter() {
            let p = data.position[entity].p;
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
        }
    }

    pub fn draw(&mut self, data: &mut DataHelper<Components, Services>, draw_list: &mut DrawList) {
        for entity in self.aspect.iter() {
            let p = data.position[entity].p;
            let size = match data.shape[entity] {
                Shape::Square { size } => size,
                _ => panic!("item should be square"),
            };

            let alpha = data.orientation[entity].angle;
            let rot_mat = Mat2::new(alpha.cos(), -alpha.sin(),
                                    alpha.sin(), alpha.cos());
            let scale_mat = Mat2::new(size, 0.0,
                                      0.0, size);
            let m = rot_mat * scale_mat;
            let model_mat = Mat4::new(m.m11, m.m12, 0.0, p.x,
                                      m.m21, m.m22, 0.0, p.y,
                                      0.0, 0.0, 1.0, 0.0,
                                      0.0, 0.0, 0.0, 1.0);
            draw_list.push(1, DrawElement::Square,
                           DrawAttributes::new(Vec4::new(0.0, 1.0, 0.0, 1.0), model_mat));
        }
    }
}

impl_cached_system!(Components, Services, DrawItemSystem, aspect);

impl Process for DrawItemSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

use std::f64;

use ecs::{Aspect, System, DataHelper, Process};

use glium::Surface;

use shared::{movement, math};
use shared::util::CachedAspect;

use components::Components;
use services::Services;
use particles::Particles;

pub struct DrawWallSystem {
    aspect: CachedAspect<Components>,
}

impl DrawWallSystem {
    pub fn new(aspect: Aspect<Components>) -> DrawWallSystem {
        DrawWallSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn draw<S: Surface>(&mut self, data: &mut DataHelper<Components, Services>, _: f32,
                            _: &mut Particles, target: &mut S) {
        for entity in self.aspect.iter() {
            let p = data.wall_position[entity].clone();
            //let w = p.pos_b[0] - p.pos_a[0];
            //let h = p.pos_b[1] - p.pos_a[1];
            let w = math::square_len([p.pos_b[0]-p.pos_a[0], p.pos_b[1]-p.pos_a[1]]).sqrt();

            let angle = movement::wall_orientation(&p);
            
            let size = 2.0; // TODO

            /*let transform = c.trans(p.pos_a[0] as f64, p.pos_a[1] as f64)
                             .rot_rad(angle as f64 + f64::consts::PI / 2.0).transform;

            graphics::rectangle([1.0, 1.0, 1.0, 1.0],
                                [0.0, -size/2.0 as f64, w as f64, size/2.0 as f64],
                                //[-5.0, -5.0, 10.0, 10.0],
                                transform,
                                gl);*/
        }
    }
}

impl_cached_system!(Components, Services, DrawWallSystem, aspect);

impl Process for DrawWallSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

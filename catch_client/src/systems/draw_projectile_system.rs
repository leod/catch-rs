use ecs::{Aspect, System, DataHelper, Process};

use shared::util::CachedAspect;

use components::{Components, Shape};
use services::Services;
use particles::Particles;
use draw::{DrawElement, DrawList, DrawAttributes};

pub struct DrawProjectileSystem {
    aspect: CachedAspect<Components>,
}

impl DrawProjectileSystem {
    pub fn new(aspect: Aspect<Components>) -> DrawProjectileSystem {
        DrawProjectileSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn draw(&mut self, data: &mut DataHelper<Components, Services>, draw_list: &mut DrawList) {
        for entity in self.aspect.iter() {
            let p = data.position[entity].p;
            let angle = data.orientation[entity].angle;

            let (w, h) = match data.shape[entity] {
                Shape::Rect { width, height } => (width as f64, height as f64),
                _ => panic!("projectile should be rect"),
            };

            /*let transform = c.trans(p[0] as f64, p[1] as f64)
                             .rot_rad(angle as f64).transform;

            graphics::rectangle([0.4, 0.4, 0.4, 1.0],
                                [-w/2.0, -h/2.0, w, h],
                                transform,
                                gl);*/
        }
    }
}

impl_cached_system!(Components, Services, DrawProjectileSystem, aspect);

impl Process for DrawProjectileSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

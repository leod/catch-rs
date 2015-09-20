use ecs::{Aspect, System, DataHelper, Process};

use graphics;
use graphics::context::Context;
use graphics::Transformed;
use opengl_graphics::GlGraphics;

use shared::util::CachedAspect;

use components::{Components, Shape};
use services::Services;

pub struct DrawBouncyEnemySystem {
    aspect: CachedAspect<Components>,
}

impl DrawBouncyEnemySystem {
    pub fn new(aspect: Aspect<Components>) -> DrawBouncyEnemySystem {
        DrawBouncyEnemySystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn draw(&mut self, data: &mut DataHelper<Components, Services>, c: graphics::Context,
                gl: &mut GlGraphics) {
        for entity in self.aspect.iter() {
            let p = data.position[entity].p;
            /*let w = 12.0;
            let h = 12.0;*/

            let r = match data.shape[entity] {
                Shape::Circle { radius } => radius,
                //_ => panic!("enemy should be circle"),
            };

            let transform = c.trans(p[0], p[1]).transform;

            graphics::ellipse([1.0, 0.0, 0.0, 1.0],
                              [-r, -r, r*2.0, r*2.0],
                              transform,
                              gl);
        }
    }
}

impl_cached_system!(Components, Services, DrawBouncyEnemySystem, aspect);

impl Process for DrawBouncyEnemySystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

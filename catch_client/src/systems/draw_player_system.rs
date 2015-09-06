use ecs;
use ecs::{Aspect, System, EntityData, EntityIter, DataHelper, BuildData, Process};
use ecs::system::EntityProcess;

use graphics;
use graphics::context::Context;
use graphics::Transformed;
use opengl_graphics::{GlGraphics, Texture};

use shared::util::CachedAspect;
use components::*;
use services::Services;

pub struct DrawPlayerSystem {
    aspect: CachedAspect<Components>,
}

impl DrawPlayerSystem {
    pub fn new(aspect: Aspect<Components>) -> DrawPlayerSystem {
        DrawPlayerSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn draw(&mut self, data: &mut DataHelper<Components, Services>, c: graphics::Context, gl: &mut GlGraphics) {
        for entity in self.aspect.iter() {
            //let transform = c.transform.translate(
            let p = data.position[entity].p;
            //println!("{:?}", data.position[entity].p);
            graphics::ellipse([1.0, 0.0, 1.0, 1.0], [0.0, 0.0, 10.0, 10.0], c.trans(p[0], p[1]).transform, gl);
        }
    }
}

impl System for DrawPlayerSystem {
    type Components = Components;
    type Services = Services;

    fn activated(&mut self, entity: &EntityData<Components>, components: &Components, services: &mut Services) {
        self.aspect.activated(entity, components);
    }

    fn reactivated(&mut self, entity: &EntityData<Components>, components: &Components, services: &mut Services) {
        self.aspect.reactivated(entity, components);
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, components: &Components, services: &mut Services) {
        self.aspect.deactivated(entity, components);
    }
}

impl Process for DrawPlayerSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

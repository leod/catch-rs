use ecs::{Aspect, System, EntityData, DataHelper, Process};

use graphics;
use graphics::context::Context;
use graphics::Transformed;
use opengl_graphics::GlGraphics;

use shared::math;
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
            // TODO: Store this somwehere
            let w = 16.0;
            let h = 16.0;

            let scale_x_target = if data.player_state[entity].dashing.is_some() {
                math::square_len(data.linear_velocity[entity].v).sqrt() / 400.0 + 1.0
            } else {
                1.0
            };

            data.draw_player[entity].scale_x += (scale_x_target - data.draw_player[entity].scale_x) * 0.1;
            let scale_x = data.draw_player[entity].scale_x;

            let transform = c.trans(p[0], p[1])
                             .rot_rad(data.orientation[entity].angle)
                             .scale(scale_x, 1.0/scale_x)
                             //.trans(-w/2.0, -h/2.0)
                             .transform;

            graphics::ellipse([0.0, 0.0, 1.0, 1.0],
                              [-w/2.0, -h/2.0, w, h],
                              transform,
                              gl);
            graphics::rectangle([0.0, 0.0, 0.0, 1.0],
                                [0.0, -1.5, 12.0, 3.0],
                                transform,
                                gl);
        }
    }
}

impl System for DrawPlayerSystem {
    type Components = Components;
    type Services = Services;

    fn activated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.aspect.activated(entity, components);
    }

    fn reactivated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.aspect.reactivated(entity, components);
    }

    fn deactivated(&mut self, entity: &EntityData<Components>, components: &Components, _: &mut Services) {
        self.aspect.deactivated(entity, components);
    }
}

impl Process for DrawPlayerSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

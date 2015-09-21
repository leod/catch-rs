use ecs::{Aspect, System, DataHelper, Process};

use graphics;
use graphics::context::Context;
use graphics::Transformed;
use opengl_graphics::GlGraphics;

use shared::math;
use shared::util::CachedAspect;
use components::{Components, Shape};
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

    pub fn draw(&mut self, data: &mut DataHelper<Components, Services>, c: graphics::Context,
                gl: &mut GlGraphics) {
        for entity in self.aspect.iter() {
            let p = data.position[entity].p;
            let r = match data.shape[entity] {
                Shape::Circle { radius } => radius,
                _ => panic!("player should be circle"),
            };

            let scale_x_target = if data.player_state[entity].dashing.is_some() {
                math::square_len(data.linear_velocity[entity].v).sqrt() / 400.0 + 1.0
            } else {
                1.0
            };

            let delta_scale = (scale_x_target - data.draw_player[entity].scale_x) * 0.15;
            data.draw_player[entity].scale_x += delta_scale;

            let scale_x = data.draw_player[entity].scale_x;
            let transform = c.trans(p[0], p[1])
                             .rot_rad(data.orientation[entity].angle)
                             .scale(scale_x, 1.0/scale_x)
                             .transform;

            let color =
                if data.player_state[entity].invulnerable_s.is_some() { [0.25, 0.25, 0.25, 1.0] }
                else { [0.0, 0.0, 1.0, 1.0] };

            graphics::ellipse(color,
                              [-r, -r, r*2.0, r*2.0],
                              transform,
                              gl);
            graphics::rectangle([0.0, 0.0, 0.0, 1.0],
                                [0.0, -1.5, r, 3.0],
                                transform,
                                gl);
        }
    }
}

impl_cached_system!(Components, Services, DrawPlayerSystem, aspect);

impl Process for DrawPlayerSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

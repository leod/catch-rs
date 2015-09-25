use std::f64;

use ecs::{Aspect, System, DataHelper, Process};

use rand;
use graphics;
use graphics::context::Context;
use graphics::Transformed;
use opengl_graphics::GlGraphics;
use color::{Rgb, Hsv, ToHsv, ToRgb};

use shared::math;
use shared::util::CachedAspect;

use components::{Components, Shape};
use services::Services;
use particles::Particles;

pub struct DrawPlayerSystem {
    aspect: CachedAspect<Components>,
}

impl DrawPlayerSystem {
    pub fn new(aspect: Aspect<Components>) -> DrawPlayerSystem {
        DrawPlayerSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn draw(&mut self, data: &mut DataHelper<Components, Services>, time_s: f64,
                particles: &mut Particles, c: graphics::Context, gl: &mut GlGraphics) {
        for entity in self.aspect.iter() {
            let p = data.position[entity].p;
            let r = match data.shape[entity] {
                Shape::Circle { radius } => radius,
                _ => panic!("player should be circle"),
            };

            if data.player_state[entity].dashing.is_some() {
                let color = if rand::random::<bool>() {
                    //[0.9, 0.5, 0.0]
                    [0.2, 0.77, 0.95]
                } else {
                    [0.2, 0.77, 0.95]
                };

                data.draw_player[entity].dash_particle_timer.add(time_s);
                while data.draw_player[entity].dash_particle_timer.next() {
                    for i in 0..5 {
                        let color = [color[0] + (-0.5 + rand::random::<f32>()) * 0.2,
                                     color[1] + (-0.5 + rand::random::<f32>()) * 0.2,
                                     color[2] + (-0.5 + rand::random::<f32>()) * 0.2];
                        particles.spawn_cone(1.0, // duration in seconds
                                             color, color,
                                             2.5, // size
                                             p, // position
                                             data.orientation[entity].angle - f64::consts::PI,
                                             f64::consts::PI / 8.0,
                                             100.0 + rand::random::<f64>() * 50.0, // speed
                                             //f64::consts::PI * 2.0,
                                             0.0,
                                             1.0,
                                             );
                    }
                }
            }

            let scale_x_target = if data.player_state[entity].dashing.is_some() {
                math::square_len(data.linear_velocity[entity].v).sqrt() / 400.0 + 1.0
            } else {
                1.0
            };
            let delta_scale = (scale_x_target - data.draw_player[entity].scale_x) * 0.15;
            data.draw_player[entity].scale_x += delta_scale;

            /*let color_target =
                if data.player_state[entity].invulnerable_s.is_some() { [0.25, 0.25, 0.25, 1.0] }
                else if data.player_state[entity].dashing.is_some() { [1.0, 0.65, 0.0, 1.0] }
                else { [0.0, 0.0, 1.0, 1.0] };*/
            let color_target_rgb =
                if data.player_state[entity].invulnerable_s.is_some() {
                    Rgb::new(0.25f32, 0.25, 0.25) 
                } else if data.player_state[entity].dashing.is_some() {
                    let t = data.player_state[entity].dashing.unwrap() / 
                            0.3;
                    Rgb::new(1.0f32, 0.65f32 - 0.5 * t as f32, 0.0)
                } else {
                    Rgb::new(0.0f32, 0.0, 1.0)
                };
            /*let color_target_hsv = color_target_rgb.to_hsv();
            let color = data.draw_player[entity].color;
            let mut color_hsv = Rgb::new(color[0], color[1], color[2]).to_hsv();
            color_hsv.h = color_hsv.h + (color_target_hsv.h - color_hsv.h) * 0.4;
            color_hsv.s = color_hsv.s + (color_target_hsv.s - color_hsv.s) * 0.4;
            color_hsv.v = color_hsv.v + (color_target_hsv.v - color_hsv.v) * 0.4;

            let color_rgb = color_hsv.to_rgb();*/
            let color_rgb = color_target_rgb;
            let color = [color_rgb.r, color_rgb.g, color_rgb.b, 1.0];
            data.draw_player[entity].color = color;

            let scale_x = data.draw_player[entity].scale_x;
            let transform = c.trans(p[0], p[1])
                             .rot_rad(data.orientation[entity].angle)
                             .scale(scale_x, 1.0/scale_x)
                             .transform;
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

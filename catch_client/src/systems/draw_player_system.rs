use std::f32;

use ecs::{Aspect, System, DataHelper, Process};
use rand;
use na::Norm;

use glium::Surface;

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

    pub fn draw<S: Surface>(&mut self, data: &mut DataHelper<Components, Services>, time_s: f32,
                            particles: &mut Particles, surface: &mut S) {
        for entity in self.aspect.iter() {
            let p = data.position[entity].p;
            let r = match data.shape[entity] {
                Shape::Circle { radius } => radius as f64,
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
                    for _ in 0..5 {
                        let color = [color[0] + (-0.5 + rand::random::<f32>()) * 0.2,
                                     color[1] + (-0.5 + rand::random::<f32>()) * 0.2,
                                     color[2] + (-0.5 + rand::random::<f32>()) * 0.2];
                        particles.spawn_cone(1.0, // duration in seconds
                                             color, color,
                                             2.5, // size
                                             p, // position
                                             data.orientation[entity].angle - f32::consts::PI,
                                             f32::consts::PI / 8.0,
                                             100.0 + rand::random::<f32>() * 50.0, // speed
                                             //f64::consts::PI * 2.0,
                                             0.0,
                                             1.0,
                                             );
                    }
                }
            }

            let scale_x_target = if data.player_state[entity].dashing.is_some() {
                data.linear_velocity[entity].v.norm() / 400.0 + 1.0
            } else {
                1.0
            };
            let delta_scale = (scale_x_target - data.draw_player[entity].scale_x) * 0.15;
            data.draw_player[entity].scale_x += delta_scale;

            let color =
                if data.player_state[entity].invulnerable_s.is_some() {
                    [0.25f32, 0.25, 0.25, 1.0] 
                } else if data.player_state[entity].dashing.is_some() {
                    let t = data.player_state[entity].dashing.unwrap() / 
                            0.3;
                    [1.0f32, 0.65f32 - 0.5 * t, 0.0, 1.0]
                } else if data.player_state[entity].is_catcher {
                    [0.0, 1.0, 0.0, 1.0]
                } else {
                    [0.0f32, 0.0, 1.0, 1.0]
                };
            data.draw_player[entity].color = color;

            let scale_x = data.draw_player[entity].scale_x;
            /*let transform = c.trans(p[0] as f64, p[1] as f64)
                             .rot_rad(data.orientation[entity].angle as f64)
                             .scale(scale_x as f64, 1.0/scale_x as f64)
                             .transform;
            graphics::ellipse(color,
                              [-r, -r, r*2.0, r*2.0],
                              transform,
                              gl);
            graphics::rectangle([0.0, 0.0, 0.0, 1.0],
                                [0.0, -1.5, r, 3.0],
                                transform,
                                gl);*/
        }
    }
}

impl_cached_system!(Components, Services, DrawPlayerSystem, aspect);

impl Process for DrawPlayerSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

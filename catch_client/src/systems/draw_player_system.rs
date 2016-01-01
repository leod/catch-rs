use std::f32;

use ecs::{Aspect, System, DataHelper, Process};
use rand;
use na::{Vec2, Vec4, Mat2, Mat4, Norm, Inv};

use shared::util::CachedAspect;

use components::{Components, Shape};
use services::Services;
use particles::Particles;
use draw::{FLAG_NONE, FLAG_BLUR, DrawElement, DrawList, DrawAttributes};

pub struct DrawPlayerSystem {
    aspect: CachedAspect<Components>,
}

impl DrawPlayerSystem {
    pub fn new(aspect: Aspect<Components>) -> DrawPlayerSystem {
        DrawPlayerSystem {
            aspect: CachedAspect::new(aspect),
        }
    }

    pub fn spawn_particles(&mut self, data: &mut DataHelper<Components, Services>, time_s: f32,
                           particles: &mut Particles) {
        for entity in self.aspect.iter() {
            let p = data.position[entity].p;

            if data.player_state[entity].dashing.is_some() {
                let color = if rand::random::<bool>() {
                    [0.2, 0.2, 1.0]
                } else {
                    [0.2, 0.2, 1.0]
                };

                data.draw_player[entity].dash_particle_timer.add(time_s);
                while data.draw_player[entity].dash_particle_timer.next() {
                    for _ in 0..20 {
                        let color = [color[0] + (-0.5 + rand::random::<f32>()) * 0.1,
                                     color[1] + (-0.5 + rand::random::<f32>()) * 0.1,
                                     color[2] + (-0.5 + rand::random::<f32>()) * 0.1];
                        particles.spawn_cone(0.5, // duration in seconds
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
        }
    }

    pub fn draw(&mut self, data: &mut DataHelper<Components, Services>, draw_list: &mut DrawList) {
        for entity in self.aspect.iter() {
            let p = data.position[entity].p;
            let r = match data.shape[entity] {
                Shape::Circle { radius } => radius,
                _ => panic!("player should be circle"),
            };
            let angle = data.orientation[entity].angle;

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
            let rot_mat = Mat2::new(angle.cos(), -angle.sin(),
                                    angle.sin(), angle.cos());

            draw_list.push_ellipse(FLAG_BLUR, Vec4::new(color[0], color[1], color[2], 1.0),
                                   scale_x * r, 1.0 / scale_x * r,
                                   p, 1.0, angle);

            if data.player_state[entity].has_shield {
                let s = 2.0*r + 8.0;
                let scale_mat = Mat2::new(scale_x * s, 0.0,
                                          0.0, 1.0 / scale_x * s);
                let m = rot_mat * scale_mat * rot_mat.inv().unwrap();
                let model_mat = Mat4::new(m.m11, m.m12, 0.0, p.x,
                                          m.m21, m.m22, 0.0, p.y,
                                          0.0, 0.0, 1.0, 0.0,
                                          0.0, 0.0, 0.0, 1.0);
                draw_list.push(DrawElement::TexturedSquare { texture: "shield".to_string() },
                               DrawAttributes::new(FLAG_NONE, 1.0, Vec4::new(0.0, 0.0, 0.0, 1.0), model_mat));
            }

            let scale_mat = Mat2::new(scale_x * r, 0.0,
                                      0.0, 1.0 / scale_x * 2.0);
            let m = rot_mat * scale_mat;
            let o = m * Vec2::new(0.5, 0.0);
            let model_mat = Mat4::new(m.m11, m.m12, 0.0, p.x + o.x,
                                      m.m21, m.m22, 0.0, p.y + o.y,
                                      0.0, 0.0, 1.0, 0.0,
                                      0.0, 0.0, 0.0, 1.0);
            draw_list.push(DrawElement::Square,
                           DrawAttributes::new(FLAG_NONE, 2.0, Vec4::new(0.0, 0.0, 0.0, 1.0), model_mat));
        }
    }
}

impl_cached_system!(Components, Services, DrawPlayerSystem, aspect);

impl Process for DrawPlayerSystem {
    fn process(&mut self, _: &mut DataHelper<Components, Services>) {
    }
}

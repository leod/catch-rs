use rand;
use graphics;
use graphics::context::Context;
use graphics::Transformed;
use opengl_graphics::GlGraphics;

use shared::Map;
use shared::math;

#[derive(Clone, Debug)]
struct Particle {
    progress_per_s: f64,
    progress: f64,

    color_a: [f32; 3],
    color_b: [f32; 3],
    size: f64,

    position: math::Vec2,
    orientation: f64,
    velocity: math::Vec2,
    ang_velocity: f64,

    friction: f64,
}

pub struct Particles {
    particles: Vec<Option<Particle>>,
    free_indices: Vec<usize>,
}

impl Particles {
    pub fn new() -> Particles {
        Particles {
            particles: Vec::new(),
            free_indices: Vec::new(),
        }
    }

    pub fn update(&mut self, time_s: f64, map: &Map) {
        for i in 0..self.particles.len() {
            let remove = if let Some(p) = self.particles[i].as_mut() {
                p.velocity = math::sub(p.velocity, math::scale(p.velocity,
                                                               p.friction * time_s));
                p.position = math::add(p.position, math::scale(p.velocity, time_s));
                p.orientation += p.ang_velocity * time_s;

                let progress = p.progress + p.progress_per_s * time_s;
                if progress >= 1.0 {
                    true
                } else {
                    p.progress = progress;
                    false
                }
            } else {
                false
            };
            if remove {
                self.particles[i] = None;
                self.free_indices.push(i);
            }
        }
    }

    pub fn draw(&self, c: graphics::Context, gl: &mut GlGraphics) {
        for p in self.particles.iter() {
            if let &Some(ref p) = p {
                let alpha = 1.0 - p.progress;
                let transform = c.trans(p.position[0], p.position[1])
                                 .rot_rad(p.orientation)
                                 .transform;
                let t = p.progress as f32;
                let color = [p.color_a[0] * (1.0-t) + p.color_b[0] * t,
                             p.color_a[1] * (1.0-t) + p.color_b[1] * t,
                             p.color_a[2] * (1.0-t) + p.color_b[2] * t];
                graphics::rectangle([color[0], color[1], color[2], alpha as f32],
                                    [-p.size/2.0, -p.size/2.0, p.size, p.size],
                                    transform,
                                    gl);
            }
        }
    }

    pub fn spawn_cone(&mut self,
                      lifetime_s: f64,
                      color_a: [f32; 3],
                      color_b: [f32; 3],
                      size: f64,
                      position: math::Vec2,
                      orientation: f64,
                      spread: f64,
                      speed: f64,
                      ang_velocity: f64,
                      friction: f64) {
        let orientation = orientation + spread * (rand::random::<f64>() - 0.5);
        let velocity = [speed * orientation.cos(), speed * orientation.sin()];

        self.add(
            &Particle {
                progress_per_s: 1.0 / lifetime_s,
                progress: 0.0,

                color_a: color_a,
                color_b: color_b,
                size: size,

                position: position,
                orientation: orientation,
                velocity: velocity,
                ang_velocity: ang_velocity,

                friction: friction,
            });
    }


    pub fn add(&mut self, p: &Particle) {
        match self.free_indices.pop() {
            Some(i) => {
                assert!(self.particles[i].is_none());
                self.particles[i] = Some(p.clone())
            }
            None => {
                self.particles.push(Some(p.clone()));
            }
        };
    }
}

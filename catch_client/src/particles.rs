use std::collections::BinaryHeap;

use rand;
use graphics;
use graphics::context::Context;
use graphics::Transformed;
use opengl_graphics::GlGraphics;

use shared::Map;
use shared::math;

#[derive(Clone, Debug)]
struct Particle {
    progress_per_s: f32,
    progress: f32,

    color_a: [f32; 3],
    color_b: [f32; 3],
    size: f32,

    position: math::Vec2,
    orientation: f32,
    velocity: math::Vec2,
    ang_velocity: f32,

    friction: f32,
}

pub struct Particles {
    particles: Vec<Option<Particle>>,
    free_indices: Vec<usize>,

    num: usize,
}

impl Particles {
    pub fn new() -> Particles {
        Particles {
            particles: Vec::new(),
            free_indices: Vec::new(),
            num: 0,
        }
    }

    pub fn update(&mut self, time_s: f32, map: &Map) {
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
                assert!(self.num > 0);
                self.num -= 1;

                self.particles[i] = None;
                self.free_indices.push(i);
            }
        }
    }

    pub fn draw(&self, c: graphics::Context, gl: &mut GlGraphics) {
        for p in self.particles.iter() {
            if let &Some(ref p) = p {
                let alpha = 1.0 - p.progress;
                let transform = c.trans(p.position[0] as f64, p.position[1] as f64)
                                 .rot_rad(p.orientation as f64)
                                 .transform;
                let t = p.progress as f32;
                let color = [p.color_a[0] * (1.0-t) + p.color_b[0] * t,
                             p.color_a[1] * (1.0-t) + p.color_b[1] * t,
                             p.color_a[2] * (1.0-t) + p.color_b[2] * t];
                graphics::rectangle([color[0], color[1], color[2], alpha as f32],
                                    [(-p.size/2.0) as f64, (-p.size/2.0) as f64,
                                     p.size as f64, p.size as f64],
                                    transform,
                                    gl);
            }
        }
    }

    pub fn spawn_cone(&mut self,
                      lifetime_s: f32,
                      color_a: [f32; 3],
                      color_b: [f32; 3],
                      size: f32,
                      position: math::Vec2,
                      orientation: f32,
                      spread: f32,
                      speed: f32,
                      ang_velocity: f32,
                      friction: f32) {
        let orientation = orientation + spread * (rand::random::<f32>() - 0.5);
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
        self.num += 1;

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

    pub fn num(&self) -> usize {
        self.num        
    }
}

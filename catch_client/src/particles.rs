use std::collections::BinaryHeap;
use std::cmp::Ordering;
use rand;

use na::{Vec2, Vec3};

use glium::{self, Surface};

use draw::{self, DrawContext, Vertex};

pub const MAX_PARTICLES: usize = 100000;

#[derive(Copy, Clone, Debug)]
struct Particle {
    start_time_s: f32,
    progress_per_s: f32,

    color_a: Vec3<f32>,
    color_b: Vec3<f32>,
    size: f32,

    world_position: Vec2<f32>,
    orientation: f32,
    velocity: Vec2<f32>,
    ang_velocity: f32,

    friction: f32,
}

#[derive(Clone)]
struct ParticleInfo {
    particle: Particle,
    is_new: bool,
}

implement_vertex!(Particle, start_time_s, progress_per_s, color_a, color_b, size, world_position,
                  orientation, velocity, ang_velocity, friction);

#[derive(PartialEq, Eq, Copy, Clone)]
struct Index(usize);
impl PartialOrd for Index {
    fn partial_cmp(&self, other: &Index) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}
impl Ord for Index {
    fn cmp(&self, other: &Index) -> Ordering {
        other.0.cmp(&self.0)
    }
}

pub struct Particles {
    particles: Vec<Option<ParticleInfo>>,
    free_indices: BinaryHeap<Index>,
    num: usize,
    num_used_indices: usize,

    square_vertex_buffer: glium::VertexBuffer<Vertex>,
    square_index_buffer: glium::IndexBuffer<u16>,
    particles_vertex_buffer: glium::VertexBuffer<Particle>,
    program: glium::Program,

    time_s: f32,
}

impl Particles {
    pub fn new(display: &glium::Display) -> Particles {
        /*let geometry_shader_src = r#"
            #version 330

            layout(triangles) in;
            layout(triangles, max_vertices = 4) out;

            uniform float time_s;

            in float start_time_s;
            in float progress_per_s;

            void main() {
                float progress = (time_s - start_time_s) * progress_per_s;
                if (progress < 1.0) {
                    EmitVertex();
                    EndPrimitive();
                }
            }
        "#;*/

        let vertex_shader_src = r#"
            #version 140

            uniform mat4 proj_mat;
            uniform mat4 camera_mat;
            uniform float time_s;

            in vec2 position;

            in float start_time_s;
            in float progress_per_s;
            in vec3 color_a;
            in vec3 color_b;
            in float size;
            in vec2 world_position;
            in float orientation;
            in vec2 velocity;
            in float ang_velocity;
            in float friction;

            out vec4 color;

            void main() {
                float t = time_s - start_time_s;
                float progress = t * progress_per_s;

                color = vec4((1 - progress) * color_a + color_b * progress, 1.0 - progress);

                float phi = orientation + t * ang_velocity;
                mat2 rot = mat2(cos(phi), sin(phi),
                                -sin(phi), cos(phi));

                vec2 p = rot * position * size + world_position + velocity * t;
                gl_Position = proj_mat * camera_mat * vec4(p, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            uniform float time_s;

            in float start_time_s;
            in float progress_per_s;

            in vec4 color;

            out vec4 out_color;

            void main() {
                float t = time_s - start_time_s;
                float progress = t * progress_per_s;
                if (progress >= 1.0) discard;

                out_color = color;
            }
        "#;

        let (square_vertex_buffer, square_index_buffer) = draw::new_square(display);
        let particles_vertex_buffer = glium::VertexBuffer::empty_dynamic(display, MAX_PARTICLES).unwrap();

        Particles {
            particles: Vec::new(),
            free_indices: BinaryHeap::new(),
            num: 0,
            num_used_indices: 0,
            square_vertex_buffer: square_vertex_buffer,
            square_index_buffer: square_index_buffer,
            particles_vertex_buffer: particles_vertex_buffer,
            program: glium::Program::from_source(display, vertex_shader_src, fragment_shader_src,
                                                 None).unwrap(),
            time_s: 0.0,
        }
    }

    pub fn update(&mut self, time_s: f32) {
        let mut mapping = self.particles_vertex_buffer.map();
        self.num_used_indices = 0;

        for i in 0..self.particles.len() {
            let remove = if let Some(p) = self.particles[i].as_mut() {
                if p.is_new {
                    mapping[i] = p.particle.clone();
                    p.is_new = false;
                }

                let progress = p.particle.progress_per_s * (self.time_s - p.particle.start_time_s);

                if progress < 1.0 {
                    self.num_used_indices = i + 1; 
                    false
                } else {
                    true
                }
            } else {
                false
            };
            if remove {
                assert!(self.num > 0);
                self.num -= 1;

                self.particles[i] = None;
                self.free_indices.push(Index(i));
            }
        }

        self.time_s += time_s;

        //debug!("used: {}, free: {}, ps: {}", self.num_used_indices, self.free_indices.len(), self.particles.len());
    }

    pub fn draw<'a, S: Surface>(&self, draw_context: &DrawContext<'a>, target: &mut S) {
        let uniforms = uniform! {
            proj_mat: draw_context.proj_mat,
            camera_mat: draw_context.camera_mat, 
            time_s: self.time_s,
        };

        let parameters = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            .. draw_context.parameters.clone()
        };

        target.draw((&self.square_vertex_buffer,
                     self.particles_vertex_buffer.slice(0..self.num_used_indices)
                         .unwrap().per_instance().unwrap()),
                    &self.square_index_buffer, &self.program, &uniforms, &parameters)
              .unwrap();
    }

    pub fn spawn_cone(&mut self,
                      lifetime_s: f32,
                      color_a: [f32; 3],
                      color_b: [f32; 3],
                      size: f32,
                      position: Vec2<f32>,
                      orientation: f32,
                      spread: f32,
                      speed: f32,
                      ang_velocity: f32,
                      friction: f32) {
        let start_time_s = self.time_s;
        let orientation = orientation + spread * (rand::random::<f32>() - 0.5);
        let velocity = Vec2::new(speed * orientation.cos(), speed * orientation.sin());

        self.add(
            &Particle {
                start_time_s: start_time_s,
                progress_per_s: 1.0 / lifetime_s,

                color_a: Vec3::new(color_a[0], color_a[1], color_a[2]),
                color_b: Vec3::new(color_b[0], color_b[1], color_b[2]),
                size: size,

                world_position: position,
                orientation: orientation,
                velocity: velocity,
                ang_velocity: ang_velocity,

                friction: friction,
            });
    }


    pub fn add(&mut self, particle: &Particle) {
        self.num += 1;

        let info = ParticleInfo {
            particle: particle.clone(),
            is_new: true,
        };

        match self.free_indices.pop() {
            Some(Index(i)) => {
                assert!(self.particles[i].is_none());
                self.particles[i] = Some(info)
            }
            None => {
                if self.particles.len() < MAX_PARTICLES { 
                    self.particles.push(Some(info));
                } else {
                    // TODO
                }
            }
        };
    }

    pub fn num(&self) -> usize {
        self.num        
    }
}

use std::f32;

use na::{Vec4, Mat4};
use glium;

pub struct DrawContext<'a> {
    pub proj_mat: Mat4<f32>,
    pub camera_mat: Mat4<f32>,
    pub parameters: glium::DrawParameters<'a>,
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}

implement_vertex!(Vertex, position);

#[derive(Copy, Clone)]
pub enum DrawElement {
    Circle,
    Square,
}

#[derive(Copy, Clone)]
pub struct DrawAttributes {
    pub color: Vec4<f32>,
    pub model_mat: Mat4<f32>,
}
implement_vertex!(DrawAttributes, color, model_mat);

pub type DrawList = Vec<(DrawElement, DrawAttributes)>;

pub const SPRITE_VERTEX_BUFFER_SIZE: usize = 1024;

pub struct DrawDrawList {
    circle_vertex_buffer: glium::VertexBuffer<Vertex>,
    square_vertex_buffer: glium::VertexBuffer<Vertex>,
    square_index_buffer: glium::IndexBuffer<u16>,
    sprite_vertex_buffers: Vec<glium::VertexBuffer<DrawAttributes>>,

    program: glium::Program,
}

impl DrawDrawList {
    pub fn new(display: &glium::Display) -> DrawDrawList {
        let vertex_shader_src = r#"
            #version 140

            in vec2 position;

            uniform mat4 proj_mat;
            uniform mat4 camera_mat;

            in vec4 color;
            in mat4 model_mat;

            out vec4 color_v;
            
            void main() {
                color_v = color;
                gl_Position = proj_mat * camera_mat * model_mat * vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            out vec4 color_f;

            in vec4 color_v;

            void main() {
                color_f = color_v;
            }
        "#;

        let (square_vertex_buffer, square_index_buffer) = new_square(display);
        let sprite_vertex_buffer =
            glium::VertexBuffer::empty_dynamic(display, SPRITE_VERTEX_BUFFER_SIZE).unwrap();

        DrawDrawList {
            circle_vertex_buffer: new_circle(display, 32),
            square_vertex_buffer: square_vertex_buffer,
            square_index_buffer: square_index_buffer,
            sprite_vertex_buffers: vec![sprite_vertex_buffer],
            program: glium::Program::from_source(display, vertex_shader_src, fragment_shader_src,
                                                 None).unwrap(),
        }
    }

    fn get_sprite_vertex_buffer(&mut self, display: &glium::Display)
                                -> glium::VertexBuffer<DrawAttributes> {
        if let Some(vertex_buffer) = self.sprite_vertex_buffers.pop() {
            vertex_buffer
        } else {
            glium::VertexBuffer::empty_dynamic(display, SPRITE_VERTEX_BUFFER_SIZE).unwrap()
        }
    }

    pub fn draw<'a, S: glium::Surface>(&mut self, list: &DrawList, context: &DrawContext<'a>,
                                       display: &glium::Display, surface: &mut S) {
        // TODO

        let mut circle_sprite_buffers: Vec<glium::VertexBuffer<DrawAttributes>> = Vec::new();
        let mut square_sprite_buffers: Vec<glium::VertexBuffer<DrawAttributes>> = Vec::new();

        let mut circle_sprite_buffer = self.get_sprite_vertex_buffer(display);
        let mut square_sprite_buffer = self.get_sprite_vertex_buffer(display);

        let mut circle_i = 0;
        let mut square_i = 0;

        {
            let mut circle_mapping = circle_sprite_buffer.map();
            let mut square_mapping = square_sprite_buffer.map();

            // Create batches
            for &(element, ref attributes) in list.iter() {
                match element {
                    DrawElement::Circle => {
                        if circle_i == SPRITE_VERTEX_BUFFER_SIZE {
                            assert!(false);
                            /*circle_sprite_buffers.push(circle_sprite_buffer); 
                            circle_sprite_buffer = self.get_sprite_vertex_buffer(display);
                            circle_mapping = Some(circle_sprite_buffer.map());
                            circle_i = 0;*/
                        }
                        circle_mapping[circle_i] = attributes.clone();
                        circle_i += 1;

                        /*
                        surface.draw(&self.circle_vertex_buffer, &indices, &self.program, &uniforms,
                                     &Default::default()).unwrap();*/
                    },
                    DrawElement::Square => {
                        square_mapping[square_i] = attributes.clone();
                        square_i += 1;

                        /*surface.draw(&self.square_vertex_buffer, &self.square_index_buffer,
                                     &self.program, &uniforms, &Default::default()).unwrap();*/
                    },
                }
            }
        }

        let uniforms = uniform! {
            proj_mat: context.proj_mat,
            camera_mat: context.camera_mat,
        };

        surface.draw((&self.square_vertex_buffer,
                      square_sprite_buffer.slice(0..square_i).unwrap().per_instance().unwrap()),
                     &self.square_index_buffer, &self.program, &uniforms,
                     &context.parameters).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);
        surface.draw((&self.circle_vertex_buffer,
                      circle_sprite_buffer.slice(0..circle_i).unwrap().per_instance().unwrap()),
                     &indices, &self.program, &uniforms, &context.parameters).unwrap();

        self.sprite_vertex_buffers.push(circle_sprite_buffer);
        self.sprite_vertex_buffers.push(square_sprite_buffer);
    }
}

/// Returns a triangle strip for a circle with radius 1
pub fn new_circle(display: &glium::Display, num_segments: usize) -> glium::VertexBuffer<Vertex> {
    let mut shape = Vec::with_capacity(num_segments + 1);
    
    shape.push(Vertex { position: [0.0, 0.0] });

    let alpha = 2.0 * f32::consts::PI / ((num_segments - 1) as f32);
    for i in 0..num_segments {
        let beta = alpha * i as f32;
        let p = [beta.cos(), beta.sin()];
        shape.push(Vertex { position: p });
    }

    glium::VertexBuffer::new(display, &shape).unwrap()
}

/// Returns a centered 1x1 square
pub fn new_square(display: &glium::Display) -> (glium::VertexBuffer<Vertex>,
                                                glium::IndexBuffer<u16>) {
    let vertices = vec![
        Vertex { position: [-0.5, -0.5] },
        Vertex { position: [-0.5, 0.5] },
        Vertex { position: [0.5, 0.5] },
        Vertex { position: [0.5, -0.5] },
    ];
    let indices = vec![0, 1, 2,
                       0, 2, 3];
    
    (glium::VertexBuffer::new(display, &vertices).unwrap(),
     glium::IndexBuffer::new(display, glium::index::PrimitiveType::TrianglesList, &indices).unwrap())
}

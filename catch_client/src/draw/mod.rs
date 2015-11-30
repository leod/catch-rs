use std::f32;

use na::Mat4;
use glium;

mod draw_list;
mod draw_draw_list;

pub use self::draw_list::{DrawElement, DrawAttributes, DrawList};
pub use self::draw_draw_list::DrawDrawList;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}

pub struct DrawContext<'a> {
    pub proj_mat: Mat4<f32>,
    pub camera_mat: Mat4<f32>,
    pub parameters: glium::DrawParameters<'a>,
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
     glium::IndexBuffer::new(display,
                             glium::index::PrimitiveType::TrianglesList,
                             &indices).unwrap())
}

use std::f32;

use na::Mat4;

use glium::{self, VertexBuffer, IndexBuffer};
use glium::backend::Facade;
use glium::index::PrimitiveType;

mod draw_list;
mod draw_draw_list;

pub use self::draw_list::{DrawElement, DrawAttributes, DrawList};
pub use self::draw_draw_list::DrawDrawList;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}

implement_vertex!(Vertex, position);

#[derive(Copy, Clone)]
pub struct TexVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

implement_vertex!(TexVertex, position, tex_coords);

pub struct DrawContext<'a> {
    pub proj_mat: Mat4<f32>,
    pub camera_mat: Mat4<f32>,
    pub parameters: glium::DrawParameters<'a>,
}

/// Returns a triangle strip for a circle with radius 1
pub fn new_circle<F: Facade + Clone>(facade: &F, num_segments: usize) -> VertexBuffer<Vertex> {
    let mut shape = Vec::with_capacity(num_segments + 1);
    
    shape.push(Vertex { position: [0.0, 0.0] });

    let alpha = 2.0 * f32::consts::PI / ((num_segments - 1) as f32);
    for i in 0..num_segments {
        let beta = alpha * i as f32;
        let p = [beta.cos(), beta.sin()];
        shape.push(Vertex { position: p });
    }

    VertexBuffer::new(facade, &shape).unwrap()
}

/// Returns a centered 1x1 square
pub fn new_square<F: Facade + Clone>(facade: &F) -> (VertexBuffer<Vertex>, IndexBuffer<u16>) {
    let vertices = vec![
        Vertex { position: [-0.5, -0.5] },
        Vertex { position: [-0.5, 0.5] },
        Vertex { position: [0.5, 0.5] },
        Vertex { position: [0.5, -0.5] },
    ];
    let indices = vec![0, 1, 2,
                       0, 2, 3];
    
    (VertexBuffer::new(facade, &vertices).unwrap(),
     IndexBuffer::new(facade, PrimitiveType::TrianglesList, &indices).unwrap())
}

use std::f32;
use std::collections::HashMap;
use std::error::Error;
use std::slice;

use na::{Norm, Vec2, Mat2, Vec4, Mat4};
use glium;
use image;

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

#[derive(Clone, Debug)]
pub enum DrawElement {
    Circle,
    Square,
    TexturedSquare { texture: String },
}

#[derive(Copy, Clone, Debug)]
pub struct DrawAttributes {
    pub z: f32,
    pub color: Vec4<f32>,
    pub model_mat: Mat4<f32>,
    pub texture_id: u32,
}
implement_vertex!(DrawAttributes, z, color, model_mat, texture_id);

impl DrawAttributes {
    pub fn new(z: f32, color: Vec4<f32>, model_mat: Mat4<f32>) -> DrawAttributes {
        DrawAttributes {
            z: z,
            color: color,
            model_mat: model_mat,
            texture_id: 0,
        }
    }
}

pub struct DrawList {
    list: Vec<(DrawElement, DrawAttributes)>,
}

impl DrawList {
    pub fn new() -> DrawList {
        DrawList {
            list: Vec::new()
        }
    }

    pub fn iter(&self) -> slice::Iter<(DrawElement, DrawAttributes)> {
        self.list.iter()
    }

    pub fn sort_by_z(&mut self) {
        self.list.sort_by(|a, b| a.1.z.partial_cmp(&b.1.z).unwrap());
    }

    pub fn push(&mut self, element: DrawElement, attributes: DrawAttributes) {
        self.list.push((element, attributes));
    }

    pub fn push_line(&mut self,
                     color: Vec4<f32>,
                     size: f32,
                     a: Vec2<f32>,
                     b: Vec2<f32>,
                     z: f32) {
        let d = b - a;
        let alpha = d.y.atan2(d.x);

        let rot_mat = Mat2::new(alpha.cos(), -alpha.sin(),
                                alpha.sin(), alpha.cos());
        let scale_mat = Mat2::new(d.norm(), 0.0,
                                  0.0, size);
        let m = rot_mat * scale_mat;
        let o = m * Vec2::new(0.5, 0.0);
        let model_mat = Mat4::new(m.m11, m.m12, 0.0, a.x + o.x,
                                  m.m21, m.m22, 0.0, a.y + o.y,
                                  0.0, 0.0, 1.0, 0.0,
                                  0.0, 0.0, 0.5, 1.0);
        self.push(DrawElement::Square, DrawAttributes::new(z, color, model_mat));
    }

    pub fn push_rect(&mut self,
                     color: Vec4<f32>,
                     width: f32,
                     height: f32,
                     p: Vec2<f32>,
                     z: f32,
                     angle: f32) {
        let rot_mat = Mat2::new(angle.cos(), -angle.sin(),
                                angle.sin(), angle.cos());
        let scale_mat = Mat2::new(width, 0.0,
                                  0.0, height);
        let m = rot_mat * scale_mat;
        let model_mat = Mat4::new(m.m11, m.m12, 0.0, p.x,
                                  m.m21, m.m22, 0.0, p.y,
                                  0.0, 0.0, 1.0, 0.0,
                                  0.0, 0.0, 0.0, 1.0);
        self.push(DrawElement::Square, DrawAttributes::new(z, color, model_mat));
    }

    pub fn push_ellipse(&mut self,
                        color: Vec4<f32>,
                        width: f32,
                        height: f32,
                        p: Vec2<f32>,
                        z: f32,
                        angle: f32) {
        let rot_mat = Mat2::new(angle.cos(), -angle.sin(),
                                angle.sin(), angle.cos());
        let scale_mat = Mat2::new(width, 0.0,
                                  0.0, height);
        let m = rot_mat * scale_mat;
        let model_mat = Mat4::new(m.m11, m.m12, 0.0, p.x,
                                  m.m21, m.m22, 0.0, p.y,
                                  0.0, 0.0, 1.0, 0.0,
                                  0.0, 0.0, 0.0, 1.0);
        self.push(DrawElement::Circle, DrawAttributes::new(z, color, model_mat));
    }
}

pub const SPRITE_VERTEX_BUFFER_SIZE: usize = 1024;

const TEXTURES: &'static [&'static str] = &[
    "shield"
];

pub struct DrawDrawList {
    circle_vertex_buffer: glium::VertexBuffer<Vertex>,
    square_vertex_buffer: glium::VertexBuffer<Vertex>,
    square_index_buffer: glium::IndexBuffer<u16>,

    sprite_vertex_buffers: Vec<glium::VertexBuffer<DrawAttributes>>,

    program: glium::Program,
    textured_square_program: glium::Program,

    texture_ids: HashMap<String, u32>,
    texture_array: glium::texture::Texture2dArray,
}

impl DrawDrawList {
    pub fn new(display: &glium::Display) -> Result<DrawDrawList, String> {
        let vertex_shader_src = r#"
            #version 140

            in vec2 position;

            uniform mat4 proj_mat;
            uniform mat4 camera_mat;

            in float z;
            in vec4 color;
            in mat4 model_mat;

            out vec4 color_v;
            
            void main() {
                color_v = color;
                vec4 p = camera_mat * model_mat * vec4(position, 0.0, 1.0);
                p.z = z;
                gl_Position = proj_mat * p;
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            in vec4 color_v;

            out vec4 color_f;

            void main() {
                color_f = color_v;
            }
        "#;

        let textured_square_vertex_shader_src = r#"
            #version 140

            in vec2 position;

            uniform mat4 proj_mat;
            uniform mat4 camera_mat;

            in float z;
            in mat4 model_mat;
            in uint texture_id;

            out vec2 texture_coords_v;
            flat out uint texture_id_v;
            
            void main() {
                if (gl_VertexID % 4 == 0) {
                    texture_coords_v = vec2(0.0, 1.0);
                } else if (gl_VertexID % 4 == 1) {
                    texture_coords_v = vec2(0.0, 0.0);
                } else if (gl_VertexID % 4 == 2) {
                    texture_coords_v = vec2(1.0, 0.0);
                } else {
                    texture_coords_v = vec2(1.0, 1.0);
                }

                texture_id_v = texture_id;

                vec4 p = camera_mat * model_mat * vec4(position, 0.0, 1.0);
                p.z = z;
                gl_Position = proj_mat * p;
            }
        "#;

        let textured_square_fragment_shader_src = r#"
            #version 140

            uniform sampler2DArray textures;

            in vec2 texture_coords_v;
            flat in uint texture_id_v;

            out vec4 color_f;

            void main() {
                color_f = texture(textures, vec3(texture_coords_v, float(texture_id_v)));
            }
        "#;

        let (square_vertex_buffer, square_index_buffer) = new_square(display);
        let sprite_vertex_buffer =
            glium::VertexBuffer::empty_dynamic(display, SPRITE_VERTEX_BUFFER_SIZE).unwrap();

        let program =
            glium::Program::from_source(display,
                                        vertex_shader_src,
                                        fragment_shader_src,
                                        None).unwrap();
        let textured_square_program =
            glium::Program::from_source(display,
                                        textured_square_vertex_shader_src,
                                        textured_square_fragment_shader_src,
                                        None).unwrap();
        let (texture_ids, texture_array) = try!(DrawDrawList::load_textures(display));

        Ok(DrawDrawList {
            circle_vertex_buffer: new_circle(display, 32),
            square_vertex_buffer: square_vertex_buffer,
            square_index_buffer: square_index_buffer,
            sprite_vertex_buffers: vec![sprite_vertex_buffer],
            program: program,
            textured_square_program: textured_square_program,
            texture_ids: texture_ids,
            texture_array: texture_array,
        })
    }

    fn load_textures(display: &glium::Display)
                     -> Result<(HashMap<String, u32>, glium::texture::Texture2dArray), String> {
        let mut texture_ids = HashMap::new();
        let mut images = Vec::new();

        for (id, texture_name) in TEXTURES.iter().enumerate() {
            let image = image::open("data/textures/".to_string() + texture_name + ".png");
            match image {
                Ok(image) => {
                    texture_ids.insert(texture_name.to_string(), id as u32);

                    let raw_image = match image {
                        image::DynamicImage::ImageRgba8(image) => {
                            let dimensions = image.dimensions();
                            let raw = image.into_raw();

                            info!("loaded texture \"{}\" with dimensions {:?} (id: {})",
                                  texture_name, dimensions, id);

                            glium::texture::RawImage2d::from_raw_rgba(raw, dimensions)
                        }
                        _ => {
                            return Err(format!("unsupported image: {}",
                                               texture_name).to_string());
                        }
                    };

                    images.push(raw_image);
                }
                Err(error) => {
                    return Err(format!("failed to load texture {}: {}", texture_name,
                                       error.description()).to_string());
                }
            };
        }
        
        Ok((texture_ids,
            glium::texture::Texture2dArray::new(display, images).unwrap()))
    }

    fn get_sprite_vertex_buffer(&mut self, display: &glium::Display)
                                -> glium::VertexBuffer<DrawAttributes> {
        if let Some(vertex_buffer) = self.sprite_vertex_buffers.pop() {
            vertex_buffer
        } else {
            info!("creating new vertex buffer for draw list");
            glium::VertexBuffer::empty_dynamic(display, SPRITE_VERTEX_BUFFER_SIZE).unwrap()
        }
    }

    fn draw_some<'a,
                 'b,
                 I: Iterator<Item=&'b (DrawElement, DrawAttributes)>,
                 S: glium::Surface>
                 (&mut self,
                  list: I,
                  context: &DrawContext<'a>,
                  display: &glium::Display,
                  surface: &mut S)
                  -> Vec<glium::VertexBuffer<DrawAttributes>> {
        let mut circle_sprite_buffer = self.get_sprite_vertex_buffer(display);
        let mut square_sprite_buffer = self.get_sprite_vertex_buffer(display);
        let mut textured_square_sprite_buffer = self.get_sprite_vertex_buffer(display);

        let mut circle_i = 0;
        let mut square_i = 0;
        let mut textured_square_i = 0;

        {
            let mut circle_mapping = circle_sprite_buffer.map();
            let mut square_mapping = square_sprite_buffer.map();
            let mut textured_square_mapping = textured_square_sprite_buffer.map();

            // Create batches
            for &(ref element, ref attributes) in list {
                match element {
                    &DrawElement::Circle => {
                        if circle_i == SPRITE_VERTEX_BUFFER_SIZE {
                            assert!(false);
                        }
                        circle_mapping[circle_i] = attributes.clone();
                        circle_i += 1;
                    },
                    &DrawElement::Square => {
                        square_mapping[square_i] = attributes.clone();
                        square_i += 1;
                    },
                    &DrawElement::TexturedSquare { ref texture } => {
                        let texture_id = self.texture_ids[texture];
                        textured_square_mapping[textured_square_i] = attributes.clone();
                        textured_square_mapping[textured_square_i].texture_id = texture_id;
                        textured_square_i += 1;
                    }
                }
            }
        }

        let uniforms = uniform! {
            proj_mat: context.proj_mat,
            camera_mat: context.camera_mat,
            textures: &self.texture_array,
        };
        let parameters = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            .. context.parameters.clone()
        };

        surface.draw((&self.square_vertex_buffer,
                      square_sprite_buffer.slice(0..square_i).unwrap().per_instance().unwrap()),
                     &self.square_index_buffer, &self.program, &uniforms,
                     &parameters).unwrap();

        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleFan);
        surface.draw((&self.circle_vertex_buffer,
                      circle_sprite_buffer.slice(0..circle_i).unwrap().per_instance().unwrap()),
                     &indices, &self.program, &uniforms, &parameters).unwrap();

        surface.draw((&self.square_vertex_buffer,
                      textured_square_sprite_buffer.slice(0..textured_square_i)
                                                   .unwrap().per_instance().unwrap()),
                     &self.square_index_buffer, &self.textured_square_program, &uniforms,
                     &parameters).unwrap();

        // Allow buffers to be reused next frame
        vec![circle_sprite_buffer, square_sprite_buffer, textured_square_sprite_buffer]
    }

    pub fn draw<'a,
                S: glium::Surface>
               (&mut self,
                list: DrawList,
                context: &DrawContext<'a>,
                display: &glium::Display,
                surface: &mut S) {
        // TODO: This stops working as soon as we require more than one buffer of a type.

        let mut list = list;
        list.sort_by_z();

        let used_buffers = self.draw_some(list.iter(), context, display, surface);

        for buffer in used_buffers.into_iter() {
            self.sprite_vertex_buffers.push(buffer);
        }
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

use std::error::Error;
use std::collections::HashMap;

use glium;
use image;

use draw::{self, DrawContext, DrawList, DrawElement, DrawAttributes, Vertex};

implement_vertex!(Vertex, position);

const SPRITE_VERTEX_BUFFER_SIZE: usize = 1024;

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

        let (square_vertex_buffer, square_index_buffer) = draw::new_square(display);
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
            circle_vertex_buffer: draw::new_circle(display, 32),
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

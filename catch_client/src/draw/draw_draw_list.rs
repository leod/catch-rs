use std::rc::Rc;
use std::error::Error;
use std::collections::HashMap;

use glium::{self, Surface, VertexBuffer, IndexBuffer, Program};
use glium::index::{PrimitiveType, NoIndices};
use glium::backend::{Facade, Context};
use glium::texture::{Texture2dArray, RawImage2d};

use image;

use draw::{self, DrawFlags, FLAG_NONE, FLAG_BLUR, DrawContext, DrawList, DrawElement, DrawAttributes, Vertex};

const SPRITE_VERTEX_BUFFER_SIZE: usize = 4096;

const TEXTURES: &'static [&'static str] = &[
    "shield"
];

pub struct DrawDrawList {
    context: Rc<Context>,

    circle_vertex_buffer: VertexBuffer<Vertex>,
    square_vertex_buffer: VertexBuffer<Vertex>,
    square_index_buffer: IndexBuffer<u16>,

    sprite_vertex_buffers: Vec<VertexBuffer<DrawAttributes>>,

    program: Program,
    textured_square_program: Program,

    texture_ids: HashMap<String, u32>,
    texture_array: Texture2dArray,
}

impl DrawDrawList {
    pub fn new<F: Facade + Clone>(facade: &F) -> Result<DrawDrawList, String> {
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

        let (square_vertex_buffer, square_index_buffer) = draw::new_square(facade);
        let sprite_vertex_buffer =
            VertexBuffer::empty_dynamic(facade, SPRITE_VERTEX_BUFFER_SIZE).unwrap();

        let program = Program::from_source(facade, vertex_shader_src, fragment_shader_src,
                                           None).unwrap();
        let textured_square_program = Program::from_source(facade,
                                                           textured_square_vertex_shader_src,
                                                           textured_square_fragment_shader_src,
                                                           None).unwrap();
        let (texture_ids, texture_array) = try!(DrawDrawList::load_textures(facade));

        Ok(DrawDrawList {
            context: facade.get_context().clone(),
            circle_vertex_buffer: draw::new_circle(facade, 32),
            square_vertex_buffer: square_vertex_buffer,
            square_index_buffer: square_index_buffer,
            sprite_vertex_buffers: vec![sprite_vertex_buffer],
            program: program,
            textured_square_program: textured_square_program,
            texture_ids: texture_ids,
            texture_array: texture_array,
        })
    }

    fn load_textures<F: Facade + Clone>
                    (facade: &F)
                    -> Result<(HashMap<String, u32>, Texture2dArray), String> {
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

                            RawImage2d::from_raw_rgba(raw, dimensions)
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
        
        Ok((texture_ids, Texture2dArray::new(facade, images).unwrap()))
    }

    fn get_sprite_vertex_buffer(&mut self) -> VertexBuffer<DrawAttributes> {
        if let Some(vertex_buffer) = self.sprite_vertex_buffers.pop() {
            vertex_buffer
        } else {
            info!("creating new vertex buffer for draw list");
            VertexBuffer::empty_dynamic(&self.context, SPRITE_VERTEX_BUFFER_SIZE).unwrap()
        }
    }

    fn draw_some<'a,
                 'b,
                 I: Iterator<Item=&'b (DrawElement, DrawAttributes)>,
                 S: Surface>
                (&mut self, list: I, context: &DrawContext<'a>, surface: &mut S)
                 -> Vec<glium::VertexBuffer<DrawAttributes>> {
        let mut circle_sprite_buffer = self.get_sprite_vertex_buffer();
        let mut square_sprite_buffer = self.get_sprite_vertex_buffer();
        let mut textured_square_sprite_buffer = self.get_sprite_vertex_buffer();

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

        let indices = NoIndices(PrimitiveType::TriangleFan);
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
                S: Surface>
               (&mut self, flags: DrawFlags, list: DrawList, context: &DrawContext<'a>, surface: &mut S) {
        // TODO: This stops working as soon as we require more than one buffer of a type.

        let mut list = list;
        list.sort_by_z();

        let used_buffers = if flags != FLAG_NONE {
            let iter = list.iter().filter(|&&(_, attributes)| attributes.flags & flags.bits() == flags.bits());
            self.draw_some(iter, context, surface)
        } else {
            let iter = list.iter();
            self.draw_some(list.iter(), context, surface)
        };

        for buffer in used_buffers.into_iter() {
            self.sprite_vertex_buffers.push(buffer);
        }
    }
}

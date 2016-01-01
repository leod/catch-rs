use std::cell::RefCell;
use std::rc::Rc;

use glium::{Surface, VertexBuffer, IndexBuffer, Program};
use glium::backend::{Facade, Context};
use glium::texture::{Texture2d, DepthFormat};
use glium::index::PrimitiveType;
use glium::framebuffer::{SimpleFrameBuffer, DepthRenderBuffer};

use draw::{DrawOp, TexVertex};

pub struct Post {
    settings: PostSettings,
    context: Rc<Context>,
    vertex_buffer: VertexBuffer<TexVertex>,
    index_buffer: IndexBuffer<u16>,
    program: Program,
    target_color: RefCell<Option<Texture2d>>,
    target_depth: RefCell<Option<DepthRenderBuffer>>,
}

pub struct PostSettings {
    pub blur: bool,
}

impl Post {
    pub fn new<F: Facade + Clone>(settings: PostSettings, facade: &F) -> Post {
        // TODO: Blur in two passes

        let vertex_shader_src = r#"
            #version 140

            in vec2 position;
            in vec2 tex_coords;

            out vec2 tex_coords_v;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                tex_coords_v = tex_coords;
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            uniform vec2 resolution;
            uniform sampler2D tex;

            in vec2 tex_coords_v;

            float norm(float x, float sigma) {
                return 0.39894 * exp(-0.5*x*x/(sigma*sigma))/sigma;
            }

            void main() {
                vec2 frag_coords = tex_coords_v * resolution;
                vec2 inv_res = 1.0 / resolution;

                const int m_size = 11;
                const int k_size = (m_size - 1)/2;
                const float sigma = 7.0;

                float kernel[m_size];
                for (int j = 0; j <= k_size; j++) {
                    kernel[k_size+j] = kernel[k_size-j] = norm(float(j), sigma);
                }

                float z = 0.0;
                for (int j = 0; j < m_size; j++) {
                    z += kernel[j];
                }

                vec3 final_color = vec3(0.0);
                for (int i = -k_size; i <= k_size; i++) {
                    for (int j = -k_size; j <= k_size; j++) {
                        vec2 p = tex_coords_v + vec2(float(i), float(j)) * inv_res;
                        final_color += kernel[k_size+j]*kernel[k_size+i] * texture2D(tex, p).rgb;
                    }
                }

                gl_FragColor = vec4(final_color / (z*z), 1.0);
            }
        "#;

        Post {
            settings: settings,
            context: facade.get_context().clone(),
            vertex_buffer: VertexBuffer::new(facade,
                &[TexVertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
                  TexVertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
                  TexVertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                  TexVertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] }]).unwrap(),
            index_buffer: IndexBuffer::new(facade, PrimitiveType::TriangleStrip,
                                           &[1 as u16, 2, 0, 3]).unwrap(),
            program: Program::from_source(facade, vertex_shader_src, fragment_shader_src,
                                          None).unwrap(),
            target_color: RefCell::new(None),
            target_depth: RefCell::new(None),
        }
    }

    pub fn draw<S: Surface,
                R,
                F: DrawOp<Result=R>> 
               (&self, target: &mut S, mut draw: &mut F) -> R { 
        // adapted from https://github.com/tomaka/glium/blob/master/examples/fxaa.rs#L150

        let target_dimensions = target.get_dimensions();

        let mut target_color = self.target_color.borrow_mut();
        let mut target_depth = self.target_depth.borrow_mut();

        {
            let clear = if let &Some(ref tex) = &*target_color {
                tex.get_width() != target_dimensions.0 ||
                    tex.get_height().unwrap() != target_dimensions.1
            } else {
                false
            };
            if clear { *target_color = None; }
        }

        {
            let clear = if let &Some(ref tex) = &*target_depth {
                tex.get_dimensions() != target_dimensions
            } else {
                false
            };
            if clear { *target_depth = None; }
        }

        if target_color.is_none() {
            let texture = Texture2d::empty(&self.context,
                                           target_dimensions.0 as u32,
                                           target_dimensions.1 as u32).unwrap();
            *target_color = Some(texture);
        }
        let target_color = target_color.as_ref().unwrap();

        if target_depth.is_none() {
            let texture = DepthRenderBuffer::new(&self.context,
                                                 DepthFormat::I24,
                                                 target_dimensions.0 as u32,
                                                 target_dimensions.1 as u32).unwrap();
            *target_depth = Some(texture);
        }
        let target_depth = target_depth.as_ref().unwrap();

        let result = draw.draw(&mut SimpleFrameBuffer::with_depth_buffer(&self.context,
                                                                         target_color,
                                                                         target_depth).unwrap());

        let uniforms = uniform! {
            tex: &*target_color,
            resolution: (target_dimensions.0 as f32, target_dimensions.1 as f32)
        };
        
        target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms,
                    &Default::default()).unwrap();

        result
    }
}

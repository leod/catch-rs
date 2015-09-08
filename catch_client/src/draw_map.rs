use gl;

use std::path::Path;

use graphics::Transformed;
use graphics::{Context, Image};
use opengl_graphics::{GlGraphics, Texture};

use shared::map::{Map, LayerId, Tile};

pub struct DrawMap {
    tileset_textures: Vec<Texture>
}

impl DrawMap {
    pub fn load(map: &Map) -> Result<DrawMap, String> {
        let mut textures = Vec::new();

        for pathname in map.tileset_image_paths().iter() {
            // TODO: data paths
            let full_pathname = "../data/maps/".to_string() + &pathname;

            let path = Path::new(&full_pathname);
            //println!("{}", full_pathname);

            match Texture::from_path(path) {
                Err(error) => return Err(error),
                Ok(texture) => textures.push(texture)
            };

            unsafe {
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_MIN_FILTER,
                    gl::NEAREST as i32
                    );
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_MAG_FILTER,
                    gl::NEAREST as i32
                    );
            }
        }

        Ok(DrawMap {
            tileset_textures: textures
        })
    }

    pub fn draw(&self, map: &Map, c: Context, gl: &mut GlGraphics) {
        self.draw_layer(map, LayerId::Floor, c, gl);
        self.draw_layer(map, LayerId::Block, c, gl);
    }

    fn draw_layer(&self, map: &Map, id: LayerId, c: Context, gl: &mut GlGraphics) {
        let width = map.tile_width();
        let height = map.tile_height();

        for (tile_x, tile_y, tile) in map.iter_layer(id) {
            match tile {
                Some(Tile { tileset: _, x: tileset_x, y: tileset_y }) => {
                    let image = Image::new().rect([0.0, 0.0,
                                                   (width as f64),
                                                   (height as f64)])
                                            .src_rect([(tileset_x * width) as i32,
                                                       (tileset_y * height) as i32,
                                                       (width as i32),
                                                       (height as i32)]);
                    let texture = &self.tileset_textures[id.to_index()];
                    let transform = c./*zoom(10.0).*/trans((tile_x * width) as f64, (tile_y * height) as f64).transform;
                    image.draw(texture, &c.draw_state, transform, gl);
                }
                None => continue
            }
        }
    }
}

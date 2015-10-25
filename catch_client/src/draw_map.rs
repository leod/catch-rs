use graphics::Context;
use opengl_graphics::GlGraphics;

use shared::map::Map;

pub struct DrawMap;

impl DrawMap {
    pub fn load(_map: &Map) -> Result<DrawMap, String> {
        Ok(DrawMap)
    }

    pub fn draw(&self, _map: &Map, _c: Context, _gl: &mut GlGraphics) {
    }
}

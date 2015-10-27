use glium::Surface;

use shared::map::Map;

pub struct DrawMap;

impl DrawMap {
    pub fn load(_map: &Map) -> Result<DrawMap, String> {
        Ok(DrawMap)
    }

    pub fn draw<S: Surface>(&self, _map: &Map, _surface: &mut S) {
    }
}

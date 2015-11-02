use std::fs::File;
use std::path::Path;

use tiled;

#[derive(Copy, Clone)]
pub struct Tile {
    pub tileset: usize,
    // Position in tileset in tiles
    pub x: usize,
    pub y: usize
}

struct Layer {
    pub tiles: Vec<Vec<Option<Tile>>>
}

pub struct Map {
    map: tiled::Map,
    layers: Vec<Layer>,
    pub objects: Vec<MapObject>,
}

/// Information about an entity on a map
pub struct MapObject {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub type_str: String,
}

#[derive(Copy, Clone)]
pub enum LayerId {
    Floor,
    Block
}

impl LayerId {
    pub fn to_index(&self) -> usize {
        match self {
            &LayerId::Floor => 0,
            &LayerId::Block => 1
        }
    }
}

impl Map {
    pub fn width(&self) -> usize {
        self.map.width as usize
    }

    pub fn height(&self) -> usize {
        self.map.height as usize
    }

    pub fn tile_width(&self) -> usize {
        self.map.tile_width as usize
    }

    pub fn tile_height(&self) -> usize {
        self.map.tile_height as usize
    }

    pub fn width_pixels(&self) -> usize {
        self.width() * self.tile_width()
    }

    pub fn height_pixels(&self) -> usize {
        self.height() * self.tile_height()
    }
    
    pub fn is_pos_valid(&self, x: usize, y: usize) -> bool {
        x < self.map.width as usize && y < self.map.height as usize
    }

    pub fn get_tile(&self, layer_id: LayerId, x: usize, y: usize) -> Option<Tile> {
        self.layers[layer_id.to_index()].tiles[y][x]
    }

    pub fn tileset_image_paths(&self) -> Vec<String> {
        self.map.tilesets.iter()
            .map(|tileset| tileset.images[0].source.clone())
            .collect()
    }

    /// Loads a tiled map from the given path. If the file is not found or has an invalid format,
    /// Err is returned.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Map, String> {
        info!("loading map {}", path.as_ref().to_str().unwrap());

        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Err("Failed to open file".to_string())
        };

        match tiled::parse(file) {
            Ok(map) => Map::from_tiled(map),
            Err(_) => Err("Invalid map".to_string())
        }
    }

    fn from_tiled(map: tiled::Map) -> Result<Map, String> {
        if map.layers.len() != 2 {
            return Err("Too many layers in the map".to_string());
        }

        let layers = map.layers.iter().map(|layer| {
            Map::convert_layer(&map.tilesets, &layer)
        }).collect();

        let objects = try!(Map::convert_objects(&map.object_groups));

        Ok(Map {
            map: map,
            layers: layers,
            objects: objects,
        })
    }

    fn tile_from_number(tilesets: &Vec<tiled::Tileset>,
                        number: u32) 
                        -> Option<Tile> {
        for (i, tileset) in tilesets.iter().enumerate() {
            let num_tiles_w = tileset.images[0].width as usize /
                              tileset.tile_width as usize;
            let num_tiles_h = tileset.images[0].height as usize / 
                              tileset.tile_height as usize;
            let num_tiles = num_tiles_w * num_tiles_h;

            if number >= tileset.first_gid &&
               number < tileset.first_gid + num_tiles as u32 {
                let number_rel = number as usize -
                                 tileset.first_gid as usize;
                let x = number_rel % num_tiles_h;
                let y = number_rel / num_tiles_w;

                return Some(Tile { tileset: i,
                                   x: x,
                                   y: y });
            }
        }

        None
    }

    /// Converts a `tiled::Layer` into our type `Layer`
    fn convert_layer(tilesets: &Vec<tiled::Tileset>,
                     layer: &tiled::Layer) -> Layer {
        let tiles = layer.tiles.iter().map(|row| {
            row.iter().map(|&number| {
                Map::tile_from_number(tilesets, number)
            }).collect()
        }).collect();

        Layer {
            tiles: tiles,
        }
    }

    /// Converts from tiled's MapObject to ours
    fn convert_objects(object_groups: &Vec<tiled::ObjectGroup>) 
                       -> Result<Vec<MapObject>, String> {
        let mut objects = Vec::new();
        for object_group in object_groups.iter() {
            for object in object_group.objects.iter() {
                match object {
                    &tiled::Object::Rect { ref x, ref y, ref width, ref height,
                                           ref type_str, visible: _ } => {
                        objects.push(MapObject {
                            x: *x as f32,
                            y: *y as f32,
                            width: *width as f32,
                            height: *height as f32,
                            type_str: type_str.clone(),
                        });
                    }
                    _ =>
                        return Err("Only rectangle objects can be used".to_string())
                }
            }
        }

        Ok(objects)
    }
}

use std::io;
use std::fs::File;
use std::path::Path;

use tiled;

use super::math;

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

pub struct TileIter<'a> {
    map: &'a Map,
    layer: usize,

    x: u32,
    y: u32
}

impl<'a> TileIter<'a> {
    fn new(map: &'a Map, layer: usize) -> TileIter<'a> {
        TileIter {
            map: map,
            layer: layer,
            x: 0,
            y: 0
        }
    }
}

impl<'a> Iterator for TileIter<'a> {
    type Item = (usize, usize, Option<Tile>);

    fn next(&mut self) -> Option<(usize, usize, Option<Tile>)> {
        if self.y == self.map.map.height {
            return None;
        }

        let (x, y) = (self.x as usize, self.y as usize);
        let tile = self.map.layers[self.layer].tiles[y][x];

        self.x += 1;

        if self.x == self.map.map.width {
            self.x = 0;
            self.y += 1;
        }

        Some((x as usize, y as usize, tile))
    }
}

impl Map {
    fn convert_layer(tilesets: &Vec<tiled::Tileset>,
                     layer: &tiled::Layer) -> Layer {
        let tiles = layer.tiles.iter().map(|row| {
            row.iter().map(|&number| {
                if number == 0 {
                    None
                } else {
                    for i in 0..tilesets.len() {
                        let tileset = &tilesets[i];
                        let num_tiles_w = tileset.images[0].width as usize /
                                          tileset.tile_width as usize;
                        let num_tiles_h = tileset.images[0].width as usize / 
                                          tileset.tile_height as usize;
                        let num_tiles = num_tiles_w * num_tiles_h;

                        if number >= tileset.first_gid &&
                           number < tileset.first_gid + num_tiles as u32 {
                            let number_rel = number as usize -
                                             tileset.first_gid as usize;
                            let x = number_rel % num_tiles_w;
                            let y = number_rel / num_tiles_h;

                            return Some(Tile {
                                tileset: i,
                                x: x,
                                y: y
                            });
                        }
                    }

                    return None;
                }
            }).collect()
        }).collect();

        Layer {
            tiles: tiles,
        }
    }

    pub fn tile_width(&self) -> usize {
        self.map.tile_width as usize
    }

    pub fn tile_height(&self) -> usize {
        self.map.tile_height as usize
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

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Map, String> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Err("Failed to open file".to_string())
        };

        match tiled::parse(file) {
            Ok(map) => Map::from_tiled(map),
            Err(_) => Err("Invalid map".to_string())
        }
    }

    pub fn from_tiled(map: tiled::Map) -> Result<Map, String> {
        if map.layers.len() != 2 {
            return Err("Too many layers in the map".to_string());
        }

        let layers = map.layers.iter().map(|layer| {
            Map::convert_layer(&map.tilesets, &layer)
        }).collect();

        Ok(Map {
            map: map,
            layers: layers,
        })
    }

    pub fn iter_layer<'a>(&'a self, id: LayerId) -> TileIter<'a> {
        TileIter::new(self, id.to_index())
    }

    pub fn line_segment_intersection(&self, p: math::Vec2, q: math::Vec2) -> Option<(usize, usize, math::Vec2, f64)> {
        let mut i_min = None;

        for (x_i, y_i, tile) in self.iter_layer(LayerId::Block) {
            if let Some(_) = tile {
                let x = (x_i * self.tile_width()) as f64;
                let y = (y_i * self.tile_width()) as f64;
                let w = self.tile_width() as f64;
                let h = self.tile_height() as f64;

                let i1 = math::line_segments_intersection(p, q, [x, y], [x+w, y])
                             .map(|t| ([0.0, -1.0], t));

                let i2 = math::line_segments_intersection(p, q, [x, y], [x, y+h])
                             .map(|t| ([-1.0, 0.0], t));

                let i3 = math::line_segments_intersection(p, q, [x+w, y], [x+w, y+h])
                             .map(|t| ([1.0, 0.0], t));

                let i4 = math::line_segments_intersection(p, q, [x, y+h], [x+w, y+h])
                             .map(|t| ([0.0, 1.0], t));

                let i = math::min_intersection(math::min_intersection(i1, i2),
                                               math::min_intersection(i3, i4))
                             .map(|(n, t)| ((x_i, y_i, n), t));

                i_min = math::min_intersection(i_min, i);
            }
        }

        /*if i_min.is_some() {
            println!("{:?}", i_min.unwrap());
        }*/

        i_min.map(|((x, y, n), t)| (x, y, n, t))
    }
}

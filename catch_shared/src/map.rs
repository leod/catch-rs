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

pub struct TileIter<'a> {
    map: &'a Map,
    layer: usize,
    tile_rect: [usize; 4],

    x: usize,
    y: usize,
}

impl<'a> TileIter<'a> {
    fn new(map: &'a Map, layer: usize, tile_rect: [isize; 4]) -> TileIter<'a> {
        let tile_rect = TileIter::clip_rect(map, tile_rect);
        TileIter {
            map: map,
            layer: layer,
            tile_rect: tile_rect,
            x: tile_rect[0],
            y: tile_rect[1],
        }
    }

    fn clip_rect(map: &'a Map, tile_rect: [isize; 4]) -> [usize; 4] {
        let min_x = if tile_rect[0] >= 0 { tile_rect[0] as usize } else { 0 };
        let min_y = if tile_rect[1] >= 0 { tile_rect[1] as usize } else { 0 };
        let max_x = if min_x + tile_rect[2] as usize <= map.width() {
                        min_x + tile_rect[2] as usize
                    } else {
                        map.width() 
                    };
        let max_y = if min_y + tile_rect[3] as usize <= map.height() {
                        min_y + tile_rect[3] as usize
                    } else {
                        map.height()
                    };
        [min_x, min_y, max_x, max_y]
    }
}

impl<'a> Iterator for TileIter<'a> {
    type Item = (usize, usize, Option<Tile>);

    fn next(&mut self) -> Option<(usize, usize, Option<Tile>)> {
        if self.y == self.tile_rect[3] {
            return None;
        }

        let (x, y) = (self.x, self.y);
        let tile = self.map.layers[self.layer].tiles[y][x];

        self.x += 1;

        if self.x == self.tile_rect[2] {
            self.x = self.tile_rect[0];
            self.y += 1;
        }

        Some((x, y, tile))
    }
}

pub struct TraceIter<'a> {
    map: &'a Map,
    
    // Current position in tiles
    tx: isize,
    ty: isize,

    // Progress
    t: f32,

    dt_dx: f32,
    dt_dy: f32,

    x_inc: isize,
    y_inc: isize,
    
    t_next_vertical: f32,
    t_next_horizontal: f32,
}

impl<'a> TraceIter<'a> {
    // http://playtechs.blogspot.de/2007/03/raytracing-on-grid.html

    fn new(map: &'a Map,
           ax: f32, ay: f32,
           bx: f32, by: f32) -> TraceIter<'a> {
        let tx = ax as isize / map.tile_width() as isize;
        let ty = ay as isize / map.tile_height() as isize;
        let (dx, dy) = (bx - ax, by - ay);
        let dt_dx = 1.0 / dx.abs();
        let dt_dy = 1.0 / dy.abs();

        // Calculate distances from the start point to the tile borders
        let (x_inc, t_next_horizontal) =
            if bx > ax {
                (1,
                 (((ax as usize / map.tile_width() + 1) * map.tile_width()) as f32 - ax) * dt_dx)
            } else if bx < ax {
                (-1,
                 (ax - (ax as usize / map.tile_width() * map.tile_width()) as f32) * dt_dx)
            } else {
                (0,
                 dt_dx) // Infinity
            };

        let (y_inc, t_next_vertical) =
            if by > ay {
                (1,
                 (((ay as usize / map.tile_height() + 1) * map.tile_height()) as f32 - ay) * dt_dy)
            } else if by < ay {
                (-1,
                 (ay - (ay as usize / map.tile_height() * map.tile_height()) as f32) * dt_dy)
            } else {
                (0,
                 dt_dy) // Infinity
            };

        assert!(t_next_horizontal >= 0.0);
        assert!(t_next_vertical >= 0.0);

        TraceIter {
            map: map,
            tx: tx, ty: ty,
            t: 0.0,
            dt_dx: dt_dx, dt_dy: dt_dy,
            x_inc: x_inc, y_inc: y_inc,
            t_next_horizontal: t_next_horizontal,
            t_next_vertical: t_next_vertical,
        }
    }
}

impl<'a> Iterator for TraceIter<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        if self.t > 1.0 {
            return None;
        }

        let (rx, ry) = (self.tx, self.ty);

        if rx < 0 || rx as usize >= self.map.width() ||
           ry < 0 || ry as usize >= self.map.height() {
            return None;
        }

        // Which side of the tile is crossed first?
        if self.t_next_vertical > self.t_next_horizontal {
            self.tx += self.x_inc;
            self.t = self.t_next_horizontal;
            self.t_next_horizontal += self.map.width() as f32 * self.dt_dx;
        } else {
            self.ty += self.y_inc;
            self.t = self.t_next_vertical;
            self.t_next_vertical += self.map.height() as f32 * self.dt_dy;
        }

        Some((rx as usize, ry as usize))
    }
}

// Intersection of a line segment with a tile in the map
pub struct LineSegmentIntersection {
    // Tile position
    pub tx: usize,
    pub ty: usize,

    // Can be used to find the intersection point, 0 <= t <= 1
    pub t: f32,

    // Normal at the intersection
    pub n: math::Vec2,
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

    /// Returns an iterator for all tiles contained in the given layer `id`
    pub fn iter_layer<'a>(&'a self, id: LayerId, tile_rect: [isize; 4]) -> TileIter<'a> {
        TileIter::new(self, id.to_index(), tile_rect)
    }

    /// Returns an iterator for the tiles hit by the line (in pixels) from `p` to `q`
    pub fn trace_line<'a>(&'a self, p: math::Vec2, q: math::Vec2) -> TraceIter<'a> {
        TraceIter::new(self, p[0], p[1], q[0], q[1])
    }

    /// Checks if the given line segment from `p` to `q` intersects with a blocking tile
    /// in the map
    pub fn line_segment_intersection(&self, p: math::Vec2, q: math::Vec2)
                                     -> Option<LineSegmentIntersection> {
        let mut i_min = None;

        for (x_i, y_i) in self.trace_line(p, q) {
            let tile = self.get_tile(LayerId::Block, x_i, y_i);
            if let Some(_) = tile {
                let x = (x_i * self.tile_width()) as f32;
                let y = (y_i * self.tile_width()) as f32;
                let w = self.tile_width() as f32;
                let h = self.tile_height() as f32;

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

        i_min.map(|((x, y, n), t)| LineSegmentIntersection {
            tx: x,
            ty: y,
            t: t,
            n: n,
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


#[derive(Clone, Copy)]
pub struct Coord {
    pub x: f32,
    pub y: f32
}

#[derive(Clone, Copy)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Colour {
    pub fn black() -> Self {
        Colour { r: 0, g: 0, b: 0 }
    }
    pub fn white() -> Self {
        Colour { r: 0xFF, g: 0xFF, b: 0xFF }
    }

    pub fn blend(&self, other: &Self) -> Self {
        Colour {
            r: ((self.r as u16 + other.r as u16) / 2) as u8,
            g: ((self.g as u16 + other.g as u16) / 2) as u8,
            b: ((self.b as u16 + other.b as u16) / 2) as u8
        }
    }
}

pub struct Polygon {
    pub vertices: [Coord; 3],
    pub colours: Option<[Colour; 3]>,
    pub tex_coords: Option<[Coord; 3]>
}

pub struct BoundingBox {
    pub min: Coord,
    pub max: Coord
}

/// Find cross product of the vectors a->b and a->c.
/// Since the Z component of both vectors is 0,
/// the result only contains a Z component. So the
/// result is effectively scalar.
/// 
/// This serves two purposes:
/// 
/// 1. If the result is positive, it means the point c is
/// on the correct side of the line from a->b.
/// This can be used to rasterise.
/// 
/// 2. The result also represents the area of the triangle
/// between the three points, multiplied by 2.
/// This can be used to find the weights of each point.
fn edge_function(a: &Coord, b: &Coord, c: &Coord) -> f32 {
    let x = (c.x - a.x) * (b.y - a.y);
    let y = (c.y - a.y) * (b.x - a.x);
    x - y
}

fn interpolate(vals: &[f32], parts: &[f32]) -> f32 {
    vals[0] * parts[0] + vals[1] * parts[1] + vals[2] * parts[2]
}

impl Polygon {
    fn bounding_box(&self) -> BoundingBox {
        let mut min = self.vertices[0];
        let mut max = self.vertices[0];
        for vertex in self.vertices.iter().skip(1) {
            min.x = f32::min(min.x, vertex.x);
            min.y = f32::min(min.y, vertex.y);
            max.x = f32::max(max.x, vertex.x);
            max.y = f32::max(max.y, vertex.y);
        }
        BoundingBox { min, max }
    }

    fn test_inside(&self, coord: &Coord) -> Option<[f32; 3]> {
        let mut w = [0.0; 3];
        let area = edge_function(&self.vertices[0], &self.vertices[1], &self.vertices[2]);
        for i in 0..3 {
            let j = (i + 1) % 3;
            // Cross product.
            // Check if the point is on the correct side of the line.
            let wx = edge_function(&self.vertices[i], &self.vertices[j], coord);
            if wx < 0.0 {
                // Point lies outside the triangle.
                return None;
            }
            // Represents the weight of the opposite point.
            let k = (i + 2) % 3;
            // Normalise wrt area.
            w[k] = wx / area;
        }
        Some(w)
    }

}

/// Rasterise to a 256x256 surface.
/// 
/// TODO: provide frag shader
pub fn rasterise(out: &mut [u8], polygons: &[Polygon], texture: &Texture) {
    for polygon in polygons {
        let bounding_box = polygon.bounding_box();
        for y in (bounding_box.min.y.floor() as i32)..=(bounding_box.max.y.ceil() as i32) {
            for x in (bounding_box.min.x.floor() as i32)..=(bounding_box.max.x.ceil() as i32) {
                let coord = Coord{x: x as f32, y: y as f32};
                if let Some(interp) = polygon.test_inside(&coord) {
                    // Interpolate colour.
                    let shaded_colours = polygon.colours.map(|colours| {
                        Colour {
                            r: interpolate(&colours.iter().map(|c| c.r as f32).collect::<Vec<_>>(), &interp) as u8,
                            g: interpolate(&colours.iter().map(|c| c.g as f32).collect::<Vec<_>>(), &interp) as u8,
                            b: interpolate(&colours.iter().map(|c| c.b as f32).collect::<Vec<_>>(), &interp) as u8,
                        }
                    });
                    let tex_colours = polygon.tex_coords.map(|tex_coords| {
                        let x = interpolate(&tex_coords.iter().map(|c| c.x as f32).collect::<Vec<_>>(), &interp);
                        let y = interpolate(&tex_coords.iter().map(|c| c.y as f32).collect::<Vec<_>>(), &interp);
                        let tex_x = (x as usize) % texture.x;
                        let tex_y = (y as usize) % texture.y;
                        let index = (tex_y * texture.x) + tex_x;
                        texture.colours[index]
                    });
                    let blended_colour = match (shaded_colours, tex_colours) {
                        (None, None) => Colour::black(),
                        (Some(c), None) => c,
                        (None, Some(c)) => c,
                        (Some(a), Some(b)) => a.blend(&b),
                    };
                    let idx = ((y * 256 + x) * 4) as usize;
                    out[idx] = blended_colour.r;
                    out[idx+1] = blended_colour.g;
                    out[idx+2] = blended_colour.b;
                }
            }
        }
    }
}

pub struct Texture {
    pub x: usize,
    pub y: usize,
    pub colours: Vec<Colour>
}

impl Texture {
    pub fn checkerboard() -> Self {
        Self {
            x: 32, 
            y: 32,
            colours: (0..1024).map(|pos| {
                let x_quad = (pos % 32) / 4;
                let y_quad = (pos / 32) / 4;
                if (x_quad + y_quad) % 2 == 0 {
                    Colour::black()
                } else {
                    Colour::white()
                }
            }).collect::<Vec<_>>()
        }
    }
}
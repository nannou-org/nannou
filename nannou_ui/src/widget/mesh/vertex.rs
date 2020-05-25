//! The vertex type used by the **Mesh** widget.

use nannou::color;
use nannou::geom::{self, Vector3};
use nannou::mesh::vertex::{WithColor, WithTexCoords};

pub type Scalar = f32;
pub type Point = geom::Point3<Scalar>;
pub type Point2 = geom::Point2<Scalar>;
pub type Color = color::LinSrgba;
pub type TexCoords = Point2;
pub type Normal = Vector3<Scalar>;

/// The vertex type produced by the **widget::Mesh**'s inner **MeshType**.
pub type Vertex =
    WithTexCoords<WithColor<Point, Color>, TexCoords>;

/// The number of channels in the color type.
pub const COLOR_CHANNEL_COUNT: usize = 4;

pub const DEFAULT_COLOR: Color = color::Alpha {
    color: color::rgb::Rgb {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
        standard: std::marker::PhantomData,
    },
    alpha: 1.0,
};

/// Default texture coordinates, for the case where a vertex is not textured.
pub const DEFAULT_TEX_COORDS: TexCoords = TexCoords { x: 0.0, y: 0.0 };

/// Simplified constructor for a **widget::mesh::Vertex**.
pub fn new(point: Point, color: Color, tex_coords: TexCoords) -> Vertex {
    WithTexCoords {
        tex_coords,
        vertex: WithColor {
            color,
            vertex: point,
        },
    }
}

pub fn colored(point: Point, color: Color) -> Vertex {
    new(point, color, DEFAULT_TEX_COORDS)
}

pub fn textured(point: Point, tex_coords: TexCoords) -> Vertex {
    new(point, DEFAULT_COLOR, tex_coords)
}

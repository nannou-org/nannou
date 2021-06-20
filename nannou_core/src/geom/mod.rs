//! Types, functions and other items related to geometry. This module is the source of all graphics
//! and lazer primitives and aids work in 2D and 3D space.
//!
//! Each module provides a set of general tools for working with the named geometry including:
//!
//! - A typed, object representation.
//! - Functions for producing vertices, triangles and triangulation indices.
//! - Functions for checking whether or not the geometry contains a point.
//! - Functions for determining the bounding rectangle or cuboid.
//! - A function for finding the centroid.

pub mod cuboid;
pub mod ellipse;
pub mod point;
pub mod polygon;
pub mod quad;
pub mod range;
pub mod rect;
pub mod scalar;
pub mod tri;
pub mod vector;
pub mod vertex;

pub use self::cuboid::Cuboid;
pub use self::ellipse::Ellipse;
pub use self::point::{pt2, pt3, pt4, Point2, Point3, Point4};
pub use self::polygon::Polygon;
pub use self::quad::Quad;
pub use self::range::{Align, Edge, Range};
pub use self::rect::{Corner, Padding, Rect};
pub use self::scalar::Scalar;
pub use self::tri::Tri;
#[allow(deprecated)]
pub use self::vector::{Vector2, Vector3, Vector4};
pub use self::vertex::{Vertex, Vertex2d, Vertex3d};
pub use glam::{
    dvec2, dvec3, dvec4, ivec2, ivec3, ivec4, vec2, vec3, vec4, DVec2, DVec3, DVec4, IVec2, IVec3,
    IVec4, Vec2, Vec3, Vec4,
};

// General geometry utility functions

/// The `Rect` that bounds the given sequence of vertices.
///
/// Returns `None` if the given iterator is empty.
pub fn bounding_rect<I>(vertices: I) -> Option<Rect<<I::Item as Vertex>::Scalar>>
where
    I: IntoIterator,
    I::Item: Vertex2d,
{
    let mut vertices = vertices.into_iter();
    vertices.next().map(|first| {
        let [x, y] = first.point2();
        let bounds = Rect {
            x: Range::new(x, x),
            y: Range::new(y, y),
        };
        vertices.fold(bounds, |b, v| b.stretch_to_point(v.point2()))
    })
}

/// The `Cuboid` that bounds the given sequence of vertices.
///
/// Returns `None` if the given iterator is empty.
pub fn bounding_cuboid<I>(vertices: I) -> Option<Cuboid<<I::Item as Vertex>::Scalar>>
where
    I: IntoIterator,
    I::Item: Vertex3d,
{
    let mut vertices = vertices.into_iter();
    vertices.next().map(|first| {
        let [x, y, z] = first.point3();
        let bounds = Cuboid {
            x: Range::new(x, x),
            y: Range::new(y, y),
            z: Range::new(z, z),
        };
        vertices.fold(bounds, |b, v| b.stretch_to_point(v.point3()))
    })
}

/// The `centroid` (average position) of all vertices in the given iterator.
///
/// Returns `None` if the given iterator contains no vertices.
pub fn centroid<I>(vertices: I) -> Option<I::Item>
where
    I: IntoIterator,
    I::Item: vertex::Average,
{
    <I::Item as vertex::Average>::average(vertices)
}

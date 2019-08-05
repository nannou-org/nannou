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

use crate::math::num_traits::cast;
use crate::math::{BaseFloat, EuclideanSpace};
use std::ops;

pub mod cuboid;
pub mod ellipse;
pub mod graph;
pub mod line;
pub mod path;
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
pub use self::graph::Graph;
pub use self::line::Line;
pub use self::path::{path, Path};
pub use self::point::{pt2, pt3, pt4, Point2, Point3, Point4};
pub use self::polygon::Polygon;
pub use self::quad::Quad;
pub use self::range::{Align, Edge, Range};
pub use self::rect::{Corner, Padding, Rect};
pub use self::tri::Tri;
pub use self::vector::{vec2, vec3, vec4, Vector2, Vector3, Vector4};
pub use self::vertex::{Vertex, Vertex2d, Vertex3d};

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
        let Point2 { x, y } = first.point2();
        let bounds = Rect {
            x: Range::new(x, x),
            y: Range::new(y, y),
        };
        vertices.fold(bounds, |b, v| {
            let Point2 { x, y } = v.point2();
            let point = Point2 { x, y };
            b.stretch_to_point(point)
        })
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
        let Point3 { x, y, z } = first.point3();
        let bounds = Cuboid {
            x: Range::new(x, x),
            y: Range::new(y, y),
            z: Range::new(z, z),
        };
        vertices.fold(bounds, |b, v| {
            let Point3 { x, y, z } = v.point3();
            let point = Point3 { x, y, z };
            b.stretch_to_point(point)
        })
    })
}

/// The `centroid` (average position) of all vertices in the given iterator.
///
/// Returns `None` if the given iterator contains no vertices.
pub fn centroid<I, S>(vertices: I) -> Option<I::Item>
where
    I: IntoIterator,
    I::Item: Vertex<Scalar = S> + EuclideanSpace<Scalar = S>,
    <I::Item as EuclideanSpace>::Diff: ops::Div<S, Output = <I::Item as EuclideanSpace>::Diff>,
    S: BaseFloat,
{
    let mut vertices = vertices.into_iter();
    vertices.next().map(|first| {
        let init = (1, first.to_vec());
        let (len, total) = vertices.fold(init, |(i, acc), p| (i + 1, acc + p.to_vec()));
        EuclideanSpace::from_vec(total / cast(len).unwrap())
    })
}

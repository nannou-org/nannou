use crate::geom::{scalar::Scalar, Point3};
use crate::math::num_traits::{cast, NumCast};
use core::ops::{Add, Div};

/// Types used as vertices that can be used to describe geometric points in space.
pub trait Vertex: Clone + Copy + PartialEq {
    /// The values used to describe the vertex position.
    type Scalar: Scalar;
}

/// Vertex types that have at least 2 dimensions.
pub trait Vertex2d: Vertex {
    /// The x, y location of the vertex.
    fn point2(self) -> [Self::Scalar; 2];
}

/// Vertex types that have at least 3 dimensions.
pub trait Vertex3d: Vertex2d {
    /// The x, y, z location of the vertex.
    fn point3(self) -> [Self::Scalar; 3];
}

/// Vertices whose average can be determined.
///
/// Useful for determining the centroid of triangles, quads and arbitrary polygons.
pub trait Average: Vertex {
    /// Produce the average of the given sequence of vertices.
    ///
    /// Returns `None` if the given iterator is empty.
    fn average<I>(vertices: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self>;
}

/// If a type is not specified for a piece of geometry, this is the default type used.
pub type Default = Point3;

/// An iterator yielding a vertex for each index yielded by the given indices iterator.
#[derive(Clone, Debug)]
pub struct IterFromIndices<'a, I, V: 'a = Default> {
    indices: I,
    vertices: &'a [V],
}

/// Produce an iterator yielding a vertex for each index yielded by the given indices iterator.
pub fn iter_from_indices<I, V>(indices: I, vertices: &[V]) -> IterFromIndices<I::IntoIter, V>
where
    I: IntoIterator<Item = usize>,
{
    let indices = indices.into_iter();
    IterFromIndices { indices, vertices }
}

// Iterators

impl<'a, I, V> Iterator for IterFromIndices<'a, I, V>
where
    I: Iterator<Item = usize>,
{
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        let IterFromIndices {
            ref mut indices,
            ref vertices,
        } = *self;
        indices.next().map(|i| &vertices[i])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

impl<'a, I, V> DoubleEndedIterator for IterFromIndices<'a, I, V>
where
    I: Iterator<Item = usize> + DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let IterFromIndices {
            ref mut indices,
            ref vertices,
        } = *self;
        indices.next_back().map(|i| &vertices[i])
    }
}

impl<'a, I, V> ExactSizeIterator for IterFromIndices<'a, I, V>
where
    I: Iterator<Item = usize> + ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.indices.len()
    }
}

// // Srgba impls.
//
// impl<V> Srgba<V> {
//     /// A reference to the inner vertex.
//     pub fn vertex(&self) -> &V {
//         &self.0
//     }
//     /// A reference to the inner rgba.
//     pub fn rgba(&self) -> &color::Srgba {
//         &self.1
//     }
// }
//
// impl<V> Deref for Srgba<V> {
//     type Target = V;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// impl<V> DerefMut for Srgba<V> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
//
// impl<V> From<(V, color::Srgba)> for Srgba<V> {
//     fn from((v, rgba): (V, color::Srgba)) -> Self {
//         Srgba(v, rgba)
//     }
// }
//
// impl<V> Into<(V, color::Srgba)> for Srgba<V> {
//     fn into(self) -> (V, color::Srgba) {
//         let Srgba(v, rgba) = self;
//         (v, rgba)
//     }
// }

// Vertex impls

impl Vertex for glam::Vec2 {
    type Scalar = f32;
}

impl Vertex for glam::Vec3 {
    type Scalar = f32;
}

impl Vertex for glam::DVec2 {
    type Scalar = f64;
}

impl Vertex for glam::DVec3 {
    type Scalar = f64;
}

impl Vertex for glam::IVec2 {
    type Scalar = i32;
}

impl Vertex for glam::IVec3 {
    type Scalar = i32;
}

impl<S> Vertex for [S; 2]
where
    S: Scalar,
{
    type Scalar = S;
}

impl<S> Vertex for [S; 3]
where
    S: Scalar,
{
    type Scalar = S;
}

impl<S> Vertex for (S, S)
where
    S: Scalar,
{
    type Scalar = S;
}

impl<S> Vertex for (S, S, S)
where
    S: Scalar,
{
    type Scalar = S;
}

// Vertex2d impls

impl Vertex2d for glam::Vec2 {
    fn point2(self) -> [Self::Scalar; 2] {
        self.to_array()
    }
}

impl Vertex2d for glam::Vec3 {
    fn point2(self) -> [Self::Scalar; 2] {
        self.truncate().to_array()
    }
}

impl Vertex2d for glam::DVec2 {
    fn point2(self) -> [Self::Scalar; 2] {
        self.to_array()
    }
}

impl Vertex2d for glam::DVec3 {
    fn point2(self) -> [Self::Scalar; 2] {
        self.truncate().to_array()
    }
}

impl Vertex2d for glam::IVec2 {
    fn point2(self) -> [Self::Scalar; 2] {
        self.to_array()
    }
}

impl Vertex2d for glam::IVec3 {
    fn point2(self) -> [Self::Scalar; 2] {
        self.truncate().to_array()
    }
}

impl<S> Vertex2d for [S; 2]
where
    S: Scalar,
{
    fn point2(self) -> [Self::Scalar; 2] {
        self
    }
}

impl<S> Vertex2d for [S; 3]
where
    S: Scalar,
{
    fn point2(self) -> [Self::Scalar; 2] {
        let [x, y, _] = self;
        [x, y]
    }
}

impl<S> Vertex2d for (S, S)
where
    S: Scalar,
{
    fn point2(self) -> [Self::Scalar; 2] {
        let (x, y) = self;
        [x, y]
    }
}

impl<S> Vertex2d for (S, S, S)
where
    S: Scalar,
{
    fn point2(self) -> [Self::Scalar; 2] {
        let (x, y, _) = self;
        [x, y]
    }
}

// Vertex3d impls

impl Vertex3d for glam::Vec3 {
    fn point3(self) -> [Self::Scalar; 3] {
        self.to_array()
    }
}

impl Vertex3d for glam::DVec3 {
    fn point3(self) -> [Self::Scalar; 3] {
        self.to_array()
    }
}

impl Vertex3d for glam::IVec3 {
    fn point3(self) -> [Self::Scalar; 3] {
        self.to_array()
    }
}

impl<S> Vertex3d for [S; 3]
where
    S: Scalar,
{
    fn point3(self) -> [Self::Scalar; 3] {
        self
    }
}

impl<S> Vertex3d for (S, S, S)
where
    S: Scalar,
{
    fn point3(self) -> [Self::Scalar; 3] {
        let (x, y, z) = self;
        [x, y, z]
    }
}

// Average impls

fn avg_glam_vecs<I>(vertices: I) -> Option<I::Item>
where
    I: IntoIterator,
    I::Item: Add<I::Item, Output = I::Item>
        + Div<<I::Item as Vertex>::Scalar, Output = I::Item>
        + Vertex,
    <I::Item as Vertex>::Scalar: NumCast,
{
    let mut vertices = vertices.into_iter();
    vertices.next().map(|first| {
        let init = (1, first);
        let (len, total) = vertices.fold(init, |(i, acc), p| (i + 1, acc + p));
        let divisor: <I::Item as Vertex>::Scalar = cast(len).unwrap();
        total / divisor
    })
}

impl Average for glam::Vec2 {
    fn average<I>(vertices: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        avg_glam_vecs(vertices)
    }
}

impl Average for glam::Vec3 {
    fn average<I>(vertices: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        avg_glam_vecs(vertices)
    }
}

impl Average for glam::DVec2 {
    fn average<I>(vertices: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        avg_glam_vecs(vertices)
    }
}

impl Average for glam::DVec3 {
    fn average<I>(vertices: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        avg_glam_vecs(vertices)
    }
}

impl<S> Average for [S; 2]
where
    S: Scalar + NumCast,
{
    fn average<I>(vertices: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let mut vertices = vertices.into_iter();
        vertices.next().map(|first| {
            let init = (1, first);
            let (len, [x, y]) =
                vertices.fold(init, |(i, [ax, ay]), [bx, by]| (i + 1, [ax + bx, ay + by]));
            let divisor: S = cast(len).unwrap();
            [x / divisor, y / divisor]
        })
    }
}

impl<S> Average for [S; 3]
where
    S: Scalar + NumCast,
{
    fn average<I>(vertices: I) -> Option<Self>
    where
        I: IntoIterator<Item = Self>,
    {
        let mut vertices = vertices.into_iter();
        vertices.next().map(|first| {
            let init = (1, first);
            let (len, [x, y, z]) = vertices.fold(init, |(i, [ax, ay, az]), [bx, by, bz]| {
                (i + 1, [ax + bx, ay + by, az + bz])
            });
            let divisor: S = cast(len).unwrap();
            [x / divisor, y / divisor, z / divisor]
        })
    }
}

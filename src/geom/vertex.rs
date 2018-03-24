use color;
use math::{BaseNum, Point2, Point3};
use std::ops::{Deref, DerefMut};

/// Types used as vertices that can be used to describe geometric points in space.
pub trait Vertex: Clone + Copy + PartialEq {
    /// The values used to describe the vertex position.
    type Scalar: BaseNum;
}

/// Vertex types that have at least 2 dimensions.
pub trait Vertex2d: Vertex {
    /// The x, y location of the vertex.
    fn point2(self) -> Point2<Self::Scalar>;
}

/// Vertex types that have at least 3 dimensions.
pub trait Vertex3d: Vertex2d {
    /// The x, y, z location of the vertex.
    fn point3(self) -> Point3<Self::Scalar>;
}

/// If a type is not specified for a scalar along an axis, this is the default type used.
pub type DefaultScalar = f32;
/// If a type is not specified for a piece of geometry, this is the default type used.
pub type Default = Point3<DefaultScalar>;

/// An iterator yielding a vertex for each index yielded by the given indices iterator.
#[derive(Clone, Debug)]
pub struct IterFromIndices<'a, I, V: 'a = Default> {
    indices: I,
    vertices: &'a [V],
}

/// A vertex that is colored with the given linear `RGBA` color.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rgba<V = Default>(pub V, pub color::Rgba);

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
        let IterFromIndices { ref mut indices, ref vertices } = *self;
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
        let IterFromIndices { ref mut indices, ref vertices } = *self;
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

// Rgba impls.

impl<V> Rgba<V> {
    /// A reference to the inner vertex.
    pub fn vertex(&self) -> &V {
        &self.0
    }
    /// A reference to the inner rgba.
    pub fn rgba(&self) -> &color::Rgba {
        &self.1
    }
}

impl<V> Deref for Rgba<V> {
    type Target = V;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> DerefMut for Rgba<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<V> From<(V, color::Rgba)> for Rgba<V> {
    fn from((v, rgba): (V, color::Rgba)) -> Self {
        Rgba(v, rgba)
    }
}

impl<V> Into<(V, color::Rgba)> for Rgba<V> {
    fn into(self) -> (V, color::Rgba) {
        let Rgba(v, rgba) = self;
        (v, rgba)
    }
}

// Vertex impls

impl<S> Vertex for Point2<S>
where
    S: BaseNum,
{
    type Scalar = S;
}

impl<S> Vertex for Point3<S>
where
    S: BaseNum,
{
    type Scalar = S;
}

impl<S> Vertex for [S; 2]
where
    S: BaseNum,
{
    type Scalar = S;
}

impl<S> Vertex for [S; 3]
where
    S: BaseNum,
{
    type Scalar = S;
}

impl<S> Vertex for (S, S)
where
    S: BaseNum,
{
    type Scalar = S;
}

impl<S> Vertex for (S, S, S)
where
    S: BaseNum,
{
    type Scalar = S;
}

// impl<S> Vertex for Vector2<S>
// where
//     S: BaseNum,
// {
//     type Scalar = S;
// }
// 
// impl<S> Vertex for Vector3<S>
// where
//     S: BaseNum,
// {
//     type Scalar = S;
// }

impl<V> Vertex for Rgba<V>
where
    V: Vertex,
{
    type Scalar = V::Scalar;
}

// Vertex2d impls

impl<S> Vertex2d for Point2<S>
where
    S: BaseNum,
{
    fn point2(self) -> Point2<S> {
        Point2 { x: self.x, y: self.y }
    }
}

impl<S> Vertex2d for Point3<S>
where
    S: BaseNum,
{
    fn point2(self) -> Point2<S> {
        Point2 { x: self.x, y: self.y }
    }
}

impl<S> Vertex2d for [S; 2]
where
    S: BaseNum,
{
    fn point2(self) -> Point2<S> {
        Point2 { x: self[0], y: self[1] }
    }
}

impl<S> Vertex2d for [S; 3]
where
    S: BaseNum,
{
    fn point2(self) -> Point2<S> {
        Point2 { x: self[0], y: self[1] }
    }
}

impl<S> Vertex2d for (S, S)
where
    S: BaseNum,
{
    fn point2(self) -> Point2<S> {
        let (x, y) = self;
        Point2 { x, y }
    }
}

impl<S> Vertex2d for (S, S, S)
where
    S: BaseNum,
{
    fn point2(self) -> Point2<S> {
        let (x, y, _) = self;
        Point2 { x, y }
    }
}

// impl<S> Vertex2d for Vector2<S>
// where
//     S: BaseNum,
// {
//     fn point2(self) -> Point2<S> {
//         self
//     }
// }
// 
// impl<S> Vertex2d for Vector3<S>
// where
//     S: BaseNum,
// {
//     fn point2(self) -> Point2<S> {
//         Point2 { x: self.x, y: self.y }
//     }
// }

impl<V> Vertex2d for Rgba<V>
where
    V: Vertex2d,
{
    fn point2(self) -> Point2<V::Scalar> {
        self.0.point2()
    }
}

// Vertex3d impls

impl<S> Vertex3d for Point3<S>
where
    S: BaseNum,
{
    fn point3(self) -> Point3<S> {
        Point3 { x: self.x, y: self.y, z: self.z }
    }
}

impl<S> Vertex3d for [S; 3]
where
    S: BaseNum,
{
    fn point3(self) -> Point3<S> {
        Point3 { x: self[0], y: self[1], z: self[2] }
    }
}

impl<S> Vertex3d for (S, S, S)
where
    S: BaseNum,
{
    fn point3(self) -> Point3<S> {
        let (x, y, z) = self;
        Point3 { x, y, z }
    }
}

// impl<S> Vertex3d for Vector3<S>
// where
//     S: BaseNum,
// {
//     fn point3(self) -> Point3<S> {
//         self
//     }
// }

impl<V> Vertex3d for Rgba<V>
where
    V: Vertex3d,
{
    fn point3(self) -> Point3<V::Scalar> {
        self.0.point3()
    }
}

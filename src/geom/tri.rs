use color::Rgba;
use math::{BaseNum, Point2, Point3, Zero};
use std::ops::Deref;

/// A triangle as three vertices.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tri<V = Point3<f64>>(pub [V; 3]);

/// A vertex that is colored with the given linear `RGBA` color.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RgbaVertex<V = Point3<f64>>(pub V, pub Rgba);

/// Types used as vertices that can be used to describe a triangle.
pub trait Vertex: Clone + Copy + PartialEq {
    /// The values used to describe the vertex position.
    type Scalar: BaseNum;
}

/// Two dimensional vertices.
pub trait Vertex2d: Vertex {
    /// The x, y location of the vertex.
    fn point2(self) -> Point2<Self::Scalar>;
}

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

impl<V> Vertex for RgbaVertex<V>
where
    V: Vertex,
{
    type Scalar = V::Scalar;
}

impl<S> Vertex2d for Point2<S>
where
    S: BaseNum + Zero,
{
    fn point2(self) -> Point2<S> {
        self
    }
}

impl<V> Vertex2d for RgbaVertex<V>
where
    V: Vertex2d,
{
    fn point2(self) -> Point2<V::Scalar> {
        self.0.point2()
    }
}

impl<V> Tri<V> {
    /// Returns `true` if the given 2D vertex is contained within the 2D `Tri`.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate nannou;
    /// # use nannou::prelude::*;
    /// # use nannou::geom::Tri;
    /// # fn main() {
    /// let a = Point2 { x: -0.5, y: 0.0 };
    /// let b = Point2 { x: 0.0, y: 1.0 };
    /// let c = Point2 { x: 0.5, y: -0.75 };
    /// let tri = Tri([a, b, c]);
    /// assert!(tri.contains(&Point2 { x: 0.0, y: 0.0 }));
    /// assert!(!tri.contains(&Point2 { x: 3.0, y: 3.0 }));
    /// # }
    /// ```
    pub fn contains(&self, v: &V) -> bool
    where
        V: Vertex2d,
    {
        let (a, b, c) = (*self).into();
        let (a, b, c) = (a.point2(), b.point2(), c.point2());
        let v = (*v).point2();

        fn sign<S>(a: Point2<S>, b: Point2<S>, c: Point2<S>) -> S
        where
            S: BaseNum,
        {
            (a[0] - c[0]) * (b[1] - c[1]) - (b[0] - c[0]) * (a[1] - c[1])
        }

        let b1 = sign(v, a, b) < V::Scalar::zero();
        let b2 = sign(v, b, c) < V::Scalar::zero();
        let b3 = sign(v, c, a) < V::Scalar::zero();

        (b1 == b2) && (b2 == b3)
    }
}

/// Returns the first `Tri` that contains the given vertex.
///
/// Returns `None` if no `Tri`'s contain the given vertex.
pub fn iter_contains<I, V>(tris: I, v: V) -> Option<I::Item>
where
    I: IntoIterator,
    I::Item: AsRef<Tri<V>>,
    V: Vertex2d,
{
    tris.into_iter().find(|tri| tri.as_ref().contains(&v))
}

impl<V> Deref for Tri<V>
where
    V: Vertex,
{
    type Target = [V; 3];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> From<[V; 3]> for Tri<V>
where
    V: Vertex,
{
    fn from(points: [V; 3]) -> Self {
        Tri(points)
    }
}

impl<V> From<(V, V, V)> for Tri<V>
where
    V: Vertex,
{
    fn from((a, b, c): (V, V, V)) -> Self {
        Tri([a, b, c])
    }
}

impl<V> Into<[V; 3]> for Tri<V>
where
    V: Vertex,
{
    fn into(self) -> [V; 3] {
        self.0
    }
}

impl<V> Into<(V, V, V)> for Tri<V>
where
    V: Vertex,
{
    fn into(self) -> (V, V, V) {
        (self[0], self[1], self[2])
    }
}

impl<V> AsRef<Tri<V>> for Tri<V>
where
    V: Vertex,
{
    fn as_ref(&self) -> &Tri<V> {
        self
    }
}

impl<V> AsRef<[V; 3]> for Tri<V>
where
    V: Vertex,
{
    fn as_ref(&self) -> &[V; 3] {
        &self.0
    }
}

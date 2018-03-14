use geom::{vertex, Cuboid, Range, Rect, Vertex, Vertex2d, Vertex3d};
use math::{BaseNum, EuclideanSpace, Point2, Zero};
use std::ops::Deref;

/// The number of vertices in a triangle.
pub const NUM_VERTICES: u8 = 3;

/// A triangle as three vertices.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Tri<V = vertex::Default>(pub [V; NUM_VERTICES as usize]);

/// An iterator yielding each of the vertices of the triangle.
#[derive(Clone, Debug)]
pub struct Vertices<V = vertex::Default> {
    tri: Tri<V>,
    index: u8,
}

impl<V> Tri<V>
where
    V: Vertex,
{
    /// Produce an iterator yielding each of the vertices of the triangle.
    pub fn vertices(self) -> Vertices<V> {
        let tri = self;
        let index = 0;
        Vertices { tri, index }
    }

    /// Produce the centroid of the triangle aka the "mean"/"average" of all the points.
    pub fn centroid(self) -> V
    where
        V: EuclideanSpace,
    {
        EuclideanSpace::centroid(&self[..])
    }

    /// Maps the underlying vertices to a new type and returns the resulting `Tri`.
    pub fn map<F, V2>(self, mut map: F) -> Tri<V2>
    where
        F: FnMut(V) -> V2,
    {
        let (a, b, c) = self.into();
        Tri([map(a), map(b), map(c)])
    }
}

impl<V> Tri<V>
where
    V: Vertex2d,
{
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

    /// The bounding `Rect` of the triangle.
    pub fn bounding_rect(self) -> Rect<V::Scalar>
    where
        V: Vertex2d,
    {
        let (a, b, c) = self.into();
        let (a, b, c) = (a.point2(), b.point2(), c.point2());
        let rect = Rect { x: Range::new(a.x, a.x), y: Range::new(a.y, a.y) };
        rect.stretch_to_point(b).stretch_to_point(c)
    }

    /// The bounding `Rect` of the triangle.
    pub fn bounding_cuboid(self) -> Cuboid<V::Scalar>
    where
        V: Vertex3d,
    {
        let (a, b, c) = self.into();
        let (a, b, c) = (a.point3(), b.point3(), c.point3());
        let cuboid = Cuboid {
            x: Range::new(a.x, a.x),
            y: Range::new(a.y, a.y),
            z: Range::new(a.z, a.z),
        };
        cuboid.stretch_to_point(b).stretch_to_point(c)
    }
}

impl<V> Iterator for Vertices<V>
where
    V: Clone,
{
    type Item = V;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < NUM_VERTICES {
            let v = self.tri.0[self.index as usize].clone();
            self.index += 1;
            Some(v)
        } else {
            None
        }
    }
}

/// Returns the first `Tri` that contains the given vertex.
///
/// Returns `None` if no `Tri`'s contain the given vertex.
pub fn iter_contains<I, V>(tris: I, v: &V) -> Option<I::Item>
where
    I: IntoIterator,
    I::Item: AsRef<Tri<V>>,
    V: Vertex2d,
{
    tris.into_iter().find(|tri| tri.as_ref().contains(v))
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

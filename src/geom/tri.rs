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

/// An iterator yielding triangles whose vertices are produced by the given iterator yielding
/// vertices.
#[derive(Clone, Debug)]
pub struct IterFromVertices<I> {
    vertices: I,
}

/// An iterator that flattens an iterator yielding triangles into its vertices.
#[derive(Clone, Debug)]
pub struct VerticesFromIter<I, V = vertex::Default> {
    tris: I,
    tri: Option<Vertices<V>>,
}

/// Converts an iterator yielding `[usize; 3]` into an iterator yielding `usize`s.
#[derive(Clone, Debug)]
pub struct FlattenIndices<I> {
    indices: I,
    b: Option<usize>,
    c: Option<usize>,
}

impl<V> Tri<V>
where
    V: Vertex,
{
    /// Create a **Tri** by indexing into the given buffer.
    ///
    /// **Panics** if any of the given indices are out of range of the given `vertices` slice.
    pub fn from_index_tri(vertices: &[V], indices: &[usize; 3]) -> Self
    where
        V: Vertex,
    {
        from_index_tri(vertices, indices)
    }

    /// Create a **Tri** from the next three vertices yielded by the given `vertices` iterator.
    ///
    /// Returns **None** if there were not at least 3 vertices in the given iterator.
    pub fn from_vertices<I>(vertices: I) -> Option<Self>
    where
        I: IntoIterator<Item = V>,
    {
        from_vertices(vertices)
    }

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
    pub fn map_vertices<F, V2>(self, mut map: F) -> Tri<V2>
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
        let rect = Rect {
            x: Range::new(a.x, a.x),
            y: Range::new(a.y, a.y),
        };
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

/// Create a **Tri** from the next three vertices yielded by the given `vertices` iterator.
///
/// Returns **None** if there were not at least 3 vertices in the given iterator.
pub fn from_vertices<I>(vertices: I) -> Option<Tri<I::Item>>
where
    I: IntoIterator,
{
    let mut vertices = vertices.into_iter();
    match (vertices.next(), vertices.next(), vertices.next()) {
        (Some(a), Some(b), Some(c)) => Some(Tri([a, b, c])),
        _ => None,
    }
}

/// Produce an iterator yielding a triangle for every three vertices yielded by the given
/// `vertices` iterator.
pub fn iter_from_vertices<I>(vertices: I) -> IterFromVertices<I::IntoIter>
where
    I: IntoIterator,
{
    let vertices = vertices.into_iter();
    IterFromVertices { vertices }
}

/// Create a **Tri** by indexing into the given buffer.
///
/// **Panics** if any of the given indices are out of range of the given `vertices` slice.
pub fn from_index_tri<V>(vertices: &[V], indices: &[usize; 3]) -> Tri<V>
where
    V: Clone,
{
    let a = vertices[indices[0]].clone();
    let b = vertices[indices[1]].clone();
    let c = vertices[indices[2]].clone();
    Tri([a, b, c])
}

/// Produce an iterator that flattens the given iterator yielding triangles into its vertices.
pub fn vertices_from_iter<I, V>(tris: I) -> VerticesFromIter<I::IntoIter, V>
where
    I: IntoIterator<Item = Tri<V>>,
{
    let tris = tris.into_iter();
    let tri = None;
    VerticesFromIter { tris, tri }
}

/// Given an iterator yielding trios of indices, produce an iterator that yields each index one at
/// a time.
pub fn flatten_index_tris<I>(index_tris: I) -> FlattenIndices<I::IntoIter>
where
    I: IntoIterator<Item = [usize; 3]>,
{
    FlattenIndices {
        indices: index_tris.into_iter(),
        b: None,
        c: None,
    }
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

impl<V> ExactSizeIterator for Vertices<V>
where
    V: Clone,
{
    fn len(&self) -> usize {
        NUM_VERTICES as usize - self.index as usize
    }
}

impl<I> Iterator for IterFromVertices<I>
where
    I: Iterator,
{
    type Item = Tri<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        from_vertices(&mut self.vertices)
    }
}

impl<I, V> Iterator for VerticesFromIter<I, V>
where
    I: Iterator<Item = Tri<V>>,
    V: Vertex,
{
    type Item = V;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(v) = self.tri.as_mut().and_then(|vs| vs.next()) {
                return Some(v);
            }
            match self.tris.next() {
                Some(t) => self.tri = Some(t.vertices()),
                None => return None,
            }
        }
    }
}

impl<I, V> ExactSizeIterator for VerticesFromIter<I, V>
where
    I: Iterator<Item = Tri<V>> + ExactSizeIterator,
    V: Vertex,
{
    fn len(&self) -> usize {
        let current_tri_vs = self.tri.as_ref().map(|vs| vs.len()).unwrap_or(0);
        let remaining_tri_vs = self.tris.len() * NUM_VERTICES as usize;
        current_tri_vs + remaining_tri_vs
    }
}

impl<I> Iterator for FlattenIndices<I>
where
    I: Iterator<Item = [usize; 3]>,
{
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.b.take() {
            return Some(next);
        }
        if let Some(next) = self.c.take() {
            return Some(next);
        }
        if let Some([next, b, c]) = self.indices.next() {
            self.b = Some(b);
            self.c = Some(c);
            return Some(next);
        }
        None
    }
}

impl<I> ExactSizeIterator for FlattenIndices<I>
where
    I: Iterator<Item = [usize; 3]> + ExactSizeIterator,
{
    fn len(&self) -> usize {
        self.indices.len() * 3 + self.b.map(|_| 1).unwrap_or(0) + self.c.map(|_| 1).unwrap_or(0)
    }
}

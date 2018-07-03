use geom::{tri, vertex, Cuboid, Range, Rect, Tri, Vertex, Vertex2d, Vertex3d};
use math::EuclideanSpace;
use std::ops::{Deref, Index};

/// The number of vertices in a quad.
pub const NUM_VERTICES: u8 = 4;

/// The number of triangles that make up a quad.
pub const NUM_TRIANGLES: u8 = 2;

/// The same as `triangles`, but instead returns the vertex indices for each triangle.
pub const TRIANGLE_INDEX_TRIS: TrianglesIndexTris = [[0, 1, 2], [0, 2, 3]];
pub type TrianglesIndexTris = [[usize; tri::NUM_VERTICES as usize]; NUM_TRIANGLES as usize];

/// The number of indices used to describe each triangle in the quad.
pub const NUM_TRIANGLE_INDICES: u8 = 6;

/// The same as `triangles`, but instead returns the vertex indices for each triangle.
pub const TRIANGLE_INDICES: [usize; NUM_TRIANGLE_INDICES as usize] = [0, 1, 2, 0, 2, 3];

/// A quad represented by its four vertices.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Quad<V = vertex::Default>(pub [V; NUM_VERTICES as usize]);

/// An `Iterator` yielding the two triangles that make up a quad.
#[derive(Clone, Debug)]
pub struct Triangles<V = vertex::Default> {
    a: Option<Tri<V>>,
    b: Option<Tri<V>>,
}

/// A simple iterator yielding each vertex in a `Quad`.
#[derive(Clone, Debug)]
pub struct Vertices<V = vertex::Default> {
    quad: Quad<V>,
    index: u8,
}

impl<V> Quad<V>
where
    V: Vertex,
{
    /// Produce an iterator yielding each vertex in the `Quad`.
    pub fn vertices(self) -> Vertices<V> {
        vertices(self)
    }

    /// Produce the centroid of the quad, aka the "mean"/"average" vertex.
    pub fn centroid(&self) -> V
    where
        V: EuclideanSpace,
    {
        centroid(self)
    }

    /// Triangulates the given quad, represented by four points that describe its edges in either
    /// clockwise or anti-clockwise order.
    ///
    /// # Example
    ///
    /// The following rectangle
    ///
    /// ```ignore
    ///  a        b
    ///   --------
    ///   |      |
    ///   |      |
    ///   |      |
    ///   --------
    ///  d        c
    ///
    /// ```
    ///
    /// given as
    ///
    /// ```ignore
    /// triangles([a, b, c, d])
    /// ```
    ///
    /// returns
    ///
    /// ```ignore
    /// (Tri([a, b, c]), Tri([a, c, d]))
    /// ```
    ///
    /// Here's a basic code example:
    ///
    /// ```
    /// extern crate nannou;
    ///
    /// use nannou::geom::{self, pt2, Quad, Tri};
    ///
    /// fn main() {
    ///     let a = pt2(0.0, 1.0);
    ///     let b = pt2(1.0, 1.0);
    ///     let c = pt2(1.0, 0.0);
    ///     let d = pt2(0.0, 0.0);
    ///     let quad = Quad([a, b, c, d]);
    ///     let triangles = geom::quad::triangles(&quad);
    ///     assert_eq!(triangles, (Tri([a, b, c]), Tri([a, c, d])));
    /// }
    /// ```
    #[inline]
    pub fn triangles(&self) -> (Tri<V>, Tri<V>) {
        triangles(self)
    }

    /// The same as `triangles` but provided as an **Iterator**.
    pub fn triangles_iter(&self) -> Triangles<V> {
        triangles_iter(self)
    }

    /// The bounding `Rect` of the quad.
    pub fn bounding_rect(self) -> Rect<V::Scalar>
    where
        V: Vertex2d,
    {
        let (a, b, c, d) = self.into();
        let (a, b, c, d) = (a.point2(), b.point2(), c.point2(), d.point2());
        let rect = Rect {
            x: Range::new(a.x, a.x),
            y: Range::new(a.y, a.y),
        };
        rect.stretch_to_point(b)
            .stretch_to_point(c)
            .stretch_to_point(d)
    }

    /// The bounding `Rect` of the triangle.
    pub fn bounding_cuboid(self) -> Cuboid<V::Scalar>
    where
        V: Vertex3d,
    {
        let (a, b, c, d) = self.into();
        let (a, b, c, d) = (a.point3(), b.point3(), c.point3(), d.point3());
        let cuboid = Cuboid {
            x: Range::new(a.x, a.x),
            y: Range::new(a.y, a.y),
            z: Range::new(a.z, a.z),
        };
        cuboid
            .stretch_to_point(b)
            .stretch_to_point(c)
            .stretch_to_point(d)
    }

    /// Map the **Quad**'s vertices to a new type.
    pub fn map_vertices<F, V2>(self, mut map: F) -> Quad<V2>
    where
        F: FnMut(V) -> V2,
    {
        let (a, b, c, d) = self.into();
        Quad([map(a), map(b), map(c), map(d)])
    }
}

/// Produce an iterator yielding each vertex in the given **Quad**.
pub fn vertices<V>(quad: Quad<V>) -> Vertices<V> {
    let index = 0;
    Vertices { quad, index }
}

/// Produce the centroid of the quad, aka the "mean"/"average" vertex.
pub fn centroid<V>(quad: &Quad<V>) -> V
where
    V: EuclideanSpace,
{
    EuclideanSpace::centroid(&quad[..])
}

/// Triangulates the given quad, represented by four points that describe its edges in either
/// clockwise or anti-clockwise order.
///
/// # Example
///
/// The following rectangle
///
/// ```ignore
///
///  a        b
///   --------
///   |      |
///   |      |
///   |      |
///   --------
///  d        c
///
/// ```
///
/// given as
///
/// ```ignore
/// triangles([a, b, c, d])
/// ```
///
/// returns
///
/// ```ignore
/// (Tri([a, b, c]), Tri([a, c, d]))
/// ```
///
/// Here's a basic code example:
///
/// ```
/// extern crate nannou;
///
/// use nannou::geom::{self, pt2, Quad, Tri};
///
/// fn main() {
///     let a = pt2(0.0, 1.0);
///     let b = pt2(1.0, 1.0);
///     let c = pt2(1.0, 0.0);
///     let d = pt2(0.0, 0.0);
///     let quad = Quad([a, b, c, d]);
///     let triangles = geom::quad::triangles(&quad);
///     assert_eq!(triangles, (Tri([a, b, c]), Tri([a, c, d])));
/// }
/// ```
#[inline]
pub fn triangles<V>(q: &Quad<V>) -> (Tri<V>, Tri<V>)
where
    V: Vertex,
{
    let a = Tri::from_index_tri(&q.0, &TRIANGLE_INDEX_TRIS[0]);
    let b = Tri::from_index_tri(&q.0, &TRIANGLE_INDEX_TRIS[1]);
    (a, b)
}

/// The same as `triangles` but provided as an `Iterator`.
pub fn triangles_iter<V>(points: &Quad<V>) -> Triangles<V>
where
    V: Vertex,
{
    let (a, b) = triangles(points);
    Triangles {
        a: Some(a),
        b: Some(b),
    }
}

impl<V> Iterator for Triangles<V> {
    type Item = Tri<V>;
    fn next(&mut self) -> Option<Self::Item> {
        self.a.take().or_else(|| self.b.take())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<V> DoubleEndedIterator for Triangles<V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.b.take().or_else(|| self.a.take())
    }
}

impl<V> ExactSizeIterator for Triangles<V> {
    fn len(&self) -> usize {
        match (&self.a, &self.b) {
            (&Some(_), &Some(_)) => 2,
            (&None, &Some(_)) => 0,
            _ => 1,
        }
    }
}

impl<V> Iterator for Vertices<V>
where
    V: Clone,
{
    type Item = V;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < NUM_VERTICES {
            let v = self.quad[self.index as usize].clone();
            self.index += 1;
            Some(v)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
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

impl<V> Deref for Quad<V> {
    type Target = [V; NUM_VERTICES as usize];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<V> From<[V; NUM_VERTICES as usize]> for Quad<V>
where
    V: Vertex,
{
    fn from(points: [V; NUM_VERTICES as usize]) -> Self {
        Quad(points)
    }
}

impl<V> From<(V, V, V, V)> for Quad<V>
where
    V: Vertex,
{
    fn from((a, b, c, d): (V, V, V, V)) -> Self {
        Quad([a, b, c, d])
    }
}

impl<V> Into<[V; NUM_VERTICES as usize]> for Quad<V>
where
    V: Vertex,
{
    fn into(self) -> [V; NUM_VERTICES as usize] {
        self.0
    }
}

impl<V> Into<(V, V, V, V)> for Quad<V>
where
    V: Vertex,
{
    fn into(self) -> (V, V, V, V) {
        (self[0], self[1], self[2], self[3])
    }
}

impl<V> AsRef<Quad<V>> for Quad<V>
where
    V: Vertex,
{
    fn as_ref(&self) -> &Quad<V> {
        self
    }
}

impl<V> AsRef<[V; NUM_VERTICES as usize]> for Quad<V>
where
    V: Vertex,
{
    fn as_ref(&self) -> &[V; NUM_VERTICES as usize] {
        &self.0
    }
}

impl<V> Index<usize> for Quad<V>
where
    V: Vertex,
{
    type Output = V;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

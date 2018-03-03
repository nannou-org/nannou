use geom::Tri;

/// A quad represented by its corners.
pub type Quad<V> = [V; 4];

/// An `Iterator` yielding the two triangles that make up a quad.
#[derive(Clone)]
pub struct Triangles<V> {
    a: Option<Tri<V>>,
    b: Option<Tri<V>>,
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
/// use nannou::geom::{self, Tri};
/// use nannou::math::vec2;
///
/// fn main() {
///     let a = vec2(0.0, 1.0);
///     let b = vec2(1.0, 1.0);
///     let c = vec2(1.0, 0.0);
///     let d = vec2(0.0, 0.0);
///     let quad = [a, b, c, d];
///     let triangles = geom::quad::triangles(&quad);
///     assert_eq!(triangles, (Tri([a, b, c]), Tri([a, c, d])));
/// }
/// ```
#[inline]
pub fn triangles<V>(points: &[V; 4]) -> (Tri<V>, Tri<V>)
where
    V: Copy,
{
    let (a, b, c, d) = (points[0], points[1], points[2], points[3]);
    (Tri([a, b, c]), Tri([a, c, d]))
}

/// The same as `triangles` but provided as an `Iterator`.
pub fn triangles_iter<V>(points: &[V; 4]) -> Triangles<V>
where
    V: Copy,
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

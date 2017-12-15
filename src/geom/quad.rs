use geom::Tri;
use math::Point2;

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
///
/// fn main() {
///     let a = [0.0, 1.0].into();
///     let b = [1.0, 1.0].into();
///     let c = [1.0, 0.0].into();
///     let d = [0.0, 0.0].into();
///     let quad = [a, b, c, d];
///     let triangles = geom::quad::triangles(&quad);
///     assert_eq!(triangles, (Tri([a, b, c]), Tri([a, c, d])));
/// }
/// ```
#[inline]
pub fn triangles<S>(points: &[Point2<S>; 4]) -> (Tri<Point2<S>>, Tri<Point2<S>>)
where
    S: Copy,
{
    let (a, b, c, d) = (points[0], points[1], points[2], points[3]);
    (Tri([a, b, c]), Tri([a, c, d]))
}

use crate::geom::{line, Line, Point2, Quad};
use crate::math::BaseFloat;

/// An iterator yielding the normal vertices of a miter polyline.
///
/// This iterator can be used in combination with the `Indices` iterator to triangulate a miter
/// polyline.
///
/// For example, given the following line abcd:
///
/// ```ignore
/// a-----------------------b
///                        /
///                       /
///                      /
///                     /
///                    /
///                   /
///                  c----------------------------d
/// ```
///
/// With some `half_thickness` of the line, the following points are produced:
///
/// ```ignore
/// 0----------------------------2  ^
///                             /   | half_thickness
/// a-----------------------b  /    v
///                        /  /
/// 1------------------3  /  /
///                   /  /  /
///                  /  /  /
///                 /  /  4-----------------------6
///                /  /  
///               /  c----------------------------d
///              /
///             5---------------------------------7
/// ```
///
/// ## Example
///
/// ```
/// 

///
/// use nannou::prelude::*;
/// use nannou::geom::line;
///
/// fn main() {
///     let half_thickness = 1.0;
///     let points = vec![
///         pt2(0.0, 0.0),
///         pt2(2.0, 0.0),
///         pt2(2.0, 2.0),
///     ];
///     let expected = vec![
///         [pt2(0.0, 1.0), pt2(0.0, -1.0)],
///         [pt2(1.0, 1.0), pt2(3.0, -1.0)],
///         [pt2(1.0, 2.0), pt2(3.0, 2.0)],
///     ];
///     let result: Vec<_> = line::join::miter::vertices(points, half_thickness).collect();
///     assert_eq!(result, expected);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct VertexPairs<I, S> {
    half_thickness: S,
    points: I,
    // The point yielded prior to `point_b`.
    point_a: Option<Point2<S>>,
    // The point for which each iteration yields normal vertices.
    point_b: Option<Point2<S>>,
}

/// The same as the **VertexPairs** iterator, but flattened in order to produce a single vertex at
/// a time.
#[derive(Clone, Debug)]
pub struct Vertices<I, S> {
    pairs: VertexPairs<I, S>,
    next: Option<Point2<S>>,
}

/// An iterator yielding the indices necessary to triangulate a miter polyline from a sequence of
/// normal vertices.
#[derive(Clone, Debug)]
pub struct TriangleIndices {
    i: usize,
    n_tris: usize,
    // Every second triangle requires a slightly different vertex order.
    second: bool,
}

impl<I, S> VertexPairs<I, S> {
    /// Produces an iterator yeilding the miter normal vertices for each point in the given
    /// iterator.
    pub fn new<P>(points: P, half_thickness: S) -> Self
    where
        I: Iterator<Item = Point2<S>>,
        P: IntoIterator<IntoIter = I, Item = Point2<S>>,
        S: BaseFloat,
    {
        let mut points = points.into_iter();
        let point_a = None;
        let point_b = points.next();
        VertexPairs {
            half_thickness,
            points,
            point_a,
            point_b,
        }
    }

    /// Flatten this iterator yielding vertex pairs into an iterator that yields a single vertex at
    /// a time.
    pub fn flatten(self) -> Vertices<I, S> {
        Vertices {
            pairs: self,
            next: None,
        }
    }
}

impl TriangleIndices {
    /// Produce an iterator yielding the indices necessary to triangulate a miter polyline from a
    /// sequence of normal vertices.
    pub fn new(n_points: usize) -> Self {
        let n_normals = n_points * 2;
        let n_tris = n_normals - 2;
        TriangleIndices {
            i: 0,
            n_tris,
            second: false,
        }
    }
}

impl<I, S> Iterator for VertexPairs<I, S>
where
    I: Iterator<Item = Point2<S>>,
    S: BaseFloat,
{
    type Item = [Point2<S>; 2];
    fn next(&mut self) -> Option<Self::Item> {
        let VertexPairs {
            half_thickness,
            ref mut points,
            ref mut point_a,
            ref mut point_b,
        } = *self;
        next_pair(half_thickness, point_a, point_b, points.next())
    }
}

/// Determine the next vertex pair in the miter sequence given the two previous points and the next
/// if one exists.
///
/// - `half_thickness`: half the thickness of the line.
/// - `point_a`: the point in the sequence prior to the last point.
/// - `point_b`: the last point in the sequence.
/// - `next_point`: the next point in the sequence.
///
/// This function expects that `point_a` is `None` and `point_b` is `Some` (containing the first
/// point in the sequence) when beginning the sequence.
///
/// This function is used within both the `VertexPairs` iterator and the `polyline().vertices()`
/// draw builder method.
pub fn next_pair<S>(
    half_thickness: S,
    point_a: &mut Option<Point2<S>>,
    point_b: &mut Option<Point2<S>>,
    next_point: Option<Point2<S>>,
) -> Option<[Point2<S>; 2]>
where
    S: BaseFloat,
{
    let a = point_a.take();
    let b = point_b.take();
    match (a, b) {
        // Only occurs if there were no points in the sequence or if the last point has already
        // been yeilded.
        (None, None) | (Some(_), None) => None,

        // Should only occur on the first iteration.
        (None, Some(b)) => {
            let c = match next_point {
                Some(c) => c,
                // Only occurs if there was only point, in which case we cannot give valid
                // vertices.
                None => return None,
            };
            // Get the line quad between the two points.
            let line = Line {
                start: b,
                end: c,
                half_thickness,
            };
            let Quad([r, l, _, _]) = line.quad_corners();
            *point_a = Some(b);
            *point_b = Some(c);
            Some([l, r])
        }

        // Every other point.
        (Some(a), Some(b)) => {
            let c = match next_point {
                Some(c) => c,
                // If this is the last point.
                None => {
                    // Get the line quad between the two points.
                    let line = Line {
                        start: a,
                        end: b,
                        half_thickness,
                    };
                    let Quad([_, _, l, r]) = line.quad_corners();
                    *point_a = Some(b);
                    return Some([l, r]);
                }
            };
            let ab = Line {
                start: a,
                end: b,
                half_thickness,
            };
            let bc = Line {
                start: b,
                end: c,
                half_thickness,
            };
            let Quad([ar, al, bl_ab, br_ab]) = ab.quad_corners();
            let Quad([br_bc, bl_bc, cl, cr]) = bc.quad_corners();
            let il = match line::join::intersect((al, bl_ab), (cl, bl_bc)) {
                Some(il) => il,
                // If the lines are parallel, produce the join vertices.
                None => bl_ab,
            };
            let ir = match line::join::intersect((ar, br_ab), (cr, br_bc)) {
                Some(ir) => ir,
                None => br_ab,
            };
            *point_a = Some(b);
            *point_b = Some(c);
            Some([il, ir])
        }
    }
}

impl<I, S> ExactSizeIterator for VertexPairs<I, S>
where
    I: Iterator<Item = Point2<S>> + ExactSizeIterator,
    S: BaseFloat,
{
    fn len(&self) -> usize {
        let remaining_points = self.points.len();
        let a = self.point_a.is_some();
        let b = self.point_b.is_some();
        match (a, b) {
            (false, true) => {
                if remaining_points <= 1 {
                    0
                } else {
                    remaining_points
                }
            }
            (true, true) => remaining_points + 1,
            _ => 0,
        }
    }
}

impl<I, S> Iterator for Vertices<I, S>
where
    I: Iterator<Item = Point2<S>>,
    S: BaseFloat,
{
    type Item = Point2<S>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next.take() {
            return Some(next);
        }
        if let Some([this, next]) = self.pairs.next() {
            self.next = Some(next);
            return Some(this);
        }
        None
    }
}

impl<I, S> ExactSizeIterator for Vertices<I, S>
where
    I: Iterator<Item = Point2<S>> + ExactSizeIterator,
    S: BaseFloat,
{
    fn len(&self) -> usize {
        self.pairs.len() * 2 + if self.next.is_some() { 1 } else { 0 }
    }
}

impl Iterator for TriangleIndices {
    type Item = [usize; 3];
    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.n_tris {
            return None;
        }
        let trio = if self.second {
            [self.i, self.i + 2, self.i + 1]
        } else {
            [self.i, self.i + 1, self.i + 2]
        };
        self.second = !self.second;
        self.i += 1;
        Some(trio)
    }
}

impl ExactSizeIterator for TriangleIndices {
    fn len(&self) -> usize {
        self.n_tris - self.i
    }
}

/// Produces an iterator yeilding the miter normal vertices for each point in the given iterator.
pub fn vertices<I, S>(points: I, half_thickness: S) -> VertexPairs<I::IntoIter, S>
where
    I: IntoIterator<Item = Point2<S>>,
    S: BaseFloat,
{
    VertexPairs::new(points, half_thickness)
}

/// Produce an iterator yielding the indices necessary to triangulate a miter polyline from a
/// sequence of normal vertices.
pub fn triangle_indices(n_points: usize) -> TriangleIndices {
    TriangleIndices::new(n_points)
}

use geom::{quad, tri, vertex, DefaultScalar};
use math::{two, vec2, BaseFloat, EuclideanSpace, InnerSpace, Point2};

/// The quad used to describe a line.
pub type Quad<S = DefaultScalar> = quad::Quad<Point2<S>>;
/// The triangle types used to describe a line quad.
pub type Tri<S = DefaultScalar> = tri::Tri<Point2<S>>;
/// The vertices used to describe the quad of a line.
pub type Vertices<S = DefaultScalar> = quad::Vertices<Point2<S>>;
/// The triangles used to describe the quad of a line.
pub type Triangles<S = DefaultScalar> = quad::Triangles<Point2<S>>;

/// A line represented by two points.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Line<S = vertex::DefaultScalar> {
    /// The start point of the line.
    pub start: Point2<S>,
    /// The end point of the line.
    pub end: Point2<S>,
    /// Half of the thickness of the line.
    pub half_thickness: S,
}

impl<S> Line<S>
where
    S: BaseFloat,
{
    /// Short-hand constructor for a `Line`.
    pub fn new(start: Point2<S>, end: Point2<S>, half_thickness: S) -> Self {
        Line { start, end, half_thickness }
    }

    /// The centre of the line.
    pub fn centroid(&self) -> Point2<S> {
        EuclideanSpace::midpoint(self.start, self.end)
    }
}

impl<S> Line<S>
where
    S: BaseFloat,
{
    /// The four corners of the rectangle describing the line.
    pub fn quad_corners(&self) -> Quad<S> {
        quad_corners(self.start, self.end, self.half_thickness)
    }

    /// Produce an iterator yielding the four corners of the rectangle describing the line.
    pub fn quad_corners_iter(&self) -> Vertices<S> {
        quad::vertices(self.quad_corners())
    }

    /// The two triangles that describe the line.
    pub fn triangles(&self) -> (Tri<S>, Tri<S>) {
        triangles(self.start, self.end, self.half_thickness)
    }

    /// Given two points and half the line thickness, return the two triangles that describe the line.
    pub fn triangles_iter(&self) -> Triangles<S> {
        triangles_iter(self.start, self.end, self.half_thickness)
    }

    /// Describes whether or not the given point touches the line.
    ///
    /// If so, the `Tri` containing the point will be returned.
    ///
    /// `None` is returned otherwise.
    pub fn contains(&self, point: &Point2<S>) -> Option<Tri<S>> {
        contains(self.start, self.end, self.half_thickness, point)
    }
}

/// Given two points and half the line thickness, return the four corners of the rectangle
/// describing the line.
///
/// Given a line *a -> b*, the indices are laid out as follows:
///
/// ```ignore
/// 0                                        2
///  ----------------------------------------
///  |a                                    b|
///  ----------------------------------------
/// 1                                        3
/// ```
pub fn quad_corners<S>(a: Point2<S>, b: Point2<S>, half_thickness: S) -> Quad<S>
where
    S: BaseFloat,
{
    let direction = b - a;
    let unit = direction.normalize();
    let neg_1 = S::from(-1).unwrap();
    let normal = vec2(unit.y * neg_1, unit.x);
    let n = normal.normalize_to(half_thickness);
    let neg_n = n * neg_1;
    let r1 = a + neg_n;
    let r2 = a + n;
    let r3 = b + neg_n;
    let r4 = b + n;
    Quad::from([r1, r2, r3, r4])
}

/// Given two points and half the line thickness, return the two triangles that describe the line.
pub fn triangles<S>(a: Point2<S>, b: Point2<S>, half_thickness: S) -> (Tri<S>, Tri<S>)
where
    S: BaseFloat,
{
    let q = quad_corners(a, b, half_thickness);
    quad::triangles(&q)
}

/// Given two points and half the line thickness, return the two triangles that describe the line.
pub fn triangles_iter<S>(a: Point2<S>, b: Point2<S>, half_thickness: S) -> Triangles<S>
where
    S: BaseFloat,
{
    let q = quad_corners(a, b, half_thickness);
    let tris = quad::triangles_iter(&q);
    tris
}

/// Describes whether or not the given point touches the line described by *a -> b* with the given
/// thickness.
///
/// If so, the `Tri` containing the point will be returned.
///
/// `None` is returned otherwise.
pub fn contains<S>(a: Point2<S>, b: Point2<S>, thickness: S, point: &Point2<S>) -> Option<Tri<S>>
where
    S: BaseFloat,
{
    let half_thickness = thickness / two::<S>();
    let tris = triangles_iter(a, b, half_thickness);
    tri::iter_contains(tris, point)
}

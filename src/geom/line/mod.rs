use geom::{self, ellipse, quad, scalar, tri, Rect};
use math::{two, vec2, BaseFloat, EuclideanSpace, InnerSpace, Point2};

pub mod cap;
pub mod join;

/// The quad used to describe a line.
pub type Quad<S = scalar::Default> = quad::Quad<Point2<S>>;
/// The triangle types used to describe a line quad.
pub type Tri<S = scalar::Default> = tri::Tri<Point2<S>>;
/// The vertices used to describe the quad of a line.
pub type Vertices<S = scalar::Default> = quad::Vertices<Point2<S>>;
/// The triangles used to describe the quad of a line.
pub type Triangles<S = scalar::Default> = quad::Triangles<Point2<S>>;

/// A line represented by two points.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Line<S = scalar::Default> {
    /// The start point of the line.
    pub start: Point2<S>,
    /// The end point of the line.
    pub end: Point2<S>,
    /// Half of the thickness of the line.
    pub half_thickness: S,
}

/// The kind of geometry with which a line may be capped.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cap {
    /// A square edge that does not protrude past the ends of the line..
    ///
    /// This is equivalent to having no cap at all.
    Butt,
    /// Rounded caps with a radius equal to half the line's thickness.
    Round {
        /// The number of sides used to represent the cap.
        resolution: usize,
    },
    /// A square cap that protudes from the end of the line a distance that is equal to half of the
    /// line's thickness.
    Square,
}

/// A line whose ends are capped with protruding geometry (either rounded or squared).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Capped<S = scalar::Default> {
    /// The line itself.
    pub line: Line<S>,
    /// The kind of geometry with which the line is capped.
    pub cap: Cap,
}

/// An iterator yielding the vertices that describe the outer edges of a **Capped** line.
#[derive(Clone, Debug)]
pub struct CappedVertices<S = scalar::Default> {
    start: Option<cap::Vertices<S>>,
    line: Option<Vertices<S>>,
    end: Option<cap::Vertices<S>>,
}

impl<S> Line<S>
where
    S: BaseFloat,
{
    /// Short-hand constructor for a `Line`.
    pub fn new(start: Point2<S>, end: Point2<S>, half_thickness: S) -> Self {
        Line {
            start,
            end,
            half_thickness,
        }
    }

    /// The centre of the line.
    pub fn centroid(&self) -> Point2<S> {
        EuclideanSpace::midpoint(self.start, self.end)
    }

    /// The bounding **Rect** of the **Line** including thickness and line caps.
    pub fn bounding_rect(&self) -> Rect<S> {
        self.quad_corners().bounding_rect()
    }

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

    /// The edge at the start of the line.
    pub fn start_edge(&self) -> (Point2<S>, Point2<S>) {
        let (a, b, _, _) = self.quad_corners().into();
        (a, b)
    }

    /// The edge at the end of the line.
    pub fn end_edge(&self) -> (Point2<S>, Point2<S>) {
        let (_, _, c, d) = self.quad_corners().into();
        (c, d)
    }

    /// Produce the ellipse section that represents the rounded cap for the start of the line.
    pub fn start_cap_round(&self, resolution: usize) -> ellipse::Section<S> {
        let (a, b) = self.start_edge();
        cap::round::ellipse_section(a, b, resolution)
    }

    /// Produce the ellipse section that represents the rounded cap for the end of the line.
    pub fn end_cap_round(&self, resolution: usize) -> ellipse::Section<S> {
        let (c, d) = self.end_edge();
        cap::round::ellipse_section(c, d, resolution)
    }

    /// Produce the ellipse section that represents the squared cap for the start of the line.
    pub fn start_cap_square(&self) -> Quad<S> {
        let (a, b) = self.start_edge();
        cap::square::quad(a, b, self.half_thickness)
    }

    /// Produce the ellipse section that represents the squared cap for the end of the line.
    pub fn end_cap_square(&self) -> Quad<S> {
        let (c, d) = self.end_edge();
        cap::square::quad(c, d, self.half_thickness)
    }

    /// Produce the ellipse sections that represent rounded caps for the line.
    pub fn caps_round(&self, resolution: usize) -> (ellipse::Section<S>, ellipse::Section<S>) {
        let (a, b, c, d) = self.quad_corners().into();
        let start = cap::round::ellipse_section(a, b, resolution);
        let end = cap::round::ellipse_section(c, d, resolution);
        (start, end)
    }

    /// Produce the quads that represent square caps for the line.
    pub fn caps_square(&self) -> (Quad<S>, Quad<S>) {
        let (a, b, c, d) = self.quad_corners().into();
        let start = cap::square::quad(a, b, self.half_thickness);
        let end = cap::square::quad(c, d, self.half_thickness);
        (start, end)
    }

    /// Produce a `Capped` line with no protrusion beyond the line ends.
    pub fn capped_butt(self) -> Capped<S> {
        let line = self;
        let cap = Cap::Butt;
        Capped { line, cap }
    }

    /// Cap the line with rounded ends.
    pub fn capped_round(self, resolution: usize) -> Capped<S> {
        let line = self;
        let cap = Cap::Round { resolution };
        Capped { line, cap }
    }

    /// Cap the line with squared ends.
    pub fn capped_square(self) -> Capped<S> {
        let line = self;
        let cap = Cap::Square;
        Capped { line, cap }
    }
}

impl<S> Capped<S>
where
    S: BaseFloat,
{
    /// The vertices encompassing the entire line and caps.
    pub fn vertices(&self) -> CappedVertices<S> {
        let (start, end) = match self.cap {
            Cap::Butt => (cap::Vertices::Butt, cap::Vertices::Butt),
            Cap::Round { resolution } => {
                let (start_cap, end_cap) = self.line.caps_round(resolution);
                let start = cap::Vertices::Round(start_cap.circumference());
                let end = cap::Vertices::Round(end_cap.circumference());
                (start, end)
            }
            Cap::Square => {
                let (start_cap, end_cap) = self.line.caps_square();
                let start = cap::Vertices::Square(start_cap.vertices());
                let end = cap::Vertices::Square(end_cap.vertices());
                (start, end)
            }
        };
        let line = self.line.quad_corners_iter();
        CappedVertices {
            start: Some(start),
            line: Some(line),
            end: Some(end),
        }
    }

    /// The polygon representing the capped line.
    ///
    /// This can be useful for taking advantage of polygon methods including `triangles`,
    /// `contains`, `bounding_rect`, etc.
    pub fn polygon(&self) -> geom::Polygon<CappedVertices<S>> {
        let vertices = self.vertices();
        geom::Polygon { points: vertices }
    }
}

/// Given two points and half the line thickness, return the four corners of the rectangle
/// describing the line.
///
/// Given a line *a -> b*, the indices are laid out as follows:
///
/// ```ignore
/// 1                                        2
///  ----------------------------------------
///  |a                                    b|
///  ----------------------------------------
/// 0                                        3
/// ```
///
/// ## Examples
///
/// ```
/// extern crate nannou;
///
/// use nannou::prelude::*;
/// use nannou::geom::line;
///
/// fn main() {
///     let half_thickness = 1.0;
///     let a = pt2(0.0, 0.0);
///     let b = pt2(2.0, 0.0);
///
///     // ab
///     let expected = [pt2(0.0, -1.0), pt2(0.0, 1.0), pt2(2.0, 1.0), pt2(2.0, -1.0)];
///     assert_eq!(expected, line::quad_corners(a, b, half_thickness).0);
///
///     // ba
///     let expected = [pt2(2.0, 1.0), pt2(2.0, -1.0), pt2(0.0, -1.0), pt2(0.0, 1.0)];
///     assert_eq!(expected, line::quad_corners(b, a, half_thickness).0);
/// }
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
    let r3 = b + n;
    let r4 = b + neg_n;
    let quad = Quad::from([r1, r2, r3, r4]);
    quad
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

impl Default for Cap {
    fn default() -> Self {
        Cap::Butt
    }
}

impl<S> Iterator for CappedVertices<S>
where
    S: BaseFloat,
{
    type Item = Point2<S>;
    fn next(&mut self) -> Option<Self::Item> {
        // First yield all start cap vertices.
        if let Some(start) = self.start.as_mut() {
            if let Some(next) = start.next() {
                return Some(next);
            }
        }

        // Then yield the line's quad vertices.
        if let Some(line) = self.line.as_mut() {
            if let Some(next) = line.next() {
                return Some(next);
            }
        }

        // Finally yield the end cap vertices.
        if let Some(end) = self.end.as_mut() {
            if let Some(next) = end.next() {
                return Some(next);
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<S> ExactSizeIterator for CappedVertices<S>
where
    S: BaseFloat,
{
    fn len(&self) -> usize {
        let start = self.start.as_ref().map(|s| s.len()).unwrap_or(0);
        let line = self.line.as_ref().map(|l| l.len()).unwrap_or(0);
        let end = self.end.as_ref().map(|e| e.len()).unwrap_or(0);
        start + line + end
    }
}

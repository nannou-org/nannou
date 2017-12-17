//! Tools for working with point paths using varying kinds of line joins and caps.
//!
//! The naming and behaviour for line joins and caps follows the SVG format conventions.
//!
//! ## Line Joins
//!
//! - **miter**: Extends the stroke to where the edges on each side bisect. This is the default.
//! - **round**: Rounds the outside edge with a circle the diameter of the thickness.
//! - **bevel**: Cuts the outside edge off where a circle the diameter of the thickness intersects
//! the thickness.
//!
//! ## Line Caps
//!
//! - **butt**: Ends the stroke flush with the start or end of the line. This is the default.
//! - **round**: Ends the line with a half circle whose radius is half the line thickness.
//! - **square**: Extends the line on either end by half the thickness.
//!
//! ```ignore
//! start                      end
//!   |                         |
//!   v                         v
//!
//!   ---------------------------
//!   |                         |   butt (default)
//!   ---------------------------
//!
//!  /---------------------------\
//! (                             ) round
//!  \---------------------------/
//!
//! -------------------------------
//! |                             | square
//! -------------------------------
//!
//!   ^                         ^
//!   |                         |
//! start                      end
//! ```
//!
//! # Examples
//!
//! ```ignore
//! point_path::miter(points, thickness).triangles();
//! point_path::round(points, thickness).square().contains(&point);
//! point_path::benel(points, thickness).round().triangles();
//!
//! // Calls `miter(points).butt().triangles()`
//! point_path::triangles(points, thickness); 
//! // Calls `miter(points).butt().contains(&point)`
//! point_path::contains(points, thickness, &point); 
//! ```

use geom::{quad, line, Tri};
use math::{self, BaseFloat, Point2};
pub use self::cap::Cap;

/// A point path described by a list of connected points joined by the given `join` style and
/// ending with the given `cap` style,
///
/// A **PointPath** can be triangulated using the `triangles()` method.
///
/// You can check if a `PointPath` contains a given point using the `contains(&point)` method.
pub struct PointPath<C, J, I, S> {
    pub cap: C,
    pub join: J,
    pub points: I,
    pub thickness: S,
}

#[derive(Clone)]
struct StartCap<C, T> {
    tris: C,
    first_line_tri: Option<T>,
}

/// Iterator yielding triangles that describe some `PointPath`.
#[derive(Clone)]
pub struct Triangles<C, J, I, S>
where
    cap::Tris<C, S>: Cap,
    I: Iterator,
{
    // All triangles from `start_cap` are returned first.
    start_cap: StartCap<<cap::Tris<C, S> as Cap>::Triangles, Tri<I::Item>>,
    // Triangles for each line are yielded in pairs.
    next: Option<Tri<I::Item>>,
    join: J,
    // Track the previous point in order to triangulate to the next.
    prev_point: I::Item,
    points: I,
    half_thickness: S,
    last_corners: (I::Item, I::Item),
    cap: C,
    end_cap: Option<<cap::Tris<C, S> as Cap>::Triangles>,
    end_cap_complete: bool,
}

pub mod join {
    pub struct Miter;
    pub struct Round;
    pub struct Bevel;
}

pub mod cap {
    use geom::{ellipse, quad, Rect, Tri};
    use math::{vec2, BaseFloat, InnerSpace, Point2};
    use math::num_traits::NumCast;
    use std::f64::consts::PI;
    use std::iter;

    /// Types that describe line caps.
    pub trait Cap {
        /// The scalar value used to describe points over the *x* and *y* axes.
        type Scalar;
        /// An iterator yielding triangles that describe the line cap.
        type Triangles: Iterator<Item=Tri<Point2<Self::Scalar>>> + Clone;
        /// Produce the `Triangles` given the start and end of the line cap and the line's thickness.
        fn triangles(self) -> Self::Triangles;
    }

    #[derive(Clone)]
    pub struct Tris<C, S> {
        cap: C,
        a: Point2<S>,
        b: Point2<S>,
        half_thickness: S,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Butt;
    #[derive(Clone, Copy, Debug)]
    pub struct Round;
    #[derive(Clone, Copy, Debug)]
    pub struct Square;

    impl<C, S> Tris<C, S> {
        pub fn new(cap: C, a: Point2<S>, b: Point2<S>, half_thickness: S) -> Self {
            Tris { cap, a, b, half_thickness }
        }
    }

    impl<S> Cap for Tris<Butt, S> {
        type Scalar = S;
        type Triangles = iter::Empty<Tri<Point2<S>>>;
        fn triangles(self) -> Self::Triangles {
            iter::empty()
        }
    }

    impl<S> Cap for Tris<Round, S>
    where
        S: BaseFloat + NumCast,
    {
        type Scalar = S;
        type Triangles = ellipse::Triangles<S>;
        fn triangles(self) -> Self::Triangles {
            let Tris { a, b, .. } = self;
            // TODO: Should make this configurable somehow, or at least adaptive to the thickness.
            let resolution = 50;
            let radians = S::from(PI).expect("could not cast from f64");
            let rect = Rect::from_corners(a, b);
            let av = vec2(a.x, a.y);
            let bv = vec2(b.x, b.y);
            let offset = av.angle(bv).0;
            ellipse::Circumference::new_section(rect, resolution, radians)
                .offset_radians(offset)
                .triangles()
        }
    }

    impl<S> Cap for Tris<Square, S>
    where
        S: BaseFloat,
    {
        type Scalar = S;
        type Triangles = quad::Triangles<S>;
        fn triangles(self) -> Self::Triangles {
            let Tris { a, b, half_thickness, .. } = self;
            let direction = b - a;
            let unit = direction.normalize();
            let normal = vec2(-unit.y, unit.x);
            let n = normal.normalize_to(half_thickness);
            let c = b + n;
            let d = a + n;
            let quad = [a, b, c, d];
            quad::triangles_iter(&quad)
        }
    }
}

impl<C, J, I, S> PointPath<C, J, I, S>
where
    C: Copy,
    cap::Tris<C, S>: Cap,
    I: Iterator<Item=Point2<S>>,
    S: BaseFloat,
{
    pub fn new(cap: C, join: J, points: I, thickness: S) -> Self {
        PointPath {
            cap,
            join,
            points,
            thickness,
        }
    }

    pub fn cap<T>(self, cap: T) -> PointPath<T, J, I, S> {
        let PointPath { join, points, thickness, .. } = self;
        PointPath { cap, join, points, thickness }
    }

    pub fn join<T>(self, join: T) -> PointPath<C, T, I, S> {
        let PointPath { cap, points, thickness, .. } = self;
        PointPath { cap, join, points, thickness }
    }

    pub fn cap_butt(self) -> PointPath<cap::Butt, J, I, S> {
        self.cap(cap::Butt)
    }

    pub fn cap_round(self) -> PointPath<cap::Round, J, I, S> {
        self.cap(cap::Round)
    }

    pub fn cap_square(self) -> PointPath<cap::Square, J, I, S> {
        self.cap(cap::Square)
    }

    pub fn join_miter(self) -> PointPath<C, join::Miter, I, S> {
        self.join(join::Miter)
    }

    pub fn join_round(self) -> PointPath<C, join::Round, I, S> {
        self.join(join::Round)
    }

    pub fn join_bevel(self) -> PointPath<C, join::Round, I, S> {
        self.join(join::Round)
    }

    /// Produce an iterator yielding all triangles that make up the point path.
    pub fn triangles(self) -> Triangles<C, J, I, S>
    where
        cap::Tris<C, S>: Cap,
    {
        let PointPath { cap, join, mut points, thickness } = self;
        // TODO: Perhaps should just use zeroed points in this case to avoid `panic`ing?
        const PANIC_MSG: &'static str = "there must be at least two points in a PointPath";
        let a = points.next().expect(PANIC_MSG);
        let b = points.next().expect(PANIC_MSG);
        let half_thickness = thickness / math::two::<S>();
        let corners = line::quad_corners(a, b, half_thickness);
        let (tri_1, tri_2) = quad::triangles(&corners);
        let c1 = corners[0];
        let c2 = corners[1];
        let start_cap_tris = cap::Tris::new(cap, c1, c2, half_thickness).triangles();
        let start_cap = StartCap { tris: start_cap_tris, first_line_tri: Some(tri_1) };
        let last_corners = (corners[2], corners[3]);
        Triangles {
            // All triangles from `start_cap` are returned first.
            start_cap,
            // Triangles for each line are yielded in pairs.
            next: Some(tri_2),
            join: join,
            // Track the previous point in order to triangulate to the next.
            prev_point: b,
            points,
            half_thickness,
            last_corners,
            cap,
            end_cap: None,
            end_cap_complete: false,
        }
    }
}

/// Construct a `PointPath` which can be either triangulated or checked for containing points.
///
/// By default this uses `Miter` line joins and `Butt` line ends.
pub fn new<I, S>(points: I, thickness: S) -> PointPath<cap::Butt, join::Miter, I::IntoIter, S>
where
    cap::Tris<cap::Butt, S>: Cap,
    I: IntoIterator<Item=Point2<S>>,
    S: BaseFloat,
{
    PointPath::new(cap::Butt, join::Miter, points.into_iter(), thickness)
}

// pub fn miter<I, S>(points: I, thickness: S) -> Joined<join::Miter, I, S> {
//     Joined::new(Miter, points, thickness)
// }
// 
// pub fn round<I, S>(points: I, thickness: S) -> Joined<join::Round, I, S> {
//     Joined::new(Round, points, thickness)
// }
// 
// pub fn bevel<I, S>(points: I, thickness: S) -> Joined<join::Bevel, I, S> {
//     Joined::new(Bevel, points, thickness)
// }

impl<C, J, I, S> Iterator for Triangles<C, J, I, S>
where
    C: Copy,
    cap::Tris<C, S>: Cap<Scalar=S>,
    I: Iterator<Item=Point2<S>>,
    S: BaseFloat,
{
    type Item = Tri<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        let Triangles {
            ref mut start_cap,
            ref mut next,
            ref join,
            ref mut prev_point,
            ref mut points,
            half_thickness,
            last_corners,
            cap,
            ref mut end_cap,
            ref mut end_cap_complete,
        } = *self;

        // First, check if there are start line cap points.
        if let Some(tri) = start_cap.tris.next().or_else(|| start_cap.first_line_tri.take()) {
            return Some(tri);
        } 

        // Next, check if there's a pending line triangle.
        if let Some(tri) = next.take() {
            return Some(tri);
        }

        // TODO: Check for `Join` triangles here?

        // If there is a remaining line point, create the next line.
        if let Some(b) = points.next() {
            let a = *prev_point;
            let tris = line::triangles(a, b, half_thickness);
            let (tri_a, tri_b) = (tris[0], tris[1]);
            *next = Some(tri_b);
            // TODO: Create join triangles here.
            return Some(tri_a);
        }

        // If the end cap has been fully returned, we're done!
        if *end_cap_complete {
            return None;
        }

        loop {
            match end_cap.take() {
                // If there is no end cap yet, create it.
                None => {
                    let (a, b) = last_corners;
                    *end_cap = Some(cap::Tris::new(cap, a, b, half_thickness).triangles());
                },
                // Return the next end cap triangle.
                Some(mut tris) => {
                    let tri = match tris.next() {
                        Some(tri) => tri,
                        None => {
                            *end_cap_complete = true;
                            return None;
                        }
                    };
                    *end_cap = Some(tris);
                    return Some(tri);
                }
            }
        }
    }
}

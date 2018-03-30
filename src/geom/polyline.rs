//! Tools for working with polylines using varying kinds of line joins and caps.
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
//! polyline::miter(points, thickness).triangles();
//! polyline::round(points, thickness).square().contains(&point);
//! polyline::benel(points, thickness).round().triangles();
//!
//! // Calls `miter(points).butt().triangles()`
//! polyline::triangles(points, thickness); 
//! // Calls `miter(points).butt().contains(&point)`
//! polyline::contains(points, thickness, &point); 
//! ```

use geom::{self, line, DefaultScalar, Rect, Tri, Vertex};
use math::{self, BaseFloat, Point2};
pub use self::cap::Cap;
pub use self::join::Join;

/// A polyline described by a list of connected points joined by the given `join` style and
/// ending with the given `cap` style,
///
/// A **Polyline** can be triangulated using the `triangles()` method.
///
/// You can check if a `Polyline` contains a given point using the `contains(&point)` method.
#[derive(Clone, Debug)]
pub struct Polyline<C, J, I, S = DefaultScalar> {
    pub cap: C,
    pub join: J,
    pub points: I,
    pub thickness: S,
}

/// A `Polyline` whose cap and join types may change at run-time.
pub type Dynamic<I, S = DefaultScalar> = Polyline<cap::Dynamic, join::Dynamic, I, S>;

#[derive(Clone)]
struct StartCap<C, T> {
    tris: C,
    first_line_tri: Option<T>,
}

/// Iterator yielding triangles that describe some `Polyline`.
#[derive(Clone)]
pub struct Parts<C, J, I, S = DefaultScalar>
where
    cap::Tris<C, S>: Cap,
    join::Tris<J, S>: Join,
    I: Iterator,
{
    cap: C,
    join: J,
    points: I,
    thickness: S,
    half_thickness: S,
    // All triangles from `start_cap` are returned first.
    start_cap: Option<<cap::Tris<C, S> as Cap>::Triangles>,
    // Triangles for each line are yielded in pairs.
    next_join: Option<<join::Tris<J, S> as Join>::Triangles>,
    // The point at which the line begins along with the two parallel corners respectively.
    next_line_start: (I::Item, I::Item, I::Item),
    // The end point of the next line.
    next_line_end: Option<(I::Item, I::Item, I::Item)>,
    // Whether or not the last end cap has been returned.
    end_cap_complete: bool,
}

/// A segment of a polyline represented by a sequence of triangles.
#[derive(Clone)]
pub enum Part<C, J, S = DefaultScalar> {
    /// The line caps at the start and end of the polyline.
    Cap {
        cap: C,
    },
    /// The straight line between either two caps, a cap and a join or two joins.
    Line {
        line: line::Triangles<S>,
    },
    /// A join between two lines.
    Join {
        join: J,
    },
}

/// Iterator yielding triangles that describe some `Polyline`.
#[derive(Clone)]
pub struct Triangles<C, J, I, S = DefaultScalar>
where
    cap::Tris<C, S>: Cap,
    join::Tris<J, S>: Join,
    I: Iterator,
    I::Item: Clone,
{
    parts: Parts<C, J, I, S>,
    current: Option<Part<<cap::Tris<C, S> as Cap>::Triangles, <join::Tris<J, S> as Join>::Triangles, S>>,
}

/// A `Triangles` iterator yielding triangles from a polyline whose cap and join types may change
/// at runtime.
#[derive(Clone)]
pub struct DynamicTriangles<I>
where
    I: Iterator,
    I::Item: Clone + Vertex,
    <I::Item as Vertex>::Scalar: BaseFloat,
{
    tris: Triangles<cap::Dynamic, join::Dynamic, I, <I::Item as Vertex>::Scalar>,
}

pub mod join {
    use geom::{ellipse, quad, DefaultScalar, Rect, Tri};
    use math::{vec2, BaseNum, BaseFloat, InnerSpace, Point2, Vector2};
    use math::num_traits::NumCast;
    use std;
    use std::f64::consts::PI;
    use std::iter;

    /// Types that can describe a join between two lines via a sequence of triangles.
    pub trait Join {
        /// The scalar value used to describe points over the *x* and *y* axes.
        type Scalar;
        /// An iterator yielding triangles that describe the line cap.
        type Triangles: Iterator<Item=Tri<Point2<Self::Scalar>>> + Clone;
        /// Produce the `Triangles` given the start and end of the line cap and the line's thickness.
        fn triangles(self) -> Self::Triangles;
    }

    /// The direction that the line is turning.
    #[derive(Copy, Clone, Debug)]
    pub enum Turn { Left, Right }

    #[derive(Clone)]
    pub struct Tris<J, S = DefaultScalar> {
        join: J,
        a: Point2<S>,
        b: Point2<S>,
        // The left side intersection between a->b and b->c.
        il: Point2<S>,
        // The right side intersection between a->b and b->c.
        ir: Point2<S>,
        turn: Turn,
        thickness: S,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Miter;
    #[derive(Clone, Copy, Debug)]
    pub struct Round;
    #[derive(Clone, Copy, Debug)]
    pub struct Bevel;

    // Join type that may change kind at runtime.
    #[derive(Clone, Copy, Debug)]
    pub enum Dynamic {
        Miter(Miter),
        Round(Round),
        Bevel(Bevel),
    }

    impl From<Miter> for Dynamic {
        fn from(miter: Miter) -> Self {
            Dynamic::Miter(miter)
        }
    }

    impl From<Round> for Dynamic {
        fn from(round: Round) -> Self {
            Dynamic::Round(round)
        }
    }

    impl From<Bevel> for Dynamic {
        fn from(bevel: Bevel) -> Self {
            Dynamic::Bevel(bevel)
        }
    }

    pub type MiterTris<S = DefaultScalar> = quad::Triangles<Point2<S>>;
    pub type RoundTris<S = DefaultScalar> = ellipse::Triangles<S>;
    pub type BevelTris<S = DefaultScalar> = iter::Once<Tri<Point2<S>>>;

    #[derive(Clone)]
    pub enum DynamicTris<S = DefaultScalar> {
        Miter(MiterTris<S>),
        Round(RoundTris<S>),
        Bevel(BevelTris<S>),
    }

    impl<S> Iterator for DynamicTris<S>
    where
        S: BaseFloat,
    {
        type Item = Tri<Point2<S>>;
        fn next(&mut self) -> Option<Self::Item> {
            match *self {
                DynamicTris::Miter(ref mut tris) => tris.next(),
                DynamicTris::Round(ref mut tris) => tris.next(),
                DynamicTris::Bevel(ref mut tris) => tris.next(),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match *self {
                DynamicTris::Miter(ref tris) => tris.size_hint(),
                DynamicTris::Round(ref tris) => tris.size_hint(),
                DynamicTris::Bevel(ref tris) => tris.size_hint(),
            }
        }
    }

    // TODO: Implement `DoubleEndedIterator` for `ellipse::Triangles` first.
    // impl<S> DoubleEndedIterator for DynamicTris<S>
    // where
    //     S: BaseFloat,
    // {
    //     fn next_back(&mut self) -> Option<Self::Item> {
    //         match *self {
    //             DynamicTris::Miter(ref mut tris) => tris.next_back(),
    //             DynamicTris::Round(ref mut tris) => tris.next_back(),
    //             DynamicTris::Bevel(ref mut tris) => tris.next_back(),
    //         }
    //     }
    // }

    impl<S> ExactSizeIterator for DynamicTris<S>
    where
        S: BaseFloat,
    {
        fn len(&self) -> usize {
            match *self {
                DynamicTris::Miter(ref tris) => tris.len(),
                DynamicTris::Round(ref tris) => tris.len(),
                DynamicTris::Bevel(ref tris) => tris.len(),
            }
        }
    }

    impl<J, S> Tris<J, S> {
        pub fn new(
            join: J,
            a: Point2<S>,
            b: Point2<S>,
            il: Point2<S>,
            ir: Point2<S>,
            turn: Turn,
            thickness: S,
        ) -> Self {
            Tris { join, a, b, il, ir, turn, thickness }
        }
    }

    impl<S> Join for Tris<Miter, S>
    where
        S: BaseNum,
    {
        type Scalar = S;
        type Triangles = MiterTris<S>;
        fn triangles(self) -> Self::Triangles {
            let Tris { a, b, il, ir, .. } = self;
            let quad = [a, il, b, ir].into();
            quad::triangles_iter(&quad)
        }
    }

    impl<S> Join for Tris<Round, S>
    where
        S: BaseFloat,
    {
        type Scalar = S;
        type Triangles = RoundTris<S>;
        fn triangles(self) -> Self::Triangles {
            const CIRCLE_RESOLUTION: f64 = 50.0;
            const TWO_PI: f64 = 2.0 * PI;
            let Tris { a, b, il, ir, turn, thickness, .. } = self;
            let wh = [thickness; 2].into();
            // Circle positioned at shortest intersection.
            let xy = match turn {
                Turn::Left => il,
                Turn::Right => ir,
            };
            fn vec<S>(p: Point2<S>) -> Vector2<S> { vec2(p.x, p.y) }
            let rect = Rect::from_xy_wh(xy, wh);
            let rad_a = vec(xy).angle(vec(a)).0;
            let rad_b = vec(xy).angle(vec(b)).0;
            let rad = rad_b - rad_a;
            let rad_f64: f64 = NumCast::from(rad).unwrap();
            let res: usize = NumCast::from(rad_f64 * CIRCLE_RESOLUTION / TWO_PI).unwrap();
            let res = std::cmp::max(res, 3);
            ellipse::Circumference::new_section(rect, res, rad).offset_radians(rad_a).triangles()
        }
    }

    impl<S> Join for Tris<Bevel, S>
    where
        S: Clone,
    {
        type Scalar = S;
        type Triangles = BevelTris<S>;
        fn triangles(self) -> Self::Triangles {
            let Tris { a, b, il, ir, turn, .. } = self;
            // Circle positioned at shortest intersection.
            let xy = match turn {
                Turn::Left => il,
                Turn::Right => ir,
            };
            let tri = Tri([xy, a, b].into());
            iter::once(tri)
        }
    }

    impl<S> Join for Tris<Dynamic, S>
    where
        S: BaseFloat,
    {
        type Scalar = S;
        type Triangles = DynamicTris<S>;
        fn triangles(self) -> Self::Triangles {
            let Tris { join, a, b, il, ir, turn, thickness } = self;
            macro_rules! join_tris {
                ($join:expr) => { Tris { join: $join, a, b, il, ir, turn, thickness } };
            }
            match join {
                Dynamic::Miter(join) => DynamicTris::Miter(join_tris!(join).triangles()),
                Dynamic::Round(join) => DynamicTris::Round(join_tris!(join).triangles()),
                Dynamic::Bevel(join) => DynamicTris::Bevel(join_tris!(join).triangles()),
            }
        }
    }
}

pub mod cap {
    use geom::{ellipse, quad, DefaultScalar, Rect, Tri};
    use math::{vec2, BaseFloat, InnerSpace, Point2};
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
    pub struct Tris<C, S = DefaultScalar> {
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

    // A type representing a line cap whose kind may change at runtime.
    #[derive(Clone, Copy, Debug)]
    pub enum Dynamic {
        Butt(Butt),
        Round(Round),
        Square(Square),
    }

    impl From<Butt> for Dynamic {
        fn from(butt: Butt) -> Self {
            Dynamic::Butt(butt)
        }
    }

    impl From<Round> for Dynamic {
        fn from(round: Round) -> Self {
            Dynamic::Round(round)
        }
    }

    impl From<Square> for Dynamic {
        fn from(square: Square) -> Self {
            Dynamic::Square(square)
        }
    }

    pub type ButtTris<S = DefaultScalar> = iter::Empty<Tri<Point2<S>>>;
    pub type RoundTris<S = DefaultScalar> = ellipse::Triangles<S>;
    pub type SquareTris<S = DefaultScalar> = quad::Triangles<Point2<S>>;

    #[derive(Clone)]
    pub enum DynamicTris<S = DefaultScalar> {
        Butt(ButtTris<S>),
        Round(RoundTris<S>),
        Square(SquareTris<S>),
    }

    impl<S> Iterator for DynamicTris<S>
    where
        S: BaseFloat,
    {
        type Item = Tri<Point2<S>>;
        fn next(&mut self) -> Option<Self::Item> {
            match *self {
                DynamicTris::Butt(ref mut tris) => tris.next(),
                DynamicTris::Round(ref mut tris) => tris.next(),
                DynamicTris::Square(ref mut tris) => tris.next(),
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            match *self {
                DynamicTris::Butt(ref tris) => tris.size_hint(),
                DynamicTris::Round(ref tris) => tris.size_hint(),
                DynamicTris::Square(ref tris) => tris.size_hint(),
            }
        }
    }

    // TODO: Implement `DoubleEndedIterator` for `ellipse::Triangles` first.
    // impl<S> DoubleEndedIterator for DynamicTris<S>
    // where
    //     S: BaseFloat,
    // {
    //     fn next_back(&mut self) -> Option<Self::Item> {
    //         match *self {
    //             DynamicTris::Miter(ref mut tris) => tris.next_back(),
    //             DynamicTris::Round(ref mut tris) => tris.next_back(),
    //             DynamicTris::Bevel(ref mut tris) => tris.next_back(),
    //         }
    //     }
    // }

    impl<S> ExactSizeIterator for DynamicTris<S>
    where
        S: BaseFloat,
    {
        fn len(&self) -> usize {
            match *self {
                DynamicTris::Butt(ref tris) => tris.len(),
                DynamicTris::Round(ref tris) => tris.len(),
                DynamicTris::Square(ref tris) => tris.len(),
            }
        }
    }

    impl<C, S> Tris<C, S> {
        pub fn new(cap: C, a: Point2<S>, b: Point2<S>, half_thickness: S) -> Self {
            Tris { cap, a, b, half_thickness }
        }
    }

    impl<S> Cap for Tris<Butt, S> {
        type Scalar = S;
        type Triangles = ButtTris<S>;
        fn triangles(self) -> Self::Triangles {
            iter::empty()
        }
    }

    impl<S> Cap for Tris<Round, S>
    where
        S: BaseFloat,
    {
        type Scalar = S;
        type Triangles = RoundTris<S>;
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
        type Triangles = SquareTris<S>;
        fn triangles(self) -> Self::Triangles {
            let Tris { a, b, half_thickness, .. } = self;
            let direction = b - a;
            let unit = direction.normalize();
            let normal = vec2(-unit.y, unit.x);
            let n = normal.normalize_to(half_thickness);
            let c = b + n;
            let d = a + n;
            let quad = [a, b, c, d].into();
            quad::triangles_iter(&quad)
        }
    }

    impl<S> Cap for Tris<Dynamic, S>
    where
        S: BaseFloat,
    {
        type Scalar = S;
        type Triangles = DynamicTris<S>;
        fn triangles(self) -> Self::Triangles {
            let Tris { cap, a, b, half_thickness } = self;
            macro_rules! cap_tris {
                ($cap:expr) => { Tris { cap: $cap, a, b, half_thickness } };
            }
            match cap {
                Dynamic::Butt(cap) => DynamicTris::Butt(cap_tris!(cap).triangles()),
                Dynamic::Round(cap) => DynamicTris::Round(cap_tris!(cap).triangles()),
                Dynamic::Square(cap) => DynamicTris::Square(cap_tris!(cap).triangles()),
            }
        }
    }
}

impl<C, J, I, S> Polyline<C, J, I, S>
where
    C: Copy,
    cap::Tris<C, S>: Cap,
    I: Iterator<Item=Point2<S>>,
    S: BaseFloat,
{
    pub fn new(cap: C, join: J, points: I, thickness: S) -> Self {
        Polyline {
            cap,
            join,
            points,
            thickness,
        }
    }

    pub fn cap<T>(self, cap: T) -> Polyline<T, J, I, S> {
        let Polyline { join, points, thickness, .. } = self;
        Polyline { cap, join, points, thickness }
    }

    pub fn join<T>(self, join: T) -> Polyline<C, T, I, S> {
        let Polyline { cap, points, thickness, .. } = self;
        Polyline { cap, join, points, thickness }
    }

    pub fn cap_butt(self) -> Polyline<cap::Butt, J, I, S> {
        self.cap(cap::Butt)
    }

    pub fn cap_round(self) -> Polyline<cap::Round, J, I, S> {
        self.cap(cap::Round)
    }

    pub fn cap_square(self) -> Polyline<cap::Square, J, I, S> {
        self.cap(cap::Square)
    }

    pub fn join_miter(self) -> Polyline<C, join::Miter, I, S> {
        self.join(join::Miter)
    }

    pub fn join_round(self) -> Polyline<C, join::Round, I, S> {
        self.join(join::Round)
    }

    pub fn join_bevel(self) -> Polyline<C, join::Round, I, S> {
        self.join(join::Round)
    }

    /// Produce an iterator yielding all `Part`s that make up the polyline.
    pub fn parts(self) -> Parts<C, J, I, S>
    where
        cap::Tris<C, S>: Cap,
        join::Tris<J, S>: Join,
    {
        let Polyline { cap, join, mut points, thickness } = self;
        // TODO: Perhaps should just use zeroed points in this case to avoid `panic`ing?
        const PANIC_MSG: &'static str = "there must be at least two points in a Polyline";
        let a = points.next().expect(PANIC_MSG);
        let b = points.next().expect(PANIC_MSG);
        let half_thickness = thickness / math::two::<S>();
        let corners = line::quad_corners(a, b, half_thickness);
        let (al, ar, bl, br) = (corners[0], corners[1], corners[2], corners[3]);
        let start_cap = cap::Tris::new(cap, al, ar, half_thickness).triangles();
        Parts {
            cap,
            join,
            points,
            thickness,
            half_thickness,
            start_cap: Some(start_cap),
            next_join: None,
            next_line_start: (a, al, ar),
            next_line_end: Some((b, bl, br)),
            end_cap_complete: false,
        }
    }
}

impl<C, J, I, S> Polyline<C, J, I, S>
where
    C: Copy,
    J: Copy,
    join::Tris<J, S>: Join<Scalar=S>,
    cap::Tris<C, S>: Cap<Scalar = S>,
    I: Iterator<Item=Point2<S>>,
    S: BaseFloat,
    Point2<S>: Vertex<Scalar = S>,
{
    /// Produce an iterator yielding all triangles that make up the polyline.
    pub fn triangles(self) -> Triangles<C, J, I, S>
    where
        J: Copy,
        join::Tris<J, S>: Join<Scalar=S>,
        cap::Tris<C, S>: Cap<Scalar=S>,
        I: Iterator<Item=Point2<S>>,
    {
        self.parts().triangles()
    }

    /// The bounding `Rect` of the polyline.
    ///
    /// This method triangulates the polyline (with joins, caps, etc) to ensure we get the *actual*
    /// bounds (not just the bounds of the points that describe the line).
    ///
    /// Returns `None` if the polyline contains no points.
    pub fn bounding_rect(self) -> Option<Rect<S>> {
        let vertices = self.triangles().flat_map(|tri| tri.vertices());
        geom::bounding_rect(vertices)
    }
}


/// Construct a `Polyline` which can be either triangulated or checked for containing points.
///
/// By default this uses `Miter` line joins and `Butt` line ends.
pub fn new<I, S>(points: I, thickness: S) -> Polyline<cap::Butt, join::Miter, I::IntoIter, S>
where
    cap::Tris<cap::Butt, S>: Cap,
    I: IntoIterator<Item=Point2<S>>,
    S: BaseFloat,
{
    Polyline::new(cap::Butt, join::Miter, points.into_iter(), thickness)
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

impl<C, J, I, S> Iterator for Parts<C, J, I, S>
where
    J: Copy,
    C: Copy,
    join::Tris<J, S>: Join<Scalar=S>,
    cap::Tris<C, S>: Cap<Scalar=S>,
    I: Iterator<Item=Point2<S>>,
    S: BaseFloat,
{
    type Item = Part<
        <cap::Tris<C, S> as Cap>::Triangles,
        <join::Tris<J, S> as Join>::Triangles,
        S>;
    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Uncomment the below code and finish TODO sections.
        unimplemented!();
        // let Parts {
        //     ref cap,
        //     ref join,
        //     ref mut points,
        //     thickness,
        //     half_thickness,
        //     ref mut start_cap,
        //     ref mut next_join,
        //     ref mut next_line_start,
        //     ref mut next_line_end,
        //     ref mut end_cap_complete,
        // } = *self;

        // // First check for the beginning line cap.
        // if let Some(cap) = start_cap.take() {
        //     return Some(Part::Cap { cap });
        // }

        // // Next, check for a pending line join.
        // if let Some(join) = next_join.take() {
        //     return Some(Part::Join { join });
        // }

        // // Otherwise, check for the next line.
        // if let Some((b, mut bl, mut br)) = next_line_end.take() {
        //     let (a, al, ar) = *next_line_start;
        //     let ab_direction = b - a;
        //     let ab_unit = ab_direction.normalize();
        //     let ab_normal = vec2(-ab_unit.y, ab_unit.x);
        //     let n = ab_normal.normalize_to(half_thickness);

        //     // If there's another point remaining, adjust bl and br for the join.
        //     //
        //     // TODO:
        //     //
        //     // - Find where bl, br intersect with the next line edges.
        //     // - Adjust `next_line_start` for angle towards `c`.
        //     if let Some(c) = points.next() {

        //         // TODO: Store this in the iterator for re-use.
        //         let bc_direction = c - b;
        //         let bc_unit = bc_direction.normalize();
        //         let bc_normal = vec2(-bc_unit.y, bc_unit.x);
        //         let n = bc_normal.normalize_to(half_thickness);
        //         let mut cl = [c.x - n.x, c.y - n.y].into();
        //         let mut cr = [c.x + n.x, c.y + n.y].into();

        //         // TODO: Find the left and right intersections.
        //         let il = unimplemented!();
        //         let ir = unimplemented!();

        //         // TODO: Create join triangles here.
        //         let turn: join::Turn = unimplemented!();
        //         let (a, b) = match turn {
        //             join::Turn::Left => (ar, br),
        //             join::Turn::Right => (al, bl),
        //         };
        //         let tris = join::Tris::new(*join, a, b, il, ir, turn, thickness).triangles();

        //         // TODO: Shorten `bl` and `br` to shortest intersection point.

        //         *next_join = Some(tris);
        //         *next_line_end = Some((c, cl, cr));
        //     }
        //     *next_line_start = (b, bl, br);

        //     let quad = [al, ar, br, bl].into();
        //     let tris = quad::triangles_iter(&quad);
        //     return Some(Part::Line { line: tris });
        // }

        // // If the end_cap has been returned, we're done.
        // if *end_cap_complete {
        //     return None;
        // }

        // // Otherwise, return the end cap.
        // *end_cap_complete = true;
        // let (_, l, r) = *next_line_start;
        // let cap = cap::Tris::new(*cap, l, r, half_thickness).triangles();
        // return Some(Part::Cap { cap });
    }
}

impl<C, J, S> Iterator for Part<C, J, S>
where
    C: Iterator<Item=Tri<Point2<S>>>,
    J: Iterator<Item=Tri<Point2<S>>>,
{
    type Item = Tri<Point2<S>>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Part::Line { ref mut line } => line.next(),
            Part::Join { ref mut join } => join.next(),
            Part::Cap { ref mut cap } => cap.next(),
        }
    }
}

impl<C, J, I, S> Parts<C, J, I, S>
where
    J: Copy,
    C: Copy,
    join::Tris<J, S>: Join<Scalar=S>,
    cap::Tris<C, S>: Cap<Scalar=S>,
    I: Iterator<Item=Point2<S>>,
    S: BaseFloat,
{
    /// Converts the `Parts` iterator into an iterator yielding all `Tri`s yielded by each `Part`.
    pub fn triangles(mut self) -> Triangles<C, J, I, S> {
        let current = self.next();
        let parts = self;
        Triangles { current, parts }
    }
}

impl<C, J, I, S> Iterator for Triangles<C, J, I, S>
where
    J: Copy,
    C: Copy,
    join::Tris<J, S>: Join<Scalar=S>,
    cap::Tris<C, S>: Cap<Scalar=S>,
    I: Iterator<Item=Point2<S>>,
    S: BaseFloat,
{
    type Item = Tri<Point2<S>>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(tri) = self.current.as_mut().and_then(Iterator::next) {
                return Some(tri);
            }
            match self.parts.next() {
                Some(part) => self.current = Some(part),
                None => return None,
            }
        }
    }
}

// impl<I> Iterator for DynamicTriangles<I>
//     I: Iterator,
//     I::Item: Clone + Vertex2d,
//     <I::Item as Vertex>::Scalar: BaseFloat,
// {
//     type Item = Tri<I::Item>;
//     fn next(&mut self) -> Option<Self::Item> {
//         self.tris.next()
//     }
// }

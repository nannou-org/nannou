//! Items related to capping the ends of **Line**s or **Polyline**s.

use geom::{ellipse, quad, DefaultScalar, Rect, Tri};
use math::{vec2, BaseFloat, InnerSpace, Point2};
use std::f64::consts::PI;
use std::iter;

pub mod butt;
pub mod round;
pub mod square;

/// Types that describe line caps.
pub trait Cap {
    /// The scalar value used to describe points over the *x* and *y* axes.
    type Scalar;
    /// An iterator yielding triangles that describe the line cap.
    type Triangles: Iterator<Item=Tri<Point2<Self::Scalar>>> + Clone;
    /// Produce the `Triangles` given the start and end of the line cap and the line's thickness.
    fn triangles(self) -> Self::Triangles;
}

// A type representing a line cap whose kind may change at runtime.
#[derive(Clone, Copy, Debug)]
pub enum Dynamic {
    Butt(Butt),
    Round(Round),
    Square(Square),
}

/// An iterator yielding the vertices of a line cap
#[derive(Clone, Debug)]
pub enum Vertices<S> {
    Butt,
    Round(ellipse::Circumference<S>),
    Square(quad::Vertices<Point2<S>>),
}

impl<S> Iterator for Vertices<S>
where
    S: BaseFloat,
{
    type Item = Point2<S>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Vertices::Butt => None,
            Vertices::Round(ref mut iter) => iter.next(),
            Vertices::Square(ref mut iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<S> ExactSizeIterator for Vertices<S>
where
    S: BaseFloat,
{
    fn len(&self) -> usize {
        match *self {
            Vertices::Butt => 0,
            Vertices::Round(ref iter) => iter.len(),
            Vertices::Square(ref iter) => iter.len(),
        }
    }
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

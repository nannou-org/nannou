//! Items related to cube geometry.
//!
//! The main type is the `Cuboid` type.

use crate::geom::{quad, scalar, Quad, Range, Scalar, Tri};
use crate::math::num_traits::Float;

/// The number of faces on a Cuboid.
pub const NUM_FACES: u8 = 6;

/// The number of corners on a Cuboid.
pub const NUM_CORNERS: u8 = 8;

/// The number of subdivisions for a Cuboid.
pub const NUM_SUBDIVISIONS: u8 = 8;

/// The number of triangles used to triangulate a cuboid.
pub const NUM_TRIANGLES: u8 = NUM_FACES * 2;

/// A light-weight `Cuboid` type with many helper and utility methods.
///
/// The cuboid is also known as a "rectangular prism".
///
/// `Cuboid` is implemented similarly to `geom::Rect` but with 3 axes instead of 2.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Cuboid<S = scalar::Default> {
    /// The start and end along the x axis.
    pub x: Range<S>,
    /// The start and end along the y axis.
    pub y: Range<S>,
    /// The start and end along the z axis.
    pub z: Range<S>,
}

/// Each of the faces of a cuboid.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Face {
    Back,
    Right,
    Top,
    Front,
    Bottom,
    Left,
}

/// An iterator yielding each corner of a cuboid in the following order.
#[derive(Clone, Debug)]
pub struct Corners<'a, S: 'a> {
    cuboid: &'a Cuboid<S>,
    corner_index: u8,
}

/// An iterator yielding the faces of a cuboid as per their ordering.
#[derive(Clone, Debug)]
pub struct Faces {
    next_face_index: u8,
}

/// A quad representing a single face of a cuboid.
pub type FaceQuad<S> = Quad<[S; 3]>;

/// An iterator yielding each face of a cuboid as a quad.
#[derive(Clone, Debug)]
pub struct FaceQuads<'a, S: 'a = scalar::Default> {
    // The cuboid object from which each face will be yielded.
    cuboid: &'a Cuboid<S>,
    // The next face to yield.
    faces: Faces,
}

/// An iterator yielding all triangles for all faces.
#[derive(Clone, Debug)]
pub struct Triangles<'a, S: 'a> {
    face_quads: FaceQuads<'a, S>,
    triangles: quad::Triangles<[S; 3]>,
}

/// The three ranges that make up the 8 subdivisions of a cuboid.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SubdivisionRanges<S> {
    /// The first half of the *x* axis range.
    pub x_a: Range<S>,
    /// The second half of the *x* axis range.
    pub x_b: Range<S>,
    /// The first half of the *y* axis range.
    pub y_a: Range<S>,
    /// The second half of the *y* axis range.
    pub y_b: Range<S>,
    /// The first half of the *z* axis range.
    pub z_a: Range<S>,
    /// The second half of the *z* axis range.
    pub z_b: Range<S>,
}

/// Yields even subdivisions of a `Cuboid`.
///
/// The eight subdivisions will each be yielded as a `Cuboid` whose dimensions are exactly half of
/// the original `Cuboid`.
#[derive(Clone)]
pub struct Subdivisions<S = scalar::Default> {
    ranges: SubdivisionRanges<S>,
    subdivision_index: u8,
}

macro_rules! corner_from_index {
    (0, $cuboid:expr) => {
        [$cuboid.x.start, $cuboid.y.start, $cuboid.z.start]
    };
    (1, $cuboid:expr) => {
        [$cuboid.x.end, $cuboid.y.start, $cuboid.z.start]
    };
    (2, $cuboid:expr) => {
        [$cuboid.x.start, $cuboid.y.end, $cuboid.z.start]
    };
    (3, $cuboid:expr) => {
        [$cuboid.x.end, $cuboid.y.end, $cuboid.z.start]
    };
    (4, $cuboid:expr) => {
        [$cuboid.x.start, $cuboid.y.start, $cuboid.z.end]
    };
    (5, $cuboid:expr) => {
        [$cuboid.x.end, $cuboid.y.start, $cuboid.z.end]
    };
    (6, $cuboid:expr) => {
        [$cuboid.x.start, $cuboid.y.end, $cuboid.z.end]
    };
    (7, $cuboid:expr) => {
        [$cuboid.x.end, $cuboid.y.end, $cuboid.z.end]
    };
}

macro_rules! face_from_index {
    (0) => {
        Face::Back
    };
    (1) => {
        Face::Right
    };
    (2) => {
        Face::Top
    };
    (3) => {
        Face::Front
    };
    (4) => {
        Face::Bottom
    };
    (5) => {
        Face::Left
    };
}

macro_rules! quad_from_corner_indices {
    ($cuboid:expr, $a:tt, $b:tt, $c:tt, $d:tt) => {
        [
            corner_from_index!($a, $cuboid).into(),
            corner_from_index!($b, $cuboid).into(),
            corner_from_index!($c, $cuboid).into(),
            corner_from_index!($d, $cuboid).into(),
        ]
    };
}

// Given some `SubdivisionRanges` and a subdivision index, produce the cuboid for that subdivision.
//
// 1. Front bottom left
// 2. Front bottom right
// 3. Front top left
// 4. Front top right
// 5. Back bottom left
// 6. Back bottom right
// 7. Back top left
// 8. Back top right
macro_rules! subdivision_from_index {
    ($ranges:expr,0) => {
        Cuboid {
            x: $ranges.x_a,
            y: $ranges.y_a,
            z: $ranges.z_a,
        }
    };
    ($ranges:expr,1) => {
        Cuboid {
            x: $ranges.x_b,
            y: $ranges.y_a,
            z: $ranges.z_a,
        }
    };
    ($ranges:expr,2) => {
        Cuboid {
            x: $ranges.x_a,
            y: $ranges.y_b,
            z: $ranges.z_a,
        }
    };
    ($ranges:expr,3) => {
        Cuboid {
            x: $ranges.x_b,
            y: $ranges.y_b,
            z: $ranges.z_a,
        }
    };
    ($ranges:expr,4) => {
        Cuboid {
            x: $ranges.x_a,
            y: $ranges.y_a,
            z: $ranges.z_b,
        }
    };
    ($ranges:expr,5) => {
        Cuboid {
            x: $ranges.x_b,
            y: $ranges.y_a,
            z: $ranges.z_b,
        }
    };
    ($ranges:expr,6) => {
        Cuboid {
            x: $ranges.x_a,
            y: $ranges.y_b,
            z: $ranges.z_b,
        }
    };
    ($ranges:expr,7) => {
        Cuboid {
            x: $ranges.x_b,
            y: $ranges.y_b,
            z: $ranges.z_b,
        }
    };
}

impl<S> Cuboid<S>
where
    S: Float + Scalar,
{
    /// Construct a Rect from a given centre point (x, y, z) and dimensions (width, height, depth).
    pub fn from_xyz_whd([x, y, z]: [S; 3], [w, h, d]: [S; 3]) -> Self {
        Cuboid {
            x: Range::from_pos_and_len(x, w),
            y: Range::from_pos_and_len(y, h),
            z: Range::from_pos_and_len(z, d),
        }
    }

    /// The position in the middle of the x range.
    pub fn x(&self) -> S {
        self.x.middle()
    }

    /// The position in the middle of the y range.
    pub fn y(&self) -> S {
        self.y.middle()
    }

    /// The position in the middle of the z range.
    pub fn z(&self) -> S {
        self.z.middle()
    }

    /// The xyz position in the middle of the bounds.
    pub fn xyz(&self) -> [S; 3] {
        [self.x(), self.y(), self.z()].into()
    }

    /// The centered x, y and z coordinates as a tuple.
    pub fn x_y_z(&self) -> (S, S, S) {
        (self.x(), self.y(), self.z())
    }

    /// The six ranges used for the `Cuboid`'s eight subdivisions.
    pub fn subdivision_ranges(&self) -> SubdivisionRanges<S> {
        let (x, y, z) = self.x_y_z();
        let x_a = Range::new(self.x.start, x);
        let x_b = Range::new(x, self.x.end);
        let y_a = Range::new(self.y.start, y);
        let y_b = Range::new(y, self.y.end);
        let z_a = Range::new(self.z.start, z);
        let z_b = Range::new(z, self.z.end);
        SubdivisionRanges {
            x_a,
            x_b,
            y_a,
            y_b,
            z_a,
            z_b,
        }
    }

    /// The position and dimensions of the cuboid.
    pub fn xyz_whd(&self) -> ([S; 3], [S; 3]) {
        (self.xyz(), self.whd())
    }

    /// The position and dimensions of the cuboid.
    pub fn x_y_z_w_h_d(&self) -> (S, S, S, S, S, S) {
        let (x, y, z) = self.x_y_z();
        let (w, h, d) = self.w_h_d();
        (x, y, z, w, h, d)
    }
}

impl<S> Cuboid<S>
where
    S: Scalar,
{
    /// Construct a cuboid from its x, y and z ranges.
    pub fn from_ranges(x: Range<S>, y: Range<S>, z: Range<S>) -> Self {
        Cuboid { x, y, z }
    }

    /// Converts `self` to an absolute `Cuboid` so that the magnitude of each range is always
    /// positive.
    pub fn absolute(&self) -> Self {
        let x = self.x.absolute();
        let y = self.y.absolute();
        let z = self.z.absolute();
        Cuboid { x, y, z }
    }

    /// Shift the cuboid along the x axis.
    pub fn shift_x(self, x: S) -> Self {
        Cuboid {
            x: self.x.shift(x),
            ..self
        }
    }

    /// Shift the cuboid along the y axis.
    pub fn shift_y(self, y: S) -> Self {
        Cuboid {
            y: self.y.shift(y),
            ..self
        }
    }

    /// Shift the cuboid along the z axis.
    pub fn shift_z(self, z: S) -> Self {
        Cuboid {
            z: self.z.shift(z),
            ..self
        }
    }

    /// Shift the cuboid by the given vector.
    pub fn shift(self, [x, y, z]: [S; 3]) -> Self {
        Cuboid {
            x: self.x.shift(x),
            y: self.y.shift(y),
            z: self.z.shift(z),
        }
    }

    /// Does the given cuboid contain the given point.
    pub fn contains(&self, [x, y, z]: [S; 3]) -> bool {
        self.x.contains(x) && self.y.contains(y) && self.z.contains(z)
    }

    /// Stretches the closest side(s) to the given point if the point lies outside of the Cuboid
    /// area.
    pub fn stretch_to_point(self, [px, py, pz]: [S; 3]) -> Self {
        let Cuboid { x, y, z } = self;
        Cuboid {
            x: x.stretch_to_value(px),
            y: y.stretch_to_value(py),
            z: z.stretch_to_value(pz),
        }
    }

    /// The cuboid representing the area in which two cuboids overlap.
    pub fn overlap(self, other: Self) -> Option<Self> {
        self.x.overlap(other.x).and_then(|x| {
            self.y
                .overlap(other.y)
                .and_then(|y| self.z.overlap(other.z).map(|z| Cuboid { x, y, z }))
        })
    }

    /// The cuboid that encompass the two given cuboids.
    pub fn max(self, other: Self) -> Self
    where
        S: Float,
    {
        Cuboid {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.y),
        }
    }

    /// The start of the range along the x axis.
    pub fn left(&self) -> S {
        self.x.start
    }

    /// The end of the range along the x axis.
    pub fn right(&self) -> S {
        self.x.end
    }

    /// The start of the range along the y axis.
    pub fn bottom(&self) -> S {
        self.y.start
    }

    /// The end of the range along the y axis.
    pub fn top(&self) -> S {
        self.y.end
    }

    /// The start of the range along the z axis.
    pub fn front(&self) -> S {
        self.z.start
    }

    /// The end of the range along the z axis.
    pub fn back(&self) -> S {
        self.z.end
    }

    /// The quad for the face at the start of the range along the x axis.
    pub fn left_quad(&self) -> FaceQuad<S> {
        Quad(quad_from_corner_indices!(self, 4, 6, 2, 0))
    }

    /// The quad for the face at the end of the range along the x axis.
    pub fn right_quad(&self) -> FaceQuad<S> {
        Quad(quad_from_corner_indices!(self, 1, 3, 7, 5))
    }

    /// The quad for the face at the start of the range along the y axis.
    pub fn bottom_quad(&self) -> FaceQuad<S> {
        Quad(quad_from_corner_indices!(self, 0, 1, 5, 4))
    }

    /// The quad for the face at the end of the range along the y axis.
    pub fn top_quad(&self) -> FaceQuad<S> {
        Quad(quad_from_corner_indices!(self, 2, 6, 7, 3))
    }

    /// The quad for the face at the start of the range along the z axis.
    pub fn front_quad(&self) -> FaceQuad<S> {
        Quad(quad_from_corner_indices!(self, 0, 2, 3, 1))
    }

    /// The quad for the face at the end of the range along the z axis.
    pub fn back_quad(&self) -> FaceQuad<S> {
        Quad(quad_from_corner_indices!(self, 5, 7, 6, 4))
    }

    /// The quad for the given face.
    pub fn face_quad(&self, face: Face) -> FaceQuad<S> {
        match face {
            Face::Front => self.front_quad(),
            Face::Right => self.right_quad(),
            Face::Back => self.back_quad(),
            Face::Left => self.left_quad(),
            Face::Bottom => self.bottom_quad(),
            Face::Top => self.top_quad(),
        }
    }

    /// The 8 corners of the cuboid in the following order:
    ///
    /// ```ignore
    /// y
    /// | z
    /// |/
    /// 0---x
    ///
    ///   6---7
    ///  /|  /|
    /// 2---3 |
    /// | 4-|-5
    /// |/  |/
    /// 0---1
    /// ```
    pub fn corners(&self) -> [[S; 3]; NUM_CORNERS as usize] {
        let a = [self.x.start, self.y.start, self.z.start].into();
        let b = [self.x.end, self.y.start, self.z.start].into();
        let c = [self.x.start, self.y.end, self.z.start].into();
        let d = [self.x.end, self.y.end, self.z.start].into();
        let e = [self.x.start, self.y.start, self.z.end].into();
        let f = [self.x.end, self.y.start, self.z.end].into();
        let g = [self.x.start, self.y.end, self.z.end].into();
        let h = [self.x.end, self.y.end, self.z.end].into();
        [a, b, c, d, e, f, g, h]
    }

    /// The same as `corners` but produces an iterator rather than a fixed-size array.
    pub fn corners_iter(&self) -> Corners<S> {
        Corners {
            cuboid: self,
            corner_index: 0,
        }
    }

    /// The 6 faces of the of the cuboid in the order yielded by the `Faces` iterator.
    pub fn faces(&self) -> [FaceQuad<S>; NUM_FACES as usize] {
        let mut faces = self.faces_iter();
        [
            faces.next().unwrap(),
            faces.next().unwrap(),
            faces.next().unwrap(),
            faces.next().unwrap(),
            faces.next().unwrap(),
            faces.next().unwrap(),
        ]
    }

    /// An iterator yielding a quad for each face on the cuboid.
    pub fn faces_iter(&self) -> FaceQuads<S> {
        FaceQuads {
            faces: Faces { next_face_index: 0 },
            cuboid: self,
        }
    }

    /// Produce an iterator yielding every triangle in the cuboid (two for each face).
    ///
    /// Uses the `faces_iter` method internally.
    pub fn triangles_iter(&self) -> Triangles<S> {
        let mut face_quads = self.faces_iter();
        let first_quad = face_quads.next().unwrap();
        let triangles = first_quad.triangles_iter();
        Triangles {
            face_quads,
            triangles,
        }
    }

    /// The length of the cuboid along the *x* axis (aka `width` or `w` for short).
    pub fn w(&self) -> S {
        self.x.len()
    }

    /// The length of the cuboid along the *y* axis (aka `height` or `h` for short).
    pub fn h(&self) -> S {
        self.y.len()
    }

    /// The length of the cuboid along the *z* axis (aka `depth` or `d` for short).
    pub fn d(&self) -> S {
        self.z.len()
    }

    /// The dimensions (width, height and depth) of the cuboid as a vector.
    pub fn whd(&self) -> [S; 3] {
        [self.w(), self.h(), self.d()].into()
    }

    /// The dimensions (width, height and depth) of the cuboid as a tuple.
    pub fn w_h_d(&self) -> (S, S, S) {
        (self.w(), self.h(), self.d())
    }

    /// The total volume of the cuboid.
    pub fn volume(&self) -> S {
        let (w, h, d) = self.w_h_d();
        w * h * d
    }

    /// The cuboid with some padding applied to the left side.
    pub fn pad_left(self, pad: S) -> Self {
        Cuboid {
            x: self.x.pad_start(pad),
            ..self
        }
    }

    /// The cuboid with some padding applied to the right side.
    pub fn pad_right(self, pad: S) -> Self {
        Cuboid {
            x: self.x.pad_end(pad),
            ..self
        }
    }

    /// The cuboid with some padding applied to the bottom side.
    pub fn pad_bottom(self, pad: S) -> Self {
        Cuboid {
            y: self.y.pad_start(pad),
            ..self
        }
    }

    /// The cuboid with some padding applied to the top side.
    pub fn pad_top(self, pad: S) -> Self {
        Cuboid {
            y: self.y.pad_end(pad),
            ..self
        }
    }

    /// The cuboid with some padding applied to the front side.
    pub fn pad_front(self, pad: S) -> Self {
        Cuboid {
            z: self.z.pad_start(pad),
            ..self
        }
    }

    /// The cuboid with some padding applied to the back side.
    pub fn pad_back(self, pad: S) -> Self {
        Cuboid {
            z: self.z.pad_end(pad),
            ..self
        }
    }

    /// The cuboid with some padding amount applied to each side.
    pub fn pad(self, pad: S) -> Self {
        let Cuboid { x, y, z } = self;
        Cuboid {
            x: x.pad(pad),
            y: y.pad(pad),
            z: z.pad(pad),
        }
    }
}

impl<S> SubdivisionRanges<S>
where
    S: Copy,
{
    /// The `Cuboid`s representing each of the eight subdivisions.
    ///
    /// Subdivisions are yielded in the following order:
    ///
    /// 1. Front bottom left
    /// 2. Front bottom right
    /// 3. Front top left
    /// 4. Front top right
    /// 5. Back bottom left
    /// 6. Back bottom right
    /// 7. Back top left
    /// 8. Back top right
    pub fn cuboids(&self) -> [Cuboid<S>; NUM_SUBDIVISIONS as usize] {
        let c1 = subdivision_from_index!(self, 0);
        let c2 = subdivision_from_index!(self, 1);
        let c3 = subdivision_from_index!(self, 2);
        let c4 = subdivision_from_index!(self, 3);
        let c5 = subdivision_from_index!(self, 4);
        let c6 = subdivision_from_index!(self, 5);
        let c7 = subdivision_from_index!(self, 6);
        let c8 = subdivision_from_index!(self, 7);
        [c1, c2, c3, c4, c5, c6, c7, c8]
    }

    /// The same as `cuboids` but each subdivision is yielded via the returned `Iterator`.
    pub fn cuboids_iter(self) -> Subdivisions<S> {
        Subdivisions {
            ranges: self,
            subdivision_index: 0,
        }
    }

    // The subdivision at the given index within the range 0..NUM_SUBDIVISIONS.
    fn subdivision_at_index(&self, index: u8) -> Option<Cuboid<S>> {
        let cuboid = match index {
            0 => subdivision_from_index!(self, 0),
            1 => subdivision_from_index!(self, 1),
            2 => subdivision_from_index!(self, 2),
            3 => subdivision_from_index!(self, 3),
            4 => subdivision_from_index!(self, 4),
            5 => subdivision_from_index!(self, 5),
            6 => subdivision_from_index!(self, 6),
            7 => subdivision_from_index!(self, 7),
            _ => return None,
        };
        Some(cuboid)
    }
}

fn corner_from_index<S>(c: &Cuboid<S>, index: u8) -> Option<[S; 3]>
where
    S: Copy,
{
    let p = match index {
        0 => corner_from_index!(0, c),
        1 => corner_from_index!(1, c),
        2 => corner_from_index!(2, c),
        3 => corner_from_index!(3, c),
        4 => corner_from_index!(4, c),
        5 => corner_from_index!(5, c),
        6 => corner_from_index!(6, c),
        7 => corner_from_index!(7, c),
        _ => return None,
    };
    Some(p.into())
}

impl<'a, S> Iterator for Corners<'a, S>
where
    S: Copy,
{
    type Item = [S; 3];
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = corner_from_index(self.cuboid, self.corner_index) {
            self.corner_index += 1;
            return Some(p);
        }
        None
    }
}

impl<'a, S> DoubleEndedIterator for Corners<'a, S>
where
    S: Copy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_index = self.corner_index + 1;
        if let Some(p) = corner_from_index(self.cuboid, NUM_CORNERS - self.corner_index) {
            self.corner_index = next_index;
            return Some(p);
        }
        None
    }
}

impl<'a, S> ExactSizeIterator for Corners<'a, S>
where
    S: Copy,
{
    fn len(&self) -> usize {
        NUM_CORNERS as usize - self.corner_index as usize
    }
}

impl Face {
    /// Produce a face from an index into the order in which faces are yielded by the cuboid
    /// `Faces` iterator.
    fn from_index(i: u8) -> Option<Self> {
        let face = match i {
            0 => face_from_index!(0),
            1 => face_from_index!(1),
            2 => face_from_index!(2),
            3 => face_from_index!(3),
            4 => face_from_index!(4),
            5 => face_from_index!(5),
            _ => return None,
        };
        Some(face)
    }
}

impl<'a, S> FaceQuads<'a, S> {}

impl Iterator for Faces {
    type Item = Face;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(face) = Face::from_index(self.next_face_index) {
            self.next_face_index += 1;
            return Some(face);
        }
        None
    }
}

impl DoubleEndedIterator for Faces {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_face_index = self.next_face_index + 1;
        if let Some(face) = Face::from_index(NUM_FACES - next_face_index) {
            self.next_face_index = next_face_index;
            return Some(face);
        }
        None
    }
}

impl ExactSizeIterator for Faces {
    fn len(&self) -> usize {
        NUM_FACES as usize - self.next_face_index as usize
    }
}

impl<'a, S> Iterator for FaceQuads<'a, S>
where
    S: Scalar,
{
    type Item = FaceQuad<S>;
    fn next(&mut self) -> Option<Self::Item> {
        self.faces.next().map(|f| self.cuboid.face_quad(f))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.faces.size_hint()
    }
}

impl<'a, S> DoubleEndedIterator for FaceQuads<'a, S>
where
    S: Scalar,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.faces.next_back().map(|f| self.cuboid.face_quad(f))
    }
}

impl<'a, S> ExactSizeIterator for FaceQuads<'a, S>
where
    S: Scalar,
{
    fn len(&self) -> usize {
        self.faces.len()
    }
}

impl<'a, S> Iterator for Triangles<'a, S>
where
    S: Scalar,
{
    type Item = Tri<[S; 3]>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(tri) = self.triangles.next() {
                return Some(tri);
            }
            self.triangles = match self.face_quads.next() {
                Some(quad) => quad.triangles_iter(),
                None => return None,
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<'a, S> DoubleEndedIterator for Triangles<'a, S>
where
    S: Scalar,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(tri) = self.triangles.next_back() {
                return Some(tri);
            }
            self.triangles = match self.face_quads.next_back() {
                Some(quad) => quad.triangles_iter(),
                None => return None,
            }
        }
    }
}

impl<'a, S> ExactSizeIterator for Triangles<'a, S>
where
    S: Scalar,
{
    fn len(&self) -> usize {
        let remaining_triangles = self.triangles.len();
        let remaining_quads = self.face_quads.len();
        remaining_triangles + remaining_quads * 2
    }
}

impl<S> Iterator for Subdivisions<S>
where
    S: Copy,
{
    type Item = Cuboid<S>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(sd) = self.ranges.subdivision_at_index(self.subdivision_index) {
            self.subdivision_index += 1;
            return Some(sd);
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<S> DoubleEndedIterator for Subdivisions<S>
where
    S: Copy,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_index = self.subdivision_index + 1;
        if let Some(sd) = self
            .ranges
            .subdivision_at_index(NUM_SUBDIVISIONS - next_index)
        {
            self.subdivision_index = next_index;
            return Some(sd);
        }
        None
    }
}

impl<S> ExactSizeIterator for Subdivisions<S>
where
    S: Copy,
{
    fn len(&self) -> usize {
        NUM_SUBDIVISIONS as usize - self.subdivision_index as usize
    }
}

use crate::geom::{quad, scalar, Align, Edge, Point2, Quad, Range, Scalar, Tri};
use crate::glam::{DVec2, Vec2};
use crate::math::{self, num_traits::Float};
use core::ops::Neg;

/// Defines a Rectangle's bounds across the x and y axes.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Rect<S = scalar::Default> {
    /// The start and end positions of the Rectangle on the x axis.
    pub x: Range<S>,
    /// The start and end positions of the Rectangle on the y axis.
    pub y: Range<S>,
}

/// The distance between the inner edge of a border and the outer edge of the inner content.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Padding<S = scalar::Default> {
    /// Padding on the start and end of the *x* axis.
    pub x: Range<S>,
    /// Padding on the start and end of the *y* axis.
    pub y: Range<S>,
}

/// Either of the four corners of a **Rect**.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Corner {
    /// The top left corner of a **Rect**.
    TopLeft,
    /// The top right corner of a **Rect**.
    TopRight,
    /// The bottom left corner of a **Rect**.
    BottomLeft,
    /// The bottom right corner of a **Rect**.
    BottomRight,
}

/// Yields even subdivisions of a `Rect`.
///
/// The four subdivisions will each be yielded as a `Rect` whose dimensions are exactly half of the
/// original `Rect`.
#[derive(Clone)]
pub struct Subdivisions<S = scalar::Default> {
    ranges: SubdivisionRanges<S>,
    subdivision_index: u8,
}

/// The ranges that describe the subdivisions of a `Rect`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct SubdivisionRanges<S = scalar::Default> {
    /// The first half of the x range.
    pub x_a: Range<S>,
    /// The second half of the x range.
    pub x_b: Range<S>,
    /// The first half of the y range.
    pub y_a: Range<S>,
    /// The second half of the y range.
    pub y_b: Range<S>,
}

/// An iterator yielding the four corners of a `Rect`.
#[derive(Clone, Debug)]
pub struct Corners<S = scalar::Default> {
    rect: Rect<S>,
    index: u8,
}

/// The triangles iterator yielded by the `Rect`.
pub type Triangles<S> = quad::Triangles<[S; 2]>;

/// The number of subdivisions when dividing a `Rect` in half along the *x* and *y* axes.
pub const NUM_SUBDIVISIONS: u8 = 4;

/// The number of subdivisions when dividing a `Rect` in half along the *x* and *y* axes.
pub const NUM_CORNERS: u8 = 4;

/// The number of triangles used to represent a `Rect`.
pub const NUM_TRIANGLES: u8 = 2;

impl<S> Padding<S>
where
    S: Scalar,
{
    /// No padding.
    pub fn none() -> Self {
        Padding {
            x: Range::new(S::zero(), S::zero()),
            y: Range::new(S::zero(), S::zero()),
        }
    }
}

// Given some `SubdivisionRanges` and a subdivision index, produce the rect for that subdivision.
macro_rules! subdivision_from_index {
    ($ranges:expr,0) => {
        Rect {
            x: $ranges.x_a,
            y: $ranges.y_a,
        }
    };
    ($ranges:expr,1) => {
        Rect {
            x: $ranges.x_b,
            y: $ranges.y_a,
        }
    };
    ($ranges:expr,2) => {
        Rect {
            x: $ranges.x_a,
            y: $ranges.y_b,
        }
    };
    ($ranges:expr,3) => {
        Rect {
            x: $ranges.x_b,
            y: $ranges.y_b,
        }
    };
}

// Given some `Rect` and an index, produce the corner for that index.
macro_rules! corner_from_index {
    ($rect:expr,0) => {
        [$rect.x.start, $rect.y.end]
    };
    ($rect:expr,1) => {
        [$rect.x.end, $rect.y.end]
    };
    ($rect:expr,2) => {
        [$rect.x.end, $rect.y.start]
    };
    ($rect:expr,3) => {
        [$rect.x.start, $rect.y.start]
    };
}

impl<S> Rect<S>
where
    S: Scalar + Float,
{
    /// Construct a Rect from the given `x` `y` coordinates and `w` `h` dimensions.
    pub fn from_x_y_w_h(x: S, y: S, w: S, h: S) -> Self {
        Rect {
            x: Range::from_pos_and_len(x, w),
            y: Range::from_pos_and_len(y, h),
        }
    }

    /// Construct a Rect at origin with the given width and height.
    pub fn from_w_h(w: S, h: S) -> Self {
        Self::from_x_y_w_h(S::zero(), S::zero(), w, h)
    }

    /// The position in the middle of the x bounds.
    pub fn x(&self) -> S {
        self.x.middle()
    }

    /// The position in the middle of the y bounds.
    pub fn y(&self) -> S {
        self.y.middle()
    }

    /// The centered x and y coordinates as a tuple.
    pub fn x_y(&self) -> (S, S) {
        (self.x(), self.y())
    }

    /// The Rect's centered coordinates and dimensions in a tuple.
    pub fn x_y_w_h(&self) -> (S, S, S, S) {
        let (x, y) = self.x_y();
        let (w, h) = self.w_h();
        (x, y, w, h)
    }

    /// Align `self` to `other` along the *x* axis in accordance with the given `Align` variant.
    pub fn align_x_of(self, align: Align, other: Self) -> Self {
        Rect {
            x: self.x.align_to(align, other.x),
            y: self.y,
        }
    }

    /// Align `self` to `other` along the *y* axis in accordance with the given `Align` variant.
    pub fn align_y_of(self, align: Align, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_to(align, other.y),
        }
    }

    /// Align the middle of `self` with the middle of the `other` **Rect** along the *x* axis.
    pub fn align_middle_x_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_middle_of(other.x),
            y: self.y,
        }
    }

    /// Align the middle of `self` with the middle of the `other` **Rect** along the *y* axis.
    pub fn align_middle_y_of(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_middle_of(other.y),
        }
    }

    /// Place `self` in the middle of the top edge of the `other` **Rect**.
    pub fn mid_top_of(self, other: Self) -> Self {
        self.align_middle_x_of(other).align_top_of(other)
    }

    /// Place `self` in the middle of the bottom edge of the `other` **Rect**.
    pub fn mid_bottom_of(self, other: Self) -> Self {
        self.align_middle_x_of(other).align_bottom_of(other)
    }

    /// Place `self` in the middle of the left edge of the `other` **Rect**.
    pub fn mid_left_of(self, other: Self) -> Self {
        self.align_left_of(other).align_middle_y_of(other)
    }

    /// Place `self` in the middle of the right edge of the `other` **Rect**.
    pub fn mid_right_of(self, other: Self) -> Self {
        self.align_right_of(other).align_middle_y_of(other)
    }

    /// Place `self` directly in the middle of the `other` **Rect**.
    pub fn middle_of(self, other: Self) -> Self {
        self.align_middle_x_of(other).align_middle_y_of(other)
    }

    /// The four ranges used for the `Rect`'s four subdivisions.
    pub fn subdivision_ranges(&self) -> SubdivisionRanges<S> {
        let (x, y) = self.x_y();
        let x_a = Range::new(self.x.start, x);
        let x_b = Range::new(x, self.x.end);
        let y_a = Range::new(self.y.start, y);
        let y_b = Range::new(y, self.y.end);
        SubdivisionRanges { x_a, x_b, y_a, y_b }
    }

    /// Divide the `Rect` in half along the *x* and *y* axes and return the four subdivisions.
    ///
    /// Subdivisions are yielded in the following order:
    ///
    /// 1. Bottom left
    /// 2. Bottom right
    /// 3. Top left
    /// 4. Top right
    pub fn subdivisions(&self) -> [Self; NUM_SUBDIVISIONS as usize] {
        self.subdivision_ranges().rects()
    }

    /// The same as `subdivisions` but each subdivision is yielded via the returned `Iterator`.
    pub fn subdivisions_iter(&self) -> Subdivisions<S> {
        self.subdivision_ranges().rects_iter()
    }

    /// Creates a rect with the specified ratio that fit in `self`.
    ///
    /// ratio = width / height
    pub fn with_ratio(&self, ratio: S) -> Self {
        let (w, h) = self.w_h();
        if w < h * ratio {
            Rect::from_w_h(w, w / ratio)
        } else {
            Rect::from_w_h(h * ratio, h)
        }
    }
}

impl<S> Rect<S>
where
    S: Scalar,
{
    /// Construct a Rect from the coordinates of two points.
    pub fn from_corner_points([ax, ay]: [S; 2], [bx, by]: [S; 2]) -> Self {
        let (left, right) = if ax < bx { (ax, bx) } else { (bx, ax) };
        let (bottom, top) = if ay < by { (ay, by) } else { (by, ay) };
        Rect {
            x: Range {
                start: left,
                end: right,
            },
            y: Range {
                start: bottom,
                end: top,
            },
        }
    }

    /// Converts `self` to an absolute `Rect` so that the magnitude of each range is always
    /// positive.
    pub fn absolute(self) -> Self {
        let x = self.x.absolute();
        let y = self.y.absolute();
        Rect { x, y }
    }

    /// The Rect representing the area in which two Rects overlap.
    pub fn overlap(self, other: Self) -> Option<Self> {
        self.x
            .overlap(other.x)
            .and_then(|x| self.y.overlap(other.y).map(|y| Rect { x: x, y: y }))
    }

    /// The Rect that encompass the two given sets of Rect.
    pub fn max(self, other: Self) -> Self
    where
        S: Float,
    {
        Rect {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    /// The Rect's lowest y value.
    pub fn bottom(&self) -> S {
        self.y.absolute().start
    }

    /// The Rect's highest y value.
    pub fn top(&self) -> S {
        self.y.absolute().end
    }

    /// The Rect's lowest x value.
    pub fn left(&self) -> S {
        self.x.absolute().start
    }

    /// The Rect's highest x value.
    pub fn right(&self) -> S {
        self.x.absolute().end
    }

    /// The edges of the **Rect** in a tuple (left, right, bottom, top).
    pub fn l_r_b_t(&self) -> (S, S, S, S) {
        (self.left(), self.right(), self.bottom(), self.top())
    }

    /// Shift the Rect along the x axis.
    pub fn shift_x(self, x: S) -> Self {
        Rect {
            x: self.x.shift(x),
            ..self
        }
    }

    /// Shift the Rect along the y axis.
    pub fn shift_y(self, y: S) -> Self {
        Rect {
            y: self.y.shift(y),
            ..self
        }
    }

    /// Align `self`'s right edge with the left edge of the `other` **Rect**.
    pub fn left_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_before(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s left edge with the right dge of the `other` **Rect**.
    pub fn right_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_after(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s top edge with the bottom edge of the `other` **Rect**.
    pub fn below(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_before(other.y),
        }
    }

    /// Align `self`'s bottom edge with the top edge of the `other` **Rect**.
    pub fn above(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_after(other.y),
        }
    }

    /// Align `self`'s left edge with the left edge of the `other` **Rect**.
    pub fn align_left_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_start_of(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s right edge with the right edge of the `other` **Rect**.
    pub fn align_right_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_end_of(other.x),
            y: self.y,
        }
    }

    /// Align `self`'s bottom edge with the bottom edge of the `other` **Rect**.
    pub fn align_bottom_of(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_start_of(other.y),
        }
    }

    /// Align `self`'s top edge with the top edge of the `other` **Rect**.
    pub fn align_top_of(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_end_of(other.y),
        }
    }

    /// Place `self` along the top left edges of the `other` **Rect**.
    pub fn top_left_of(self, other: Self) -> Self {
        self.align_left_of(other).align_top_of(other)
    }

    /// Place `self` along the top right edges of the `other` **Rect**.
    pub fn top_right_of(self, other: Self) -> Self {
        self.align_right_of(other).align_top_of(other)
    }

    /// Place `self` along the bottom left edges of the `other` **Rect**.
    pub fn bottom_left_of(self, other: Self) -> Self {
        self.align_left_of(other).align_bottom_of(other)
    }

    /// Place `self` along the bottom right edges of the `other` **Rect**.
    pub fn bottom_right_of(self, other: Self) -> Self {
        self.align_right_of(other).align_bottom_of(other)
    }

    /// Does the given point touch the Rectangle.
    pub fn contains_point(self, [x, y]: [S; 2]) -> bool {
        self.x.contains(x) && self.y.contains(y)
    }

    /// Stretches the closest edge(s) to the given point if the point lies outside of the Rect area.
    pub fn stretch_to_point(self, [px, py]: [S; 2]) -> Self {
        let Rect { x, y } = self;
        Rect {
            x: x.stretch_to_value(px),
            y: y.stretch_to_value(py),
        }
    }

    /// Return the **Corner** of `self` that is closest to the given **Point**.
    pub fn closest_corner(&self, [x, y]: [S; 2]) -> Corner {
        let x_edge = self.x.closest_edge(x);
        let y_edge = self.y.closest_edge(y);
        match (x_edge, y_edge) {
            (Edge::Start, Edge::Start) => Corner::BottomLeft,
            (Edge::Start, Edge::End) => Corner::TopLeft,
            (Edge::End, Edge::Start) => Corner::BottomRight,
            (Edge::End, Edge::End) => Corner::TopRight,
        }
    }

    /// The four corners of the `Rect`.
    pub fn corners(&self) -> Quad<[S; 2]> {
        Quad::from([
            corner_from_index!(self, 0),
            corner_from_index!(self, 1),
            corner_from_index!(self, 2),
            corner_from_index!(self, 3),
        ])
    }

    /// An iterator yielding the four corners of the `Rect`.
    pub fn corners_iter(&self) -> Corners<S> {
        let rect = *self;
        let index = 0;
        Corners { rect, index }
    }

    /// Return two `Tri`s that represent the `Rect`.
    pub fn triangles(&self) -> (Tri<[S; 2]>, Tri<[S; 2]>) {
        self.corners().triangles()
    }

    /// An iterator yielding the `Rect`'s two `Tri`'s.
    pub fn triangles_iter(self) -> Triangles<S> {
        self.corners().triangles_iter()
    }

    /// Produce the corner at the given index.
    pub fn corner_at_index(&self, index: u8) -> Option<[S; 2]> {
        match index {
            0 => Some(corner_from_index!(self, 0)),
            1 => Some(corner_from_index!(self, 1)),
            2 => Some(corner_from_index!(self, 2)),
            3 => Some(corner_from_index!(self, 3)),
            _ => None,
        }
    }
}

impl Rect<f32> {
    /// Construct a Rect from a given `Point` and `Dimensions`.
    pub fn from_xy_wh(p: Point2, s: Vec2) -> Self {
        Self::from_x_y_w_h(p.x, p.y, s.x, s.y)
    }

    /// Construct a Rect at origin with the given dimensions.
    pub fn from_wh(s: Vec2) -> Self {
        Self::from_w_h(s.x, s.y)
    }

    /// Construct a Rect from the coordinates of two points.
    pub fn from_corners(a: Point2, b: Point2) -> Self {
        Self::from_corner_points(a.into(), b.into())
    }

    /// The xy position in the middle of the bounds.
    pub fn xy(&self) -> Point2 {
        [self.x(), self.y()].into()
    }

    /// The total dimensions of the Rect.
    pub fn wh(&self) -> Vec2 {
        [self.w(), self.h()].into()
    }

    /// Convert the Rect to a `Point` and `Dimensions`.
    pub fn xy_wh(&self) -> (Point2, Vec2) {
        (self.xy(), self.wh())
    }

    /// The top left corner **Point**.
    pub fn top_left(&self) -> Point2 {
        [self.left(), self.top()].into()
    }

    /// The bottom left corner **Point**.
    pub fn bottom_left(&self) -> Point2 {
        [self.left(), self.bottom()].into()
    }

    /// The top right corner **Point**.
    pub fn top_right(&self) -> Point2 {
        [self.right(), self.top()].into()
    }

    /// The bottom right corner **Point**.
    pub fn bottom_right(&self) -> Point2 {
        [self.right(), self.bottom()].into()
    }

    /// The middle of the left edge.
    pub fn mid_left(&self) -> Point2 {
        [self.left(), self.y()].into()
    }

    /// The middle of the top edge.
    pub fn mid_top(&self) -> Point2 {
        [self.x(), self.top()].into()
    }

    /// The middle of the right edge.
    pub fn mid_right(&self) -> Point2 {
        [self.right(), self.y()].into()
    }

    /// The middle of the bottom edge.
    pub fn mid_bottom(&self) -> Point2 {
        [self.x(), self.bottom()].into()
    }

    /// Shift the Rect by the given vector.
    pub fn shift(self, v: Vec2) -> Self {
        self.shift_x(v.x).shift_y(v.y)
    }

    /// Does the given point touch the Rectangle.
    pub fn contains(&self, p: Point2) -> bool {
        self.contains_point(p.into())
    }

    /// Stretches the closest edge(s) to the given point if the point lies outside of the Rect area.
    pub fn stretch_to(self, p: Point2) -> Self {
        self.stretch_to_point(p.into())
    }
}

impl Rect<f64> {
    /// Construct a Rect from a given `Point` and `Dimensions`.
    pub fn from_xy_wh_f64(p: DVec2, s: DVec2) -> Self {
        Self::from_x_y_w_h(p.x, p.y, s.x, s.y)
    }

    /// Construct a Rect at origin with the given dimensions.
    pub fn from_wh_f64(s: DVec2) -> Self {
        Self::from_w_h(s.x, s.y)
    }

    /// Construct a Rect from the coordinates of two points.
    pub fn from_corners_f64(a: DVec2, b: DVec2) -> Self {
        Self::from_corner_points(a.into(), b.into())
    }

    /// The xy position in the middle of the bounds.
    pub fn xy(&self) -> DVec2 {
        [self.x(), self.y()].into()
    }

    /// The total dimensions of the Rect.
    pub fn wh(&self) -> DVec2 {
        [self.w(), self.h()].into()
    }

    /// Convert the Rect to a `Point` and `Dimensions`.
    pub fn xy_wh(&self) -> (DVec2, DVec2) {
        (self.xy(), self.wh())
    }

    /// The top left corner **Point**.
    pub fn top_left(&self) -> DVec2 {
        [self.left(), self.top()].into()
    }

    /// The bottom left corner **Point**.
    pub fn bottom_left(&self) -> DVec2 {
        [self.left(), self.bottom()].into()
    }

    /// The top right corner **Point**.
    pub fn top_right(&self) -> DVec2 {
        [self.right(), self.top()].into()
    }

    /// The bottom right corner **Point**.
    pub fn bottom_right(&self) -> DVec2 {
        [self.right(), self.bottom()].into()
    }

    /// The middle of the left edge.
    pub fn mid_left(&self) -> DVec2 {
        [self.left(), self.y()].into()
    }

    /// The middle of the top edge.
    pub fn mid_top(&self) -> DVec2 {
        [self.x(), self.top()].into()
    }

    /// The middle of the right edge.
    pub fn mid_right(&self) -> DVec2 {
        [self.right(), self.y()].into()
    }

    /// The middle of the bottom edge.
    pub fn mid_bottom(&self) -> DVec2 {
        [self.x(), self.bottom()].into()
    }

    /// Shift the Rect by the given vector.
    pub fn shift(self, v: DVec2) -> Self {
        self.shift_x(v.x).shift_y(v.y)
    }

    /// Does the given point touch the Rectangle.
    pub fn contains(&self, p: DVec2) -> bool {
        self.contains_point(p.into())
    }

    /// Stretches the closest edge(s) to the given point if the point lies outside of the Rect area.
    pub fn stretch_to(self, p: DVec2) -> Self {
        self.stretch_to_point(p.into())
    }
}

impl<S> SubdivisionRanges<S>
where
    S: Copy,
{
    /// The `Rect`s representing each of the four subdivisions.
    ///
    /// Subdivisions are yielded in the following order:
    ///
    /// 1. Bottom left
    /// 2. Bottom right
    /// 3. Top left
    /// 4. Top right
    pub fn rects(&self) -> [Rect<S>; NUM_SUBDIVISIONS as usize] {
        let r1 = subdivision_from_index!(self, 0);
        let r2 = subdivision_from_index!(self, 1);
        let r3 = subdivision_from_index!(self, 2);
        let r4 = subdivision_from_index!(self, 3);
        [r1, r2, r3, r4]
    }

    /// The same as `rects` but each subdivision is yielded via the returned `Iterator`.
    pub fn rects_iter(self) -> Subdivisions<S> {
        Subdivisions {
            ranges: self,
            subdivision_index: 0,
        }
    }

    // The subdivision at the given index within the range 0..NUM_SUBDIVISIONS.
    fn subdivision_at_index(&self, index: u8) -> Option<Rect<S>> {
        let rect = match index {
            0 => subdivision_from_index!(self, 0),
            1 => subdivision_from_index!(self, 1),
            2 => subdivision_from_index!(self, 2),
            3 => subdivision_from_index!(self, 3),
            _ => return None,
        };
        Some(rect)
    }
}

impl<S> Rect<S>
where
    S: Scalar + Neg<Output = S>,
{
    /// The width of the Rect.
    pub fn w(&self) -> S {
        self.x.len()
    }

    /// The height of the Rect.
    pub fn h(&self) -> S {
        self.y.len()
    }

    /// The width and height of the Rect as a tuple.
    pub fn w_h(&self) -> (S, S) {
        (self.w(), self.h())
    }

    /// The length of the longest side of the rectangle.
    pub fn len(&self) -> S {
        math::partial_max(self.w(), self.h())
    }

    /// The left and top edges of the **Rect** along with the width and height.
    pub fn l_t_w_h(&self) -> (S, S, S, S) {
        let (w, h) = self.w_h();
        (self.left(), self.top(), w, h)
    }

    /// The left and bottom edges of the **Rect** along with the width and height.
    pub fn l_b_w_h(&self) -> (S, S, S, S) {
        let (w, h) = self.w_h();
        (self.left(), self.bottom(), w, h)
    }

    /// The Rect with some padding applied to the left edge.
    pub fn pad_left(self, pad: S) -> Self {
        Rect {
            x: self.x.pad_start(pad),
            ..self
        }
    }

    /// The Rect with some padding applied to the right edge.
    pub fn pad_right(self, pad: S) -> Self {
        Rect {
            x: self.x.pad_end(pad),
            ..self
        }
    }

    /// The rect with some padding applied to the bottom edge.
    pub fn pad_bottom(self, pad: S) -> Self {
        Rect {
            y: self.y.pad_start(pad),
            ..self
        }
    }

    /// The Rect with some padding applied to the top edge.
    pub fn pad_top(self, pad: S) -> Self {
        Rect {
            y: self.y.pad_end(pad),
            ..self
        }
    }

    /// The Rect with some padding amount applied to each edge.
    pub fn pad(self, pad: S) -> Self {
        let Rect { x, y } = self;
        Rect {
            x: x.pad(pad),
            y: y.pad(pad),
        }
    }

    /// The Rect with some padding applied.
    pub fn padding(self, padding: Padding<S>) -> Self {
        Rect {
            x: self.x.pad_ends(padding.x.start, padding.x.end),
            y: self.y.pad_ends(padding.y.start, padding.y.end),
        }
    }

    /// Returns a `Rect` with a position relative to the given position on the *x* axis.
    pub fn relative_to_x(self, x: S) -> Self {
        Rect {
            x: self.x.shift(-x),
            ..self
        }
    }

    /// Returns a `Rect` with a position relative to the given position on the *y* axis.
    pub fn relative_to_y(self, y: S) -> Self {
        Rect {
            y: self.y.shift(-y),
            ..self
        }
    }

    /// Returns a `Rect` with a position relative to the given position.
    pub fn relative_to(self, [x, y]: [S; 2]) -> Self {
        self.relative_to_x(x).relative_to_y(y)
    }

    /// Invert the x axis (aka flip *around* the y axis).
    pub fn invert_x(self) -> Self {
        Rect {
            x: self.x.invert(),
            ..self
        }
    }

    /// Invert the y axis (aka flip *around* the x axis).
    pub fn invert_y(self) -> Self {
        Rect {
            y: self.y.invert(),
            ..self
        }
    }
}

impl<S> Iterator for Subdivisions<S>
where
    S: Copy,
{
    type Item = Rect<S>;

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

impl<S> Iterator for Corners<S>
where
    S: Scalar,
{
    type Item = [S; 2];
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(corner) = self.rect.corner_at_index(self.index) {
            self.index += 1;
            return Some(corner);
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<S> DoubleEndedIterator for Corners<S>
where
    S: Scalar,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_index = self.index + 1;
        if let Some(corner) = self.rect.corner_at_index(NUM_CORNERS - next_index) {
            self.index = next_index;
            return Some(corner);
        }
        None
    }
}

impl<S> ExactSizeIterator for Corners<S>
where
    S: Scalar,
{
    fn len(&self) -> usize {
        (NUM_CORNERS - self.index) as usize
    }
}

use geom::{self, Align, Edge, Range, Tri};
use math::{self, BaseNum, Point2, Vector2};
use math::num_traits::Float;
use std::ops::Neg;

/// Defines a Rectangle's bounds across the x and y axes.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rect<S = f64> {
    /// The start and end positions of the Rectangle on the x axis.
    pub x: Range<S>,
    /// The start and end positions of the Rectangle on the y axis.
    pub y: Range<S>,
}

/// The distance between the inner edge of a border and the outer edge of the inner content.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Padding<S> {
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

impl<S> Padding<S>
where
    S: BaseNum,
{
    /// No padding.
    pub fn none() -> Self {
        Padding {
            x: Range::new(S::zero(), S::zero()),
            y: Range::new(S::zero(), S::zero()),
        }
    }
}

impl<S> Rect<S>
where
    S: BaseNum,
{
    /// Construct a Rect from a given `Point` and `Dimensions`.
    pub fn from_xy_wh(p: Point2<S>, wh: Vector2<S>) -> Self {
        Rect {
            x: Range::from_pos_and_len(p.x, wh.x),
            y: Range::from_pos_and_len(p.y, wh.y),
        }
    }

    /// Construct a Rect from the coordinates of two points.
    pub fn from_corners(a: Point2<S>, b: Point2<S>) -> Self {
        let (left, right) = if a.x < b.x { (a.x, b.x) } else { (b.x, a.x) };
        let (bottom, top) = if a.y < b.y { (a.y, b.y) } else { (b.y, a.y) };
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

    /// The Rect representing the area in which two Rects overlap.
    pub fn overlap(self, other: Self) -> Option<Self> {
        self.x.overlap(other.x).and_then(|x| {
            self.y.overlap(other.y).map(|y| Rect { x: x, y: y })
        })
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

    /// The position in the middle of the x bounds.
    pub fn x(&self) -> S {
        self.x.middle()
    }

    /// The position in the middle of the y bounds.
    pub fn y(&self) -> S {
        self.y.middle()
    }

    /// The xy position in the middle of the bounds.
    pub fn xy(&self) -> Point2<S> {
        [self.x(), self.y()].into()
    }

    /// The centered x and y coordinates as a tuple.
    pub fn x_y(&self) -> (S, S) {
        (self.x(), self.y())
    }

    /// The Rect's lowest y value.
    pub fn bottom(&self) -> S {
        self.y.undirected().start
    }

    /// The Rect's highest y value.
    pub fn top(&self) -> S {
        self.y.undirected().end
    }

    /// The Rect's lowest x value.
    pub fn left(&self) -> S {
        self.x.undirected().start
    }

    /// The Rect's highest x value.
    pub fn right(&self) -> S {
        self.x.undirected().end
    }

    /// The top left corner **Point**.
    pub fn top_left(&self) -> Point2<S> {
        [self.left(), self.top()].into()
    }

    /// The bottom left corner **Point**.
    pub fn bottom_left(&self) -> Point2<S> {
        [self.left(), self.bottom()].into()
    }

    /// The top right corner **Point**.
    pub fn top_right(&self) -> Point2<S> {
        [self.right(), self.top()].into()
    }

    /// The bottom right corner **Point**.
    pub fn bottom_right(&self) -> Point2<S> {
        [self.right(), self.bottom()].into()
    }

    /// The edges of the **Rect** in a tuple (top, bottom, left, right).
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

    /// Shift the Rect by the given Point.
    pub fn shift(self, p: Point2<S>) -> Self {
        self.shift_x(p.x).shift_y(p.y)
    }

    /// Does the given point touch the Rectangle.
    pub fn contains(&self, p: Point2<S>) -> bool {
        self.x.contains(p.x) && self.y.contains(p.y)
    }

    /// Stretches the closest edge(s) to the given point if the point lies outside of the Rect area.
    pub fn stretch_to_point(self, point: Point2<S>) -> Self {
        let Rect { x, y } = self;
        Rect {
            x: x.stretch_to_value(point[0]),
            y: y.stretch_to_value(point[1]),
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
            y: self.y.align_before(other.x),
        }
    }

    /// Align `self`'s bottom edge with the top edge of the `other` **Rect**.
    pub fn above(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_after(other.x),
        }
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

    /// Align `self`'s left edge with the left edge of the `other` **Rect**.
    pub fn align_left_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_start_of(other.x),
            y: self.y,
        }
    }

    /// Align the middle of `self` with the middle of the `other` **Rect** along the *x* axis.
    pub fn align_middle_x_of(self, other: Self) -> Self {
        Rect {
            x: self.x.align_middle_of(other.x),
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

    /// Align the middle of `self` with the middle of the `other` **Rect** along the *y* axis.
    pub fn align_middle_y_of(self, other: Self) -> Self {
        Rect {
            x: self.x,
            y: self.y.align_middle_of(other.y),
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

    /// Return the **Corner** of `self` that is closest to the given **Point**.
    pub fn closest_corner(&self, p: Point2<S>) -> Corner {
        let x_edge = self.x.closest_edge(p.x);
        let y_edge = self.y.closest_edge(p.y);
        match (x_edge, y_edge) {
            (Edge::Start, Edge::Start) => Corner::BottomLeft,
            (Edge::Start, Edge::End) => Corner::TopLeft,
            (Edge::End, Edge::Start) => Corner::BottomRight,
            (Edge::End, Edge::End) => Corner::TopRight,
        }
    }

    /// Thee four corners of the `Rect`.
    pub fn corners(&self) -> [Point2<S>; 4] {
        let (l, r, b, t) = self.l_r_b_t();
        let lb = [l, b].into();
        let lt = [l, t].into();
        let rt = [r, t].into();
        let rb = [r, b].into();
        [lb, lt, rt, rb]
    }

    /// Return two `Tri`s that represent the `Rect`.
    pub fn triangles(&self) -> (Tri<Point2<S>>, Tri<Point2<S>>) {
        let corners = self.corners();
        geom::quad::triangles(&corners)
    }
}

impl<S> Rect<S>
where
    S: BaseNum + Neg<Output = S>,
{
    /// The width of the Rect.
    pub fn w(&self) -> S {
        self.x.len()
    }

    /// The height of the Rect.
    pub fn h(&self) -> S {
        self.y.len()
    }

    /// The total dimensions of the Rect.
    pub fn wh(&self) -> Vector2<S> {
        [self.w(), self.h()].into()
    }

    /// The width and height of the Rect as a tuple.
    pub fn w_h(&self) -> (S, S) {
        (self.w(), self.h())
    }

    /// Convert the Rect to a `Point` and `Dimensions`.
    pub fn xy_wh(&self) -> (Point2<S>, Vector2<S>) {
        (self.xy(), self.wh())
    }

    /// The Rect's centered coordinates and dimensions in a tuple.
    pub fn x_y_w_h(&self) -> (S, S, S, S) {
        let (xy, wh) = self.xy_wh();
        (xy[0], xy[1], wh[0], wh[1])
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
    pub fn relative_to(self, p: Point2<S>) -> Self {
        self.relative_to_x(p.x).relative_to_y(p.y)
    }
}

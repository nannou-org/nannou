use geom::{Rect, Tri};
use math::{self, BaseNum, Point2};
use math::num_traits::{Float, NumCast};
use std;
use std::ops::Neg;

/// An iterator yielding the edges of an ellipse (or some section of an `ellipse`) as a series of
/// points.
#[derive(Clone)]
#[allow(missing_copy_implementations)]
pub struct Circumference<S> {
    index: usize,
    num_points: usize,
    point: Point2<S>,
    rad_step: S,
    rad_offset: S,
    half_w: S,
    half_h: S,
}

/// An iterator yielding triangles that describe an oval or some section of an oval.
#[derive(Clone)]
pub struct Triangles<S> {
    // The last circumference point yielded by the `CircumferenceOffset` iterator.
    last: Point2<S>,
    // The circumference points used to yield yielded by the `CircumferenceOffset` iterator.
    points: Circumference<S>,
}

impl<S> Circumference<S>
where
    S: BaseNum + Neg<Output=S>,
{
    fn new_inner(rect: Rect<S>, num_points: usize, rad_step: S) -> Self {
        let (x, y, w, h) = rect.x_y_w_h();
        let two = math::two();
        Circumference {
            index: 0,
            num_points: num_points,
            point: [x, y].into(),
            half_w: w / two,
            half_h: h / two,
            rad_step: rad_step,
            rad_offset: S::zero(),
        }
    }

    /// An iterator yielding the ellipse's edges as a circumference represented as a series of
    /// points.
    ///
    /// `resolution` is clamped to a minimum of `1` as to avoid creating a `Circumference` that
    /// produces `NaN` values.
    pub fn new(rect: Rect<S>, mut resolution: usize) -> Self {
        resolution = std::cmp::max(resolution, 1);
        use std::f64::consts::PI;
        let radians = S::from(2.0 * PI).unwrap();
        Self::new_section(rect, resolution, radians)
    }

    /// Produces a new iterator that yields only a section of the ellipse's circumference, where
    /// the section is described via its angle in radians.
    ///
    /// `resolution` is clamped to a minimum of `1` as to avoid creating a `Circumference` that
    /// produces `NaN` values.
    pub fn new_section(rect: Rect<S>, resolution: usize, radians: S) -> Self
    where
        S: BaseNum,
    {
        let res = S::from(resolution).unwrap();
        Self::new_inner(rect, resolution + 1, radians / res)
    }

    /// Produces a new iterator that yields only a section of the ellipse's circumference, where
    /// the section is described via its angle in radians.
    pub fn section(mut self, radians: S) -> Self {
        let resolution = self.num_points - 1;
        let res = S::from(resolution).unwrap();
        self.rad_step = radians / res;
        self
    }

    /// Rotates the position at which the iterator starts yielding points by the given radians.
    ///
    /// This is particularly useful for yielding a different section of the circumference when
    /// using `circumference_section`
    pub fn offset_radians(mut self, radians: S) -> Self {
        self.rad_offset = radians;
        self
    }

    /// Produces an `Iterator` yielding `Triangle`s.
    ///
    /// Triangles are created by joining each edge yielded by the inner `Circumference` to the
    /// middle of the ellipse.
    pub fn triangles(mut self) -> Triangles<S>
    where
        S: Float,
    {
        let last = self.next().unwrap_or(self.point);
        Triangles { last, points: self }
    }
}

impl<S> Iterator for Circumference<S>
where
    S: BaseNum + Float,
{
    type Item = Point2<S>;
    fn next(&mut self) -> Option<Self::Item> {
        let Circumference {
            ref mut index,
            num_points,
            point,
            rad_step,
            rad_offset,
            half_w,
            half_h,
        } = *self;
        if *index >= num_points {
            return None;
        }
        let index_s: S = NumCast::from(*index).unwrap();
        let x = point.x + half_w * (rad_offset + rad_step * index_s).cos();
        let y = point.y + half_h * (rad_offset + rad_step * index_s).sin();
        *index += 1;
        Some([x, y].into())
    }
}

impl<S> Iterator for Triangles<S>
where
    S: BaseNum + Float,
{
    type Item = Tri<Point2<S>>;
    fn next(&mut self) -> Option<Self::Item> {
        let Triangles { ref mut points, ref mut last } = *self;
        points.next().map(|next| {
            let triangle = Tri([points.point, *last, next]);
            *last = next;
            triangle
        })
    }
}

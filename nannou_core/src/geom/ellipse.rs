use crate::geom::{
    self,
    scalar::{self, Scalar},
    Rect, Tri,
};
use crate::math::{
    self,
    num_traits::{Float, NumCast},
};

/// Scalar types compatible with ellipses.
pub trait EllipseScalar: Float + Scalar {
    /// 2 * PI.
    const TAU: Self;
}

/// A simple ellipse type with helper methods around the `ellipse` module's functions.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Ellipse<S = scalar::Default> {
    /// The width and height off the `Ellipse`.
    pub rect: Rect<S>,
    /// The resolution (number of sides) of the `Ellipse`.
    pub resolution: S,
}

/// A subsection of an `Ellipse`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Section<S = scalar::Default> {
    /// The ellipse from which this section is produced.
    pub ellipse: Ellipse<S>,
    /// The angle in radians of the start of the section.
    pub offset_radians: S,
    /// The section of the circumference in radians.
    pub section_radians: S,
}

/// An iterator yielding the edges of an ellipse (or some section of an `ellipse`) as a series of
/// points.
#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct Circumference<S = scalar::Default> {
    index: S,
    num_points: S,
    middle: [S; 2],
    rad_step: S,
    rad_offset: S,
    half_w: S,
    half_h: S,
}

/// All vertices necessary for yielding the triangles that make up the centre.
#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct TriangleVertices<S = scalar::Default> {
    middle: Option<[S; 2]>,
    circumference: Circumference<S>,
}

/// An iterator yielding an index for every vertex in every triangle in the ellipse.
#[derive(Clone, Debug)]
#[allow(missing_copy_implementations)]
pub struct TriangleIndices {
    yield_middle: bool,
    left: Option<usize>,
    right: Option<usize>,
    total: usize,
}

/// An iterator yielding triangles that describe an oval or some section of an oval.
#[derive(Clone, Debug)]
pub struct Triangles<S = scalar::Default> {
    // The last circumference point yielded by the `CircumferenceOffset` iterator.
    last: [S; 2],
    // The circumference points used to yield yielded by the `CircumferenceOffset` iterator.
    points: Circumference<S>,
}

impl<S> Ellipse<S>
where
    S: EllipseScalar,
{
    /// Construct a new ellipse from its bounding rect and resolution (number of sides).
    pub fn new(rect: Rect<S>, resolution: S) -> Self {
        Ellipse { rect, resolution }
    }

    /// A section of the `Ellipse`.
    ///
    /// `offset_radians` describes the angle at which the offset begins.
    ///
    /// `section_radians` describes how large the section is as an angle.
    pub fn section(self, offset_radians: S, section_radians: S) -> Section<S> {
        Section {
            ellipse: self,
            offset_radians,
            section_radians,
        }
    }

    /// Produces an iterator yielding the points of the ellipse circumference.
    pub fn circumference(self) -> Circumference<S> {
        let Ellipse { rect, resolution } = self;
        Circumference::new(rect, resolution)
    }

    /// Produces an iterator yielding the triangles that describe the ellipse.
    ///
    /// TODO: Describe the order.
    pub fn triangles(self) -> Triangles<S> {
        self.circumference().triangles()
    }

    /// The same as **Triangles<S>**, but produces the indices of each triangle into the returned
    /// **TriangleVertices** iterator rather than the vertices for each corner.
    pub fn triangle_indices(&self) -> (TriangleVertices<S>, TriangleIndices) {
        self.circumference().triangle_indices()
    }
}

impl<S> Section<S>
where
    S: EllipseScalar,
{
    /// Produces an iterator yielding the points of the ellipse circumference.
    pub fn circumference(self) -> Circumference<S> {
        let Section {
            ellipse,
            offset_radians,
            section_radians,
        } = self;
        let circ = Circumference::new_section(ellipse.rect, ellipse.resolution, section_radians);
        circ.offset_radians(offset_radians)
    }

    /// Produces an iterator yielding the triangles that describe the ellipse section.
    ///
    /// TODO: Describe the order.
    pub fn triangles(self) -> Triangles<S> {
        self.circumference().triangles()
    }

    /// The same as **Triangles<S>**, but produces the indices of each triangle into the returned
    /// **TriangleVertices** iterator rather than the vertices for each corner.
    pub fn triangle_indices(&self) -> (TriangleVertices<S>, TriangleIndices) {
        self.circumference().triangle_indices()
    }
}

impl<S> Circumference<S>
where
    S: EllipseScalar,
{
    fn new_inner(rect: Rect<S>, num_points: S, rad_step: S) -> Self {
        let (x, y, w, h) = rect.x_y_w_h();
        let two = math::two();
        Circumference {
            index: S::zero(),
            num_points: num_points,
            middle: [x, y].into(),
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
    pub fn new(rect: Rect<S>, mut resolution: S) -> Self {
        resolution = crate::math::partial_max(resolution, S::one());
        Self::new_section(rect, resolution, S::TAU)
    }

    /// Produces a new iterator that yields only a section of the ellipse's circumference, where
    /// the section is described via its angle in radians.
    ///
    /// `resolution` is clamped to a minimum of `1` as to avoid creating a `Circumference` that
    /// produces `NaN` values.
    pub fn new_section(rect: Rect<S>, resolution: S, radians: S) -> Self {
        Self::new_inner(rect, resolution + S::one(), radians / resolution)
    }

    /// Produces a new iterator that yields only a section of the ellipse's circumference, where
    /// the section is described via its angle in radians.
    pub fn section(mut self, radians: S) -> Self {
        let resolution = self.num_points - S::one();
        self.rad_step = radians / resolution;
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
    pub fn triangles(mut self) -> Triangles<S> {
        let last = self.next().unwrap_or(self.middle);
        Triangles { last, points: self }
    }

    /// The same as **Triangles<S>**, but produces the indices of each triangle into the returned
    /// **TriangleVertices** iterator rather than the vertices for each corner.
    pub fn triangle_indices(self) -> (TriangleVertices<S>, TriangleIndices) {
        let middle = Some(self.middle);
        let num_vertices = self.len();
        let circumference = self;
        let vertices = TriangleVertices {
            middle,
            circumference,
        };
        let indices = TriangleIndices {
            yield_middle: true,
            left: Some(1),
            right: Some(2),
            total: num_vertices,
        };
        (vertices, indices)
    }
}

impl EllipseScalar for f32 {
    const TAU: Self = core::f32::consts::TAU;
}

impl EllipseScalar for f64 {
    const TAU: Self = core::f64::consts::TAU;
}

impl<S> Iterator for Circumference<S>
where
    S: EllipseScalar,
{
    type Item = [S; 2];
    fn next(&mut self) -> Option<Self::Item> {
        let Circumference {
            ref mut index,
            num_points,
            middle: [mx, my],
            rad_step,
            rad_offset,
            half_w,
            half_h,
        } = *self;
        if *index >= num_points {
            return None;
        }
        let x = mx + half_w * (rad_offset + rad_step * *index).cos();
        let y = my + half_h * (rad_offset + rad_step * *index).sin();
        *index += S::one();
        Some([x, y].into())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

// TODO:
// impl<S> DoubleEndedIterator for Circumference<S>
// where
//     S: Scalar,
// {
// }

impl<S> ExactSizeIterator for Circumference<S>
where
    S: EllipseScalar + NumCast,
{
    fn len(&self) -> usize {
        NumCast::from(self.num_points - self.index).unwrap()
    }
}

impl<S> Iterator for TriangleVertices<S>
where
    S: EllipseScalar,
{
    type Item = [S; 2];
    fn next(&mut self) -> Option<Self::Item> {
        self.middle.take().or_else(|| self.circumference.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<S> ExactSizeIterator for TriangleVertices<S>
where
    S: EllipseScalar,
{
    fn len(&self) -> usize {
        (if self.middle.is_some() { 1 } else { 0 }) + self.circumference.len()
    }
}

impl Iterator for TriangleIndices {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.yield_middle {
            self.yield_middle = false;
            Some(0)
        } else if let Some(left) = self.left.take() {
            Some(left)
        } else if let Some(right) = self.right.take() {
            // Check if we're done. If not, step the left and right indices.
            if right < self.total {
                self.yield_middle = true;
                self.left = Some(right);
                self.right = Some(right + 1);
            }
            Some(right)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for TriangleIndices {
    fn len(&self) -> usize {
        if let Some(right) = self.right {
            let n_tris = self.total - right;
            let remaining_middle = if self.yield_middle { 1 } else { 0 };
            let remaining_left = if self.left.is_some() { 1 } else { 0 };
            let remaining_right = 1;
            n_tris * geom::tri::NUM_VERTICES as usize
                + remaining_middle
                + remaining_left
                + remaining_right
        } else {
            0
        }
    }
}

impl<S> Iterator for Triangles<S>
where
    S: EllipseScalar,
{
    type Item = Tri<[S; 2]>;
    fn next(&mut self) -> Option<Self::Item> {
        let Triangles {
            ref mut points,
            ref mut last,
        } = *self;
        points.next().map(|next| {
            let triangle = Tri([points.middle, *last, next]);
            *last = next;
            triangle
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<S> ExactSizeIterator for Triangles<S>
where
    S: EllipseScalar,
{
    fn len(&self) -> usize {
        self.points.len()
    }
}

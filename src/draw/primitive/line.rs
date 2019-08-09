use crate::color::LinSrgba;
use crate::draw::primitive::{PathStroke, Primitive};
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{
    ColorScalar, Draw, Drawn, IntoDrawn, SetColor, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Drawing};
use crate::geom::{self, pt2, Point2};
use crate::math::BaseFloat;
use lyon::tessellation::StrokeOptions;

/// A path containing only two points - a start and end.
///
/// The usage of this type is almost identical to `PathStroke` but provides `start`, `end` and
/// `points(a, b)` methods.
#[derive(Clone, Debug)]
pub struct Line<S = geom::scalar::Default> {
    path: PathStroke<S>,
    start: Option<Point2<S>>,
    end: Option<Point2<S>>,
}

/// The drawing context for a line.
pub type DrawingLine<'a, S = geom::scalar::Default> = Drawing<'a, Line<S>, S>;

impl<S> Line<S> {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_path(|p| p.stroke_weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_path(|p| p.stroke_tolerance(tolerance))
    }

    /// Specify the start point of the line.
    pub fn start(mut self, start: Point2<S>) -> Self {
        self.start = Some(start);
        self
    }

    /// Specify the end point of the line.
    pub fn end(mut self, end: Point2<S>) -> Self {
        self.end = Some(end);
        self
    }

    /// Specify the start and end points of the line.
    pub fn points(self, start: Point2<S>, end: Point2<S>) -> Self {
        self.start(start).end(end)
    }

    // Map the inner `PathStroke<S>` using the given function.
    fn map_path<F>(self, map: F) -> Self
    where
        F: FnOnce(PathStroke<S>) -> PathStroke<S>,
    {
        let Line { path, start, end } = self;
        let path = map(path);
        Line { path, start, end }
    }
}

impl<'a, S> DrawingLine<'a, S>
where
    S: BaseFloat,
{
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_ty(|ty| ty.weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.tolerance(tolerance))
    }

    /// Specify the start point of the line.
    pub fn start(self, start: Point2<S>) -> Self {
        self.map_ty(|ty| ty.start(start))
    }

    /// Specify the end point of the line.
    pub fn end(self, end: Point2<S>) -> Self {
        self.map_ty(|ty| ty.end(end))
    }

    /// Specify the start and end points of the line.
    pub fn points(self, start: Point2<S>, end: Point2<S>) -> Self {
        self.map_ty(|ty| ty.points(start, end))
    }
}

impl<S> SetStroke for Line<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.path)
    }
}

impl<S> SetOrientation<S> for Line<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.path)
    }
}

impl<S> SetPosition<S> for Line<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.path)
    }
}

impl<S> SetColor<ColorScalar> for Line<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.path)
    }
}

impl<S> From<Line<S>> for Primitive<S> {
    fn from(prim: Line<S>) -> Self {
        Primitive::Line(prim)
    }
}

impl<S> Into<Option<Line<S>>> for Primitive<S> {
    fn into(self) -> Option<Line<S>> {
        match self {
            Primitive::Line(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> IntoDrawn<S> for Line<S>
where
    S: BaseFloat,
{
    type Vertices = draw::properties::VerticesFromRanges;
    type Indices = draw::properties::IndicesFromRange;

    fn into_drawn(self, mut draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Line { path, start, end } = self;
        let start = start.unwrap_or_else(|| pt2(S::zero(), S::zero()));
        let end = end.unwrap_or_else(|| pt2(S::zero(), S::zero()));
        let points = [start, end];
        let path = draw.drawing_context(|ctxt| path.points(ctxt, points.iter().cloned()));
        path.into_drawn(draw)
    }
}

impl<S> Default for Line<S> {
    fn default() -> Self {
        Line {
            path: Default::default(),
            start: Default::default(),
            end: Default::default(),
        }
    }
}

use crate::color::LinSrgba;
use crate::draw::primitive::path;
use crate::draw::primitive::{PathStroke, Primitive};
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{ColorScalar, SetColor, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};
use crate::geom::{self, pt2, Point2};
use crate::math::{BaseFloat, Zero};
use lyon::tessellation::StrokeOptions;

/// A path containing only two points - a start and end.
///
/// The usage of this type is almost identical to `PathStroke` but provides `start`, `end` and
/// `points(a, b)` methods.
#[derive(Clone, Debug)]
pub struct Line<S = geom::scalar::Default> {
    pub path: PathStroke<S>,
    pub start: Option<Point2<S>>,
    pub end: Option<Point2<S>>,
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

impl<'l, S> From<Line<S>> for Primitive<'l, S> {
    fn from(prim: Line<S>) -> Self {
        Primitive::Line(prim)
    }
}

impl<'l, S> Into<Option<Line<S>>> for Primitive<'l, S> {
    fn into(self) -> Option<Line<S>> {
        match self {
            Primitive::Line(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<'l> draw::renderer::RenderPrimitive<'l> for Line<f32> {
    fn render_primitive(
        self,
        mut ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender<'l> {
        let Line { path, start, end } = self;
        let start = start.unwrap_or(pt2(0.0, 0.0));
        let end = end.unwrap_or(pt2(0.0, 0.0));
        if start == end {
            return draw::renderer::PrimitiveRender::default();
        }
        let close = false;
        let points = [start, end];
        let points = points.iter().cloned().map(Into::into);
        let events = lyon::path::iterator::FromPolyline::new(close, points);

        // Determine the transform to apply to all points.
        let global_transform = ctxt.transform;
        let local_transform = path.position.transform() * path.orientation.transform();
        let transform = global_transform * local_transform;

        path::render_path_events(
            events,
            path.color,
            transform,
            path::Options::Stroke(path.opts),
            &ctxt.theme,
            &draw::theme::Primitive::Line,
            &mut ctxt.fill_tessellator,
            &mut ctxt.stroke_tessellator,
            mesh,
        );

        draw::renderer::PrimitiveRender::default()
    }
}

impl<S> Default for Line<S>
where
    S: Zero,
{
    fn default() -> Self {
        Line {
            path: Default::default(),
            start: Default::default(),
            end: Default::default(),
        }
    }
}

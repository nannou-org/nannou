use crate::color::LinSrgba;
use crate::draw::primitive::path;
use crate::draw::primitive::{PathStroke, Primitive};
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{ColorScalar, SetColor, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};
use crate::geom::{pt2, Point2};
use lyon::tessellation::StrokeOptions;

/// A path containing only two points - a start and end.
///
/// The usage of this type is almost identical to `PathStroke` but provides `start`, `end` and
/// `points(a, b)` methods.
#[derive(Clone, Debug, Default)]
pub struct Line {
    pub path: PathStroke,
    pub start: Option<Point2>,
    pub end: Option<Point2>,
}

/// The drawing context for a line.
pub type DrawingLine<'a> = Drawing<'a, Line>;

impl Line {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_path(|p| p.stroke_weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_path(|p| p.stroke_tolerance(tolerance))
    }

    /// Specify the start point of the line.
    pub fn start(mut self, start: Point2) -> Self {
        self.start = Some(start);
        self
    }

    /// Specify the end point of the line.
    pub fn end(mut self, end: Point2) -> Self {
        self.end = Some(end);
        self
    }

    /// Specify the start and end points of the line.
    pub fn points(self, start: Point2, end: Point2) -> Self {
        self.start(start).end(end)
    }

    // Map the inner `PathStroke<S>` using the given function.
    fn map_path<F>(self, map: F) -> Self
    where
        F: FnOnce(PathStroke) -> PathStroke,
    {
        let Line { path, start, end } = self;
        let path = map(path);
        Line { path, start, end }
    }
}

impl<'a> DrawingLine<'a> {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_ty(|ty| ty.weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.tolerance(tolerance))
    }

    /// Specify the start point of the line.
    pub fn start(self, start: Point2) -> Self {
        self.map_ty(|ty| ty.start(start))
    }

    /// Specify the end point of the line.
    pub fn end(self, end: Point2) -> Self {
        self.map_ty(|ty| ty.end(end))
    }

    /// Specify the start and end points of the line.
    pub fn points(self, start: Point2, end: Point2) -> Self {
        self.map_ty(|ty| ty.points(start, end))
    }
}

impl SetStroke for Line {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.path)
    }
}

impl SetOrientation for Line {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.path)
    }
}

impl SetPosition for Line {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.path)
    }
}

impl SetColor<ColorScalar> for Line {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.path)
    }
}

impl From<Line> for Primitive {
    fn from(prim: Line) -> Self {
        Primitive::Line(prim)
    }
}

impl Into<Option<Line>> for Primitive {
    fn into(self) -> Option<Line> {
        match self {
            Primitive::Line(prim) => Some(prim),
            _ => None,
        }
    }
}

impl draw::renderer::RenderPrimitive for Line {
    fn render_primitive<R>(
        self,
        _ctxt: draw::renderer::RenderContext,
        mut renderer: R,
    ) -> draw::renderer::PrimitiveRender
    where
        R: draw::renderer::PrimitiveRenderer,
    {
        let Line { path, start, end } = self;
        let start = start.unwrap_or(pt2(0.0, 0.0));
        let end = end.unwrap_or(pt2(0.0, 0.0));
        if start == end {
            return draw::renderer::PrimitiveRender::default();
        }
        let close = false;
        let points = [start, end];
        let points = points.iter().cloned().map(|p| p.to_array().into());
        let events = lyon::path::iterator::FromPolyline::new(close, points);

        let local_transform = path.position.transform() * path.orientation.transform();

        renderer.path_flat_color(
            local_transform,
            events,
            path.color,
            draw::theme::Primitive::Line,
            path::Options::Stroke(path.opts),
        );

        draw::renderer::PrimitiveRender::default()
    }
}

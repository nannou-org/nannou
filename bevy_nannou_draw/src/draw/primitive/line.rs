use crate::draw::primitive::path;
use crate::draw::primitive::{PathStroke, Primitive};
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{SetColor, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};
use bevy::prelude::*;
use lyon::tessellation::StrokeOptions;

/// A path containing only two points - a start and end.
///
/// The usage of this type is almost identical to `PathStroke` but provides `start`, `end` and
/// `points(a, b)` methods.
#[derive(Clone, Debug, Default)]
pub struct Line<M: Material> {
    pub path: PathStroke<M>,
    pub start: Option<Vec2>,
    pub end: Option<Vec2>,
    pub material: M,
}

/// The drawing context for a line.
pub type DrawingLine<'a, M: Material> = Drawing<'a, Line<M>>;

impl <M: Material> Line<M> {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_path(|p| p.stroke_weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_path(|p| p.stroke_tolerance(tolerance))
    }

    /// Specify the start point of the line.
    pub fn start(mut self, start: Vec2) -> Self {
        self.start = Some(start);
        self
    }

    /// Specify the end point of the line.
    pub fn end(mut self, end: Vec2) -> Self {
        self.end = Some(end);
        self
    }

    /// Specify the start and end points of the line.
    pub fn points(self, start: Vec2, end: Vec2) -> Self {
        self.start(start).end(end)
    }

    // Map the inner `PathStroke<S>` using the given function.
    fn map_path<F>(self, map: F) -> Self
    where
        F: FnOnce(PathStroke<M>) -> PathStroke<M>,
    {
        let Line { path, start, end, material } = self;
        let path = map(path);
        Line { path, start, end, material }
    }
}

impl<'a, M: Material> DrawingLine<'a, M> {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_ty(|ty| ty.weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.tolerance(tolerance))
    }

    /// Specify the start point of the line.
    pub fn start(self, start: Vec2) -> Self {
        self.map_ty(|ty| ty.start(start))
    }

    /// Specify the end point of the line.
    pub fn end(self, end: Vec2) -> Self {
        self.map_ty(|ty| ty.end(end))
    }

    /// Specify the start and end points of the line.
    pub fn points(self, start: Vec2, end: Vec2) -> Self {
        self.map_ty(|ty| ty.points(start, end))
    }
}

impl <M: Material> SetStroke for Line<M> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.path)
    }
}

impl <M: Material> SetOrientation for Line<M> {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.path)
    }
}

impl <M: Material> SetPosition for Line<M> {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.path)
    }
}

impl <M: Material> SetColor for Line<M> {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.path)
    }
}

impl <M: Material> From<Line<M>> for Primitive {
    fn from(prim: Line<M>) -> Self {
        Primitive::Line(prim)
    }
}

impl <M: Material> Into<Option<Line<M>>> for Primitive {
    fn into(self) -> Option<Line<M>> {
        match self {
            Primitive::Line(prim) => Some(prim),
            _ => None,
        }
    }
}

impl <M: Material> draw::render::RenderPrimitive for Line<M> {
    fn render_primitive(
        self,
        mut ctxt: draw::render::RenderContext,
        mesh: &mut Mesh,
    ) -> draw::render::PrimitiveRender {
        let Line { path, start, end } = self;
        let start = start.unwrap_or(Vec2::new(0.0, 0.0));
        let end = end.unwrap_or(Vec2::new(0.0, 0.0));
        if start == end {
            return draw::render::PrimitiveRender::default();
        }
        let close = false;
        let points = [start, end];
        let points = points.iter().cloned().map(|p| p.to_array().into());
        let events = lyon::path::iterator::FromPolyline::new(close, points);

        // Determine the transform to apply to all points.
        let global_transform = *ctxt.transform;
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

        draw::render::PrimitiveRender::default()
    }
}

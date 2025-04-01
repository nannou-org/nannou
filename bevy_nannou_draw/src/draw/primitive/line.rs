use bevy::prelude::*;
use lyon::tessellation::StrokeOptions;

use crate::{
    draw::{
        self,
        primitive::{path, PathStroke, Primitive},
        properties::{
            spatial::{orientation, position},
            SetColor, SetOrientation, SetPosition, SetStroke,
        },
        Drawing,
    },
    render::ShaderModel,
};

/// A path containing only two points - a start and end.
///
/// The usage of this type is almost identical to `PathStroke` but provides `start`, `end` and
/// `points(a, b)` methods.
#[derive(Clone, Debug, Default)]
pub struct Line {
    pub path: PathStroke,
    pub start: Option<Vec2>,
    pub end: Option<Vec2>,
}

/// The drawing context for a line.
pub type DrawingLine<'a, SM> = Drawing<'a, Line, SM>;

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
        F: FnOnce(PathStroke) -> PathStroke,
    {
        let Line { path, start, end } = self;
        let path = map(path);
        Line { path, start, end }
    }
}

impl<'a, SM> DrawingLine<'a, SM>
where
    SM: ShaderModel + Default,
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

impl SetColor for Line {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.path)
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

impl draw::render::RenderPrimitive for Line {
    fn render_primitive(self, mut ctxt: draw::render::RenderContext, mesh: &mut Mesh) {
        let Line { path, start, end } = self;
        let start = start.unwrap_or(Vec2::new(0.0, 0.0));
        let end = end.unwrap_or(Vec2::new(0.0, 0.0));
        if start == end {
            return;
        }
        let close = false;
        let points = [start, end];
        let tex_coords = [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)];
        let points = points.iter().cloned().zip(tex_coords.iter().copied());

        // Determine the transform to apply to all points.
        let global_transform = *ctxt.transform;
        let local_transform = path.position.transform() * path.orientation.transform();
        let transform = global_transform * local_transform;

        path::render_path_points_themed(
            points,
            close,
            path.color,
            transform,
            path::Options::Stroke(path.opts),
            &ctxt.theme,
            &draw::theme::Primitive::Line,
            &mut ctxt.fill_tessellator,
            &mut ctxt.stroke_tessellator,
            mesh,
        );
    }
}

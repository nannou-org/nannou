use bevy::prelude::*;
use lyon::tessellation::StrokeOptions;

use crate::draw::primitive::path;
use crate::draw::primitive::Line;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{SetColor, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};
use crate::render::ShaderModel;

/// A path containing only two points - a start and end.
///
/// A triangle is drawn on the end to indicate direction.
#[derive(Clone, Debug)]
pub struct Arrow {
    line: Line,
    head_length: Option<f32>,
    head_width: Option<f32>,
}

/// The drawing context for a line.
pub type DrawingArrow<'a, SM> = Drawing<'a, Arrow, SM>;

impl Arrow {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_line(|l| l.weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_line(|l| l.tolerance(tolerance))
    }

    /// Specify the start point of the arrow.
    pub fn start(self, start: Vec2) -> Self {
        self.map_line(|l| l.start(start))
    }

    /// Specify the end point of the arrow.
    pub fn end(self, end: Vec2) -> Self {
        self.map_line(|l| l.end(end))
    }

    /// Specify the start and end points of the arrow.
    pub fn points(self, start: Vec2, end: Vec2) -> Self {
        self.map_line(|l| l.points(start, end))
    }

    /// The length of the arrow head.
    ///
    /// By default, this is equal to `weight * 4.0`.
    ///
    /// This value will be clamped to the length of the line itself.
    pub fn head_length(mut self, length: f32) -> Self {
        self.head_length = Some(length);
        self
    }

    /// The width of the arrow head.
    ///
    /// By default, this is equal to `weight * 2.0`.
    pub fn head_width(mut self, width: f32) -> Self {
        self.head_width = Some(width);
        self
    }

    // Map the inner `PathStroke<S>` using the given function.
    fn map_line<F>(self, map: F) -> Self
    where
        F: FnOnce(Line) -> Line,
    {
        let Arrow {
            line,
            head_length,
            head_width,
        } = self;
        let line = map(line);
        Arrow {
            line,
            head_length,
            head_width,
        }
    }
}

impl<'a, SM> DrawingArrow<'a, SM>
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

    /// Specify the start point of the arrow.
    pub fn start(self, start: Vec2) -> Self {
        self.map_ty(|ty| ty.start(start))
    }

    /// Specify the end point of the arrow.
    pub fn end(self, end: Vec2) -> Self {
        self.map_ty(|ty| ty.end(end))
    }

    /// Specify the start and end points of the arrow.
    pub fn points(self, start: Vec2, end: Vec2) -> Self {
        self.map_ty(|ty| ty.points(start, end))
    }

    /// The length of the arrow head.
    ///
    /// By default, this is equal to `weight * 4.0`.
    ///
    /// This value will be clamped to the length of the line itself.
    pub fn head_length(self, length: f32) -> Self {
        self.map_ty(|ty| ty.head_length(length))
    }

    /// The width of the arrow head.
    ///
    /// By default, this is equal to `weight * 2.0`.
    pub fn head_width(self, width: f32) -> Self {
        self.map_ty(|ty| ty.head_width(width))
    }
}

impl SetStroke for Arrow {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.line)
    }
}

impl SetOrientation for Arrow {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.line)
    }
}

impl SetPosition for Arrow {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.line)
    }
}

impl SetColor for Arrow {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.line)
    }
}

impl From<Arrow> for Primitive {
    fn from(prim: Arrow) -> Self {
        Primitive::Arrow(prim)
    }
}

impl Into<Option<Arrow>> for Primitive {
    fn into(self) -> Option<Arrow> {
        match self {
            Primitive::Arrow(prim) => Some(prim),
            _ => None,
        }
    }
}

impl draw::render::RenderPrimitive for Arrow {
    fn render_primitive(self, mut ctxt: draw::render::RenderContext, mesh: &mut Mesh) {
        let Arrow {
            line,
            head_length,
            head_width,
        } = self;
        let start = line.start.unwrap_or(Vec2::new(0.0, 0.0));
        let end = line.end.unwrap_or(Vec2::new(0.0, 0.0));
        if start == end {
            return;
        }

        // Calculate the arrow head points.
        let line_w_2 = line.path.opts.line_width * 2.0;
        let line_w_4 = line_w_2 * 2.0;
        let head_width = head_width.unwrap_or(line_w_2);
        let head_length = head_length.unwrap_or(line_w_4);
        let line_dir = end - start;
        let line_dir_len = line_dir.length();
        let tri_len = head_length.min(line_dir_len);
        let tri_dir_norm = line_dir.normalize() * tri_len;
        let tri_start = end - tri_dir_norm;
        let tri_end = end;
        let line_start = start;
        let line_end = tri_start;
        let tri_a = tri_end;
        let tri_w_dir = Vec2::new(-tri_dir_norm.y, tri_dir_norm.x).normalize() * head_width;
        let tri_b = tri_start + tri_w_dir;
        let tri_c = tri_start - tri_w_dir;
        // The line should only be drawn if there is space after drawing the triangle.
        let draw_line = line_dir_len > tri_len;

        // Determine the transform to apply to all points.
        let global_transform = *ctxt.transform;
        let local_transform = line.path.position.transform() * line.path.orientation.transform();
        let transform = global_transform * local_transform;

        // Draw the tri.
        let tri_points = [tri_a, tri_b, tri_c];
        let tri_tex_coords = [
            Vec2::new(0.5, 1.0), // Tip of the arrowhead
            Vec2::new(0.0, 0.0), // Left corner
            Vec2::new(1.0, 0.0), // Right corner
        ];
        let tri_points = tri_points
            .iter()
            .cloned()
            .zip(tri_tex_coords.iter().copied());
        let close_tri = true;
        path::render_path_points_themed(
            tri_points,
            close_tri,
            line.path.color,
            transform,
            path::Options::Fill(Default::default()),
            &ctxt.theme,
            &draw::theme::Primitive::Arrow,
            &mut ctxt.fill_tessellator,
            &mut ctxt.stroke_tessellator,
            mesh,
        );

        // Draw the line.
        if draw_line {
            let line_points = [line_start, line_end];
            let line_tex_coords = [Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)];
            let line_points = line_points
                .iter()
                .cloned()
                .zip(line_tex_coords.iter().copied());
            let close_line = false;
            path::render_path_points_themed(
                line_points,
                close_line,
                line.path.color,
                transform,
                path::Options::Stroke(line.path.opts),
                &ctxt.theme,
                &draw::theme::Primitive::Arrow,
                &mut ctxt.fill_tessellator,
                &mut ctxt.stroke_tessellator,
                mesh,
            );
        }
    }
}

impl Default for Arrow {
    fn default() -> Self {
        let line = Default::default();
        let head_length = Default::default();
        let head_width = Default::default();
        Arrow {
            line,
            head_length,
            head_width,
        }
    }
}

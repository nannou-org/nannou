use crate::color::LinSrgba;
use crate::draw::primitive::path;
use crate::draw::primitive::Line;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{ColorScalar, SetColor, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};
use crate::geom::{pt2, Point2};
use crate::glam::vec2;
use lyon::tessellation::StrokeOptions;

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
pub type DrawingArrow<'a> = Drawing<'a, Arrow>;

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
    pub fn start(self, start: Point2) -> Self {
        self.map_line(|l| l.start(start))
    }

    /// Specify the end point of the arrow.
    pub fn end(self, end: Point2) -> Self {
        self.map_line(|l| l.end(end))
    }

    /// Specify the start and end points of the arrow.
    pub fn points(self, start: Point2, end: Point2) -> Self {
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

impl<'a> DrawingArrow<'a> {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_ty(|ty| ty.weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.tolerance(tolerance))
    }

    /// Specify the start point of the arrow.
    pub fn start(self, start: Point2) -> Self {
        self.map_ty(|ty| ty.start(start))
    }

    /// Specify the end point of the arrow.
    pub fn end(self, end: Point2) -> Self {
        self.map_ty(|ty| ty.end(end))
    }

    /// Specify the start and end points of the arrow.
    pub fn points(self, start: Point2, end: Point2) -> Self {
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

impl SetColor<ColorScalar> for Arrow {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.line)
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

impl draw::renderer::RenderPrimitive2 for Arrow {
    fn render_primitive<R>(
        self,
        _ctxt: draw::renderer::RenderContext2,
        mut renderer: R,
    ) -> draw::renderer::PrimitiveRender
    where
        R: draw::renderer::PrimitiveRenderer,
    {
        let Arrow {
            line,
            head_length,
            head_width,
        } = self;
        let start = line.start.unwrap_or(pt2(0.0, 0.0));
        let end = line.end.unwrap_or(pt2(0.0, 0.0));
        if start == end {
            return draw::renderer::PrimitiveRender::default();
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
        let tri_w_dir = vec2(-tri_dir_norm.y, tri_dir_norm.x).normalize() * head_width;
        let tri_b = tri_start + tri_w_dir;
        let tri_c = tri_start - tri_w_dir;
        // The line should only be drawn if there is space after drawing the triangle.
        let draw_line = line_dir_len > tri_len;

        let theme_primitive = draw::theme::Primitive::Arrow;
        let local_transform = line.path.position.transform() * line.path.orientation.transform();

        // Draw the tri.
        let tri_points = [tri_a, tri_b, tri_c];
        let tri_points = tri_points.iter().cloned().map(|p| p.to_array().into());
        let close_tri = true;
        let tri_events = lyon::path::iterator::FromPolyline::new(close_tri, tri_points);
        renderer.path_flat_color(
            local_transform,
            tri_events,
            line.path.color,
            theme_primitive,
            path::Options::Fill(Default::default()),
        );

        // Draw the line.
        if draw_line {
            let line_points = [line_start, line_end];
            let line_points = line_points.iter().cloned().map(|p| p.to_array().into());
            let close_line = false;
            let line_events = lyon::path::iterator::FromPolyline::new(close_line, line_points);
            renderer.path_flat_color(
                local_transform,
                line_events,
                line.path.color,
                theme_primitive,
                path::Options::Stroke(line.path.opts),
            );
        }

        draw::renderer::PrimitiveRender::default()
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

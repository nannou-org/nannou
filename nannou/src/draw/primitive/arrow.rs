use crate::color::LinSrgba;
use crate::draw::primitive::path;
use crate::draw::primitive::Line;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{ColorScalar, SetColor, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};
use crate::geom::{self, pt2, vec2, Point2};
use crate::math::{BaseFloat, Zero};
use lyon::tessellation::StrokeOptions;

/// A path containing only two points - a start and end.
///
/// A triangle is drawn on the end to indicate direction.
#[derive(Clone, Debug)]
pub struct Arrow<S = geom::scalar::Default> {
    line: Line<S>,
    head_length: Option<S>,
    head_width: Option<S>,
}

/// The drawing context for a line.
pub type DrawingArrow<'a, S = geom::scalar::Default> = Drawing<'a, Arrow<S>, S>;

impl<S> Arrow<S> {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_line(|l| l.weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_line(|l| l.tolerance(tolerance))
    }

    /// Specify the start point of the arrow.
    pub fn start(self, start: Point2<S>) -> Self {
        self.map_line(|l| l.start(start))
    }

    /// Specify the end point of the arrow.
    pub fn end(self, end: Point2<S>) -> Self {
        self.map_line(|l| l.end(end))
    }

    /// Specify the start and end points of the arrow.
    pub fn points(self, start: Point2<S>, end: Point2<S>) -> Self {
        self.map_line(|l| l.points(start, end))
    }

    /// The length of the arrow head.
    ///
    /// By default, this is equal to `weight * 4.0`.
    ///
    /// This value will be clamped to the length of the line itself.
    pub fn head_length(mut self, length: S) -> Self {
        self.head_length = Some(length);
        self
    }

    /// The width of the arrow head.
    ///
    /// By default, this is equal to `weight * 2.0`.
    pub fn head_width(mut self, width: S) -> Self {
        self.head_width = Some(width);
        self
    }

    // Map the inner `PathStroke<S>` using the given function.
    fn map_line<F>(self, map: F) -> Self
    where
        F: FnOnce(Line<S>) -> Line<S>,
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

impl<'a, S> DrawingArrow<'a, S>
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

    /// Specify the start point of the arrow.
    pub fn start(self, start: Point2<S>) -> Self {
        self.map_ty(|ty| ty.start(start))
    }

    /// Specify the end point of the arrow.
    pub fn end(self, end: Point2<S>) -> Self {
        self.map_ty(|ty| ty.end(end))
    }

    /// Specify the start and end points of the arrow.
    pub fn points(self, start: Point2<S>, end: Point2<S>) -> Self {
        self.map_ty(|ty| ty.points(start, end))
    }

    /// The length of the arrow head.
    ///
    /// By default, this is equal to `weight * 4.0`.
    ///
    /// This value will be clamped to the length of the line itself.
    pub fn head_length(self, length: S) -> Self {
        self.map_ty(|ty| ty.head_length(length))
    }

    /// The width of the arrow head.
    ///
    /// By default, this is equal to `weight * 2.0`.
    pub fn head_width(self, width: S) -> Self {
        self.map_ty(|ty| ty.head_width(width))
    }
}

impl<S> SetStroke for Arrow<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.line)
    }
}

impl<S> SetOrientation<S> for Arrow<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.line)
    }
}

impl<S> SetPosition<S> for Arrow<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.line)
    }
}

impl<S> SetColor<ColorScalar> for Arrow<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.line)
    }
}

impl<'a, S> From<Arrow<S>> for Primitive<'a, S> {
    fn from(prim: Arrow<S>) -> Self {
        Primitive::Arrow(prim)
    }
}

impl<'a, S> Into<Option<Arrow<S>>> for Primitive<'a, S> {
    fn into(self) -> Option<Arrow<S>> {
        match self {
            Primitive::Arrow(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<'render> draw::renderer::RenderPrimitive<'render> for Arrow<f32> {
    fn render_primitive(
        self,
        mut ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender<'render> {
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
        let line_dir_mag = line_dir.magnitude();
        let tri_len = head_length.min(line_dir_mag);
        let tri_dir_norm = line_dir.with_magnitude(tri_len);
        let tri_start = end - tri_dir_norm;
        let tri_end = end;
        let line_start = start;
        let line_end = tri_start;
        let tri_a = tri_end;
        let tri_w_dir = vec2(-tri_dir_norm.y, tri_dir_norm.x).with_magnitude(head_width);
        let tri_b = tri_start + tri_w_dir;
        let tri_c = tri_start - tri_w_dir;
        // The line should only be drawn if there is space after drawing the triangle.
        let draw_line = line_dir_mag > tri_len;

        // Determine the transform to apply to all points.
        let global_transform = ctxt.transform;
        let local_transform = line.path.position.transform() * line.path.orientation.transform();
        let transform = global_transform * local_transform;

        // Draw the tri.
        let tri_points = [tri_a, tri_b, tri_c];
        let tri_points = tri_points.iter().cloned().map(Into::into);
        let close_tri = true;
        let tri_events = lyon::path::iterator::FromPolyline::new(close_tri, tri_points);
        path::render_path_events(
            tri_events,
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
            let line_points = line_points.iter().cloned().map(Into::into);
            let close_line = false;
            let line_events = lyon::path::iterator::FromPolyline::new(close_line, line_points);
            path::render_path_events(
                line_events,
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

        draw::renderer::PrimitiveRender::default()
    }
}

impl<S> Default for Arrow<S>
where
    S: Zero,
{
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

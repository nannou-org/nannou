use lyon::tessellation::{LineCap, LineJoin, StrokeOptions};

/// Nodes that support stroke tessellation.
///
/// This trait allows the `Drawing` context to automatically provide an implementation of the
/// following builder methods for all primitives that provide some stroke tessellation options.
pub trait SetStroke: Sized {
    /// Provide a mutable reference to the `StrokeOptions` field.
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions;

    /// Specify the whole set of stroke tessellation options.
    fn stroke_opts(mut self, opts: StrokeOptions) -> Self {
        *self.stroke_options_mut() = opts;
        self
    }

    /// The start line cap as specified by the SVG spec.
    fn start_cap(mut self, cap: LineCap) -> Self {
        self.stroke_options_mut().start_cap = cap;
        self
    }

    /// The end line cap as specified by the SVG spec.
    fn end_cap(mut self, cap: LineCap) -> Self {
        self.stroke_options_mut().end_cap = cap;
        self
    }

    /// The start and end line cap as specified by the SVG spec.
    fn caps(self, cap: LineCap) -> Self {
        self.start_cap(cap).end_cap(cap)
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    fn start_cap_butt(self) -> Self {
        self.start_cap(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    fn start_cap_square(self) -> Self {
        self.start_cap(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    fn start_cap_round(self) -> Self {
        self.start_cap(LineCap::Round)
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    fn end_cap_butt(self) -> Self {
        self.end_cap(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    fn end_cap_square(self) -> Self {
        self.end_cap(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    fn end_cap_round(self) -> Self {
        self.end_cap(LineCap::Round)
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    fn caps_butt(self) -> Self {
        self.caps(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    fn caps_square(self) -> Self {
        self.caps(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    fn caps_round(self) -> Self {
        self.caps(LineCap::Round)
    }

    /// The way in which lines are joined at the vertices, matching the SVG spec.
    ///
    /// Default value is `MiterClip`.
    fn join(mut self, join: LineJoin) -> Self {
        self.stroke_options_mut().line_join = join;
        self
    }

    /// A sharp corner is to be used to join path segments.
    fn join_miter(self) -> Self {
        self.join(LineJoin::Miter)
    }

    /// Same as a `join_miter`, but if the miter limit is exceeded, the miter is clipped at a miter
    /// length equal to the miter limit value multiplied by the stroke width.
    fn join_miter_clip(self) -> Self {
        self.join(LineJoin::MiterClip)
    }

    /// A round corner is to be used to join path segments.
    fn join_round(self) -> Self {
        self.join(LineJoin::Round)
    }

    /// A bevelled corner is to be used to join path segments. The bevel shape is a triangle that
    /// fills the area between the two stroked segments.
    fn join_bevel(self) -> Self {
        self.join(LineJoin::Bevel)
    }

    /// The total stroke_weight (aka width) of the line.
    fn stroke_weight(mut self, stroke_weight: f32) -> Self {
        self.stroke_options_mut().line_width = stroke_weight;
        self
    }

    /// Describes the limit before miter lines will clip, as described in the SVG spec.
    ///
    /// Must be greater than or equal to `1.0`.
    fn miter_limit(mut self, limit: f32) -> Self {
        self.stroke_options_mut().miter_limit = limit;
        self
    }

    /// Maximum allowed distance to the path when building an approximation.
    fn stroke_tolerance(mut self, tolerance: f32) -> Self {
        self.stroke_options_mut().tolerance = tolerance;
        self
    }
}

impl SetStroke for Option<StrokeOptions> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        self.get_or_insert_with(Default::default)
    }
}

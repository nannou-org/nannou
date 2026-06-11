use lyon::tessellation::FillOptions;

use crate::draw::{Draw, drawing};

/// Nodes that support fill tessellation.
///
/// This trait allows the `Drawing` context to automatically provide an implementation of the
/// following builder methods for all primitives that provide some fill tessellation options.
pub trait SetFill: Sized {
    /// Provide a mutable reference to the `FillOptions` field.
    fn fill_options_mut(&mut self) -> &mut FillOptions;

    /// Specify the whole set of fill tessellation options.
    fn fill_opts(mut self, opts: FillOptions) -> Self {
        *self.fill_options_mut() = opts;
        self
    }

    /// Maximum allowed distance to the path when building an approximation.
    fn fill_tolerance(mut self, tolerance: f32) -> Self {
        self.fill_options_mut().tolerance = tolerance;
        self
    }

    /// Specify the rule used to determine what is inside and what is outside of the shape.
    ///
    /// Currently, only the `EvenOdd` rule is implemented.
    fn fill_rule(mut self, rule: lyon::tessellation::FillRule) -> Self {
        self.fill_options_mut().fill_rule = rule;
        self
    }

    /// Whether to perform a vertical or horizontal traversal of the geometry.
    ///
    /// Default value: `Vertical`.
    fn fill_sweep_orientation(mut self, orientation: lyon::tessellation::Orientation) -> Self {
        self.fill_options_mut().sweep_orientation = orientation;
        self
    }

    /// A fast path to avoid some expensive operations if the path is known to not have any
    /// self-intersections.
    ///
    /// Do not set this to `false` if the path may have intersecting edges else the tessellator may
    /// panic or produce incorrect results. In doubt, do not change the default value.
    ///
    /// Default value: `true`.
    fn handle_intersections(mut self, handle: bool) -> Self {
        self.fill_options_mut().handle_intersections = handle;
        self
    }
}

impl SetFill for Option<FillOptions> {
    fn fill_options_mut(&mut self) -> &mut FillOptions {
        self.get_or_insert_with(Default::default)
    }
}

// An update to a primitive's fill tessellation options.
pub(crate) enum Update {
    Opts(FillOptions),
    Tolerance(f32),
    Rule(lyon::tessellation::FillRule),
    SweepOrientation(lyon::tessellation::Orientation),
    HandleIntersections(bool),
}

// Update the fill options of the primitive being drawn at `index`.
pub(crate) fn set_fill(draw: &Draw, index: usize, update: Update) {
    drawing::with_primitive(draw, index, |prim| match prim.fill_options_mut() {
        Some(opts) => apply_update(opts, update),
        None => bevy::log::warn_once!("drawing primitive does not support `fill` options"),
    })
}

fn apply_update(opts: &mut FillOptions, update: Update) {
    match update {
        Update::Opts(new) => *opts = new,
        Update::Tolerance(tolerance) => opts.tolerance = tolerance,
        Update::Rule(rule) => opts.fill_rule = rule,
        Update::SweepOrientation(orientation) => opts.sweep_orientation = orientation,
        Update::HandleIntersections(handle) => opts.handle_intersections = handle,
    }
}

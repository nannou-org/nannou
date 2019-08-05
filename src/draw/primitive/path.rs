use crate::color::LinSrgba;
use crate::draw::mesh::vertex::ColoredPoint2;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{self, orientation, position};
use crate::draw::properties::{
    ColorScalar, Draw, Drawn, IntoDrawn, SetColor, SetOrientation, SetPosition,
};
use crate::draw::{self, Drawing, DrawingContext};
use crate::geom::{self, pt2, Point2};
use crate::math::BaseFloat;
use lyon::path::iterator::FlattenedIterator;
use lyon::path::PathEvent;
use lyon::tessellation::geometry_builder::{self, GeometryBuilder, GeometryBuilderError, VertexId};
use lyon::tessellation::{
    FillOptions, FillTessellator, FillVertex, LineCap, LineJoin, StrokeOptions, StrokeTessellator,
    StrokeVertex, TessellationResult,
};
use std::cell::Cell;
use std::iter;
use std::ops;

/// A set of path tessellation options (FillOptions or StrokeOptions).
pub trait TessellationOptions {
    /// The tessellator for which the options are built.
    type Tessellator;
    /// The input vertex type for the geometry builder.
    type VertexInput;

    /// Initialise the tessellator.
    fn tessellator(tessellators: Tessellators) -> &mut Self::Tessellator;

    /// Tessellate the given path events into the given output.
    fn tessellate<I>(
        &self,
        tessellator: &mut Self::Tessellator,
        events: I,
        output: &mut dyn GeometryBuilder<Self::VertexInput>,
    ) -> TessellationResult
    where
        I: IntoIterator<Item = PathEvent>;
}

/// The beginning of the path building process, prior to choosing the tessellation mode (fill or
/// stroke).
#[derive(Clone, Debug, Default)]
pub struct PathInit;

/// A path drawing context ready to specify tessellation options.
#[derive(Clone, Debug)]
pub struct PathOptions<T> {
    opts: T,
}

/// Mutable access to stroke and fill tessellators.
pub struct Tessellators<'a> {
    pub fill: &'a mut FillTessellator,
    pub stroke: &'a mut StrokeTessellator,
}

/// A filled path drawing context.
pub type PathFill = PathOptions<FillOptions>;

/// A stroked path drawing context.
pub type PathStroke = PathOptions<StrokeOptions>;

/// Properties related to drawing a **Path**.
#[derive(Clone, Debug)]
pub struct Path<S = geom::scalar::Default> {
    color: Option<LinSrgba>,
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    vertex_data_ranges: draw::IntermediaryVertexDataRanges,
    index_range: ops::Range<usize>,
}

/// The initial drawing context for a path.
pub type DrawingPathInit<'a, S = geom::scalar::Default> = Drawing<'a, PathInit, S>;

/// The drawing context for a path in the tessellation options state.
pub type DrawingPathOptions<'a, T, S = geom::scalar::Default> = Drawing<'a, PathOptions<T>, S>;

/// The drawing context for a stroked path, prior to path event submission.
pub type DrawingPathStroke<'a, S = geom::scalar::Default> = Drawing<'a, PathStroke, S>;

/// The drawing context for a filled path, prior to path event submission.
pub type DrawingPathFill<'a, S = geom::scalar::Default> = Drawing<'a, PathFill, S>;

/// The drawing context for a polyline whose vertices have been specified.
pub type DrawingPath<'a, S = geom::scalar::Default> = Drawing<'a, Path<S>, S>;

pub struct PathGeometryBuilder<'a, 'mesh, S = geom::scalar::Default> {
    builder: &'a mut draw::IntermediaryMeshBuilder<'mesh, S>,
    color: &'a Cell<Option<draw::mesh::vertex::Color>>,
}

impl PathInit {
    /// Specify that we want to use fill tessellation for the path.
    ///
    /// The returned building context allows for specifying the fill tessellation options.
    pub fn fill(self) -> PathFill {
        let mut opts = FillOptions::default();
        opts.compute_normals = false;
        opts.on_error = lyon::tessellation::OnError::Recover;
        PathFill { opts }
    }

    /// Specify that we want to use stroke tessellation for the path.
    ///
    /// The returned building context allows for specifying the stroke tessellation options.
    pub fn stroke(self) -> PathStroke {
        let opts = Default::default();
        PathStroke { opts }
    }
}

impl PathFill {
    /// Maximum allowed distance to the path when building an approximation.
    pub fn tolerance(mut self, tolerance: f32) -> Self {
        self.opts.tolerance = tolerance;
        self
    }

    /// Specify the rule used to determine what is inside and what is outside of the shape.
    ///
    /// Currently, only the `EvenOdd` rule is implemented.
    pub fn rule(mut self, rule: lyon::tessellation::FillRule) -> Self {
        self.opts.fill_rule = rule;
        self
    }

    /// A fast path to avoid some expensive operations if the path is known to not have any
    /// self-intesections.
    ///
    /// Do not set this to `true` if the path may have intersecting edges else the tessellator may
    /// panic or produce incorrect results. In doubt, do not change the default value.
    ///
    /// Default value: `false`.
    pub fn assume_no_intersections(mut self, b: bool) -> Self {
        self.opts.assume_no_intersections = b;
        self
    }
}

impl PathStroke {
    /// The start line cap as specified by the SVG spec.
    pub fn start_cap(mut self, cap: LineCap) -> Self {
        self.opts.start_cap = cap;
        self
    }

    /// The end line cap as specified by the SVG spec.
    pub fn end_cap(mut self, cap: LineCap) -> Self {
        self.opts.end_cap = cap;
        self
    }

    /// The start and end line cap as specified by the SVG spec.
    pub fn caps(self, cap: LineCap) -> Self {
        self.start_cap(cap).end_cap(cap)
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn start_cap_butt(self) -> Self {
        self.start_cap(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn start_cap_square(self) -> Self {
        self.start_cap(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn start_cap_round(self) -> Self {
        self.start_cap(LineCap::Round)
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn end_cap_butt(self) -> Self {
        self.end_cap(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn end_cap_square(self) -> Self {
        self.end_cap(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn end_cap_round(self) -> Self {
        self.end_cap(LineCap::Round)
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn caps_butt(self) -> Self {
        self.caps(LineCap::Butt)
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn caps_square(self) -> Self {
        self.caps(LineCap::Square)
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn caps_round(self) -> Self {
        self.caps(LineCap::Round)
    }

    /// The way in which lines are joined at the vertices, matching the SVG spec.
    ///
    /// Default value is `MiterClip`.
    pub fn join(mut self, join: LineJoin) -> Self {
        self.opts.line_join = join;
        self
    }

    /// A sharp corner is to be used to join path segments.
    pub fn join_miter(self) -> Self {
        self.join(LineJoin::Miter)
    }

    /// Same as a `join_miter`, but if the miter limit is exceeded, the miter is clipped at a miter
    /// length equal to the miter limit value multiplied by the stroke width.
    pub fn join_miter_clip(self) -> Self {
        self.join(LineJoin::MiterClip)
    }

    /// A round corner is to be used to join path segments.
    pub fn join_round(self) -> Self {
        self.join(LineJoin::Round)
    }

    /// A bevelled corner is to be used to join path segments. The bevel shape is a triangle that
    /// fills the area between the two stroked segments.
    pub fn join_bevel(self) -> Self {
        self.join(LineJoin::Bevel)
    }

    /// The total thickness (aka width) of the line.
    pub fn thickness(mut self, thickness: f32) -> Self {
        self.opts.line_width = thickness;
        self
    }

    /// Describes the limit before miter lines will clip, as described in the SVG spec.
    ///
    /// Must be greater than or equal to `1.0`.
    pub fn miter_limit(mut self, limit: f32) -> Self {
        self.opts.miter_limit = limit;
        self
    }

    /// Maximum allowed distance to the path when building an approximation.
    pub fn tolerance(mut self, tolerance: f32) -> Self {
        self.opts.tolerance = tolerance;
        self
    }

    /// Specify the full set of stroke options for the stroke path tessellation.
    pub fn options(mut self, opts: StrokeOptions) -> Self {
        self.opts = opts;
        self
    }
}

impl PathStroke {
    /// Submit path events as a polyline of colored points.
    pub fn colored_points<S, I>(self, ctxt: DrawingContext<S>, points: I) -> Path<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<ColoredPoint2<S>>,
    {
        self.colored_points_inner(ctxt, false, points)
    }

    /// Submit path events as a polyline of colored points.
    pub fn colored_points_closed<S, I>(self, ctxt: DrawingContext<S>, points: I) -> Path<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<ColoredPoint2<S>>,
    {
        self.colored_points_inner(ctxt, true, points)
    }
}

impl<T> PathOptions<T>
where
    T: TessellationOptions,
{
    /// Submit the path events to be tessellated.
    pub(crate) fn events<'ctxt, S, I>(self, ctxt: DrawingContext<'ctxt, S>, events: I) -> Path<S>
    where
        S: BaseFloat,
        I: IntoIterator<Item = PathEvent>,
        for<'a> PathGeometryBuilder<'a, 'ctxt, S>: GeometryBuilder<T::VertexInput>,
    {
        let DrawingContext {
            mesh,
            fill_tessellator,
        } = ctxt;
        let color = Cell::new(None);
        let stroke = &mut StrokeTessellator::default();
        let tessellators = Tessellators {
            fill: fill_tessellator,
            stroke,
        };
        let mut tessellator = T::tessellator(tessellators);
        let mut builder = mesh.builder();
        let res = self.opts.tessellate(
            &mut tessellator,
            events,
            &mut PathGeometryBuilder {
                builder: &mut builder,
                color: &color,
            },
        );
        if let Err(err) = res {
            eprintln!("failed to tessellate polyline: {:?}", err);
        }
        Path::new(builder.vertex_data_ranges(), builder.index_range())
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points<'ctxt, S, I>(self, ctxt: DrawingContext<'ctxt, S>, points: I) -> Path<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<Point2<S>>,
        for<'a> PathGeometryBuilder<'a, 'ctxt, S>: GeometryBuilder<T::VertexInput>,
    {
        self.points_inner(ctxt, false, points)
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    ///
    /// Closes the start and end points.
    pub fn points_closed<'ctxt, S, I>(self, ctxt: DrawingContext<'ctxt, S>, points: I) -> Path<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<Point2<S>>,
        for<'a> PathGeometryBuilder<'a, 'ctxt, S>: GeometryBuilder<T::VertexInput>,
    {
        self.points_inner(ctxt, true, points)
    }

    // Consumes an iterator of points and converts them to an iterator yielding events.
    fn points_inner<'ctxt, S, I>(
        self,
        ctxt: DrawingContext<'ctxt, S>,
        close: bool,
        points: I,
    ) -> Path<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<Point2<S>>,
        for<'a> PathGeometryBuilder<'a, 'ctxt, S>: GeometryBuilder<T::VertexInput>,
    {
        let DrawingContext {
            mesh,
            fill_tessellator,
        } = ctxt;
        let color = Cell::new(None);
        let iter = points.into_iter().map(Into::into).map(|p| {
            let p: geom::Point2 = p.cast().expect("failed to cast point");
            lyon::math::point(p.x, p.y)
        });
        let events = lyon::path::iterator::FromPolyline::new(close, iter).path_events();
        let stroke = &mut StrokeTessellator::default();
        let tessellators = Tessellators {
            fill: fill_tessellator,
            stroke,
        };
        let mut tessellator = T::tessellator(tessellators);
        let mut builder = mesh.builder();
        let res = self.opts.tessellate(
            &mut tessellator,
            events,
            &mut PathGeometryBuilder {
                builder: &mut builder,
                color: &color,
            },
        );
        if let Err(err) = res {
            eprintln!("failed to tessellate polyline: {:?}", err);
        }
        Path::new(builder.vertex_data_ranges(), builder.index_range())
    }

    // Consumes an iterator of points and converts them to an iterator yielding events.
    fn colored_points_inner<'ctxt, S, I>(
        self,
        ctxt: DrawingContext<'ctxt, S>,
        close: bool,
        points: I,
    ) -> Path<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<ColoredPoint2<S>>,
        for<'a> PathGeometryBuilder<'a, 'ctxt, S>: GeometryBuilder<T::VertexInput>,
    {
        let DrawingContext {
            mesh,
            fill_tessellator,
        } = ctxt;
        let color = Cell::new(None);
        let iter = points.into_iter().map(Into::into).map(|p| {
            color.set(Some(p.color));
            let p: geom::Point2 = p.cast().expect("failed to cast point");
            lyon::math::point(p.x, p.y)
        });
        let events = lyon::path::iterator::FromPolyline::new(close, iter).path_events();
        let stroke = &mut StrokeTessellator::default();
        let tessellators = Tessellators {
            fill: fill_tessellator,
            stroke,
        };
        let mut tessellator = T::tessellator(tessellators);
        let mut builder = mesh.builder();
        let res = self.opts.tessellate(
            &mut tessellator,
            events,
            &mut PathGeometryBuilder {
                builder: &mut builder,
                color: &color,
            },
        );
        if let Err(err) = res {
            eprintln!("failed to tessellate polyline: {:?}", err);
        }
        Path::new(builder.vertex_data_ranges(), builder.index_range())
    }
}

impl<S> Path<S>
where
    S: BaseFloat,
{
    // Initialise a new `Path` with its ranges into the intermediary mesh, ready for drawing.
    fn new(
        vertex_data_ranges: draw::IntermediaryVertexDataRanges,
        index_range: ops::Range<usize>,
    ) -> Self {
        let orientation = Default::default();
        let position = Default::default();
        let color = Default::default();
        Path {
            color,
            orientation,
            position,
            vertex_data_ranges,
            index_range,
        }
    }
}

impl<'a, S> DrawingPathInit<'a, S>
where
    S: BaseFloat,
{
    /// Specify that we want to use fill tessellation for the path.
    ///
    /// The returned building context allows for specifying the fill tessellation options.
    pub fn fill(self) -> DrawingPathFill<'a, S> {
        self.map_ty(|ty| ty.fill())
    }

    /// Specify that we want to use stroke tessellation for the path.
    ///
    /// The returned building context allows for specifying the stroke tessellation options.
    pub fn stroke(self) -> DrawingPathStroke<'a, S> {
        self.map_ty(|ty| ty.stroke())
    }
}

impl<'a, S> DrawingPathFill<'a, S>
where
    S: BaseFloat,
{
    /// Maximum allowed distance to the path when building an approximation.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.tolerance(tolerance))
    }

    /// Specify the rule used to determine what is inside and what is outside of the shape.
    ///
    /// Currently, only the `EvenOdd` rule is implemented.
    pub fn rule(self, rule: lyon::tessellation::FillRule) -> Self {
        self.map_ty(|ty| ty.rule(rule))
    }

    /// A fast path to avoid some expensive operations if the path is known to not have any
    /// self-intesections.
    ///
    /// Do not set this to `true` if the path may have intersecting edges else the tessellator may
    /// panic or produce incorrect results. In doubt, do not change the default value.
    ///
    /// Default value: `false`.
    pub fn assume_no_intersections(self, b: bool) -> Self {
        self.map_ty(|ty| ty.assume_no_intersections(b))
    }
}

impl<'a, S> DrawingPathStroke<'a, S>
where
    S: BaseFloat,
{
    /// The start line cap as specified by the SVG spec.
    pub fn start_cap(self, cap: LineCap) -> Self {
        self.map_ty(|ty| ty.start_cap(cap))
    }

    /// The end line cap as specified by the SVG spec.
    pub fn end_cap(self, cap: LineCap) -> Self {
        self.map_ty(|ty| ty.end_cap(cap))
    }

    /// The start and end line cap as specified by the SVG spec.
    pub fn caps(self, cap: LineCap) -> Self {
        self.map_ty(|ty| ty.caps(cap))
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn start_cap_butt(self) -> Self {
        self.map_ty(|ty| ty.start_cap_butt())
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn start_cap_square(self) -> Self {
        self.map_ty(|ty| ty.start_cap_square())
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn start_cap_round(self) -> Self {
        self.map_ty(|ty| ty.start_cap_round())
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn end_cap_butt(self) -> Self {
        self.map_ty(|ty| ty.end_cap_butt())
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn end_cap_square(self) -> Self {
        self.map_ty(|ty| ty.end_cap_square())
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn end_cap_round(self) -> Self {
        self.map_ty(|ty| ty.end_cap_round())
    }

    /// The stroke for each sub-path does not extend beyond its two endpoints. A zero length
    /// sub-path will therefore not have any stroke.
    pub fn caps_butt(self) -> Self {
        self.map_ty(|ty| ty.caps_butt())
    }

    /// At the end of each sub-path, the shape representing the stroke will be extended by a
    /// rectangle with the same width as the stroke width and whose length is half of the stroke
    /// width. If a sub-path has zero length, then the resulting effect is that the stroke for that
    /// sub-path consists solely of a square with side length equal to the stroke width, centered
    /// at the sub-path's point.
    pub fn caps_square(self) -> Self {
        self.map_ty(|ty| ty.caps_square())
    }

    /// At each end of each sub-path, the shape representing the stroke will be extended by a half
    /// circle with a radius equal to the stroke width. If a sub-path has zero length, then the
    /// resulting effect is that the stroke for that sub-path consists solely of a full circle
    /// centered at the sub-path's point.
    pub fn caps_round(self) -> Self {
        self.map_ty(|ty| ty.caps_round())
    }

    /// The way in which lines are joined at the vertices, matching the SVG spec.
    ///
    /// Default value is `MiterClip`.
    pub fn join(self, join: LineJoin) -> Self {
        self.map_ty(|ty| ty.join(join))
    }

    /// A sharp corner is to be used to join path segments.
    pub fn join_miter(self) -> Self {
        self.map_ty(|ty| ty.join_miter())
    }

    /// Same as a `join_miter`, but if the miter limit is exceeded, the miter is clipped at a miter
    /// length equal to the miter limit value multiplied by the stroke width.
    pub fn join_miter_clip(self) -> Self {
        self.map_ty(|ty| ty.join_miter_clip())
    }

    /// A round corner is to be used to join path segments.
    pub fn join_round(self) -> Self {
        self.map_ty(|ty| ty.join_round())
    }

    /// A bevelled corner is to be used to join path segments. The bevel shape is a triangle that
    /// fills the area between the two stroked segments.
    pub fn join_bevel(self) -> Self {
        self.map_ty(|ty| ty.join_bevel())
    }

    /// The total thickness (aka width) of the line.
    pub fn thickness(self, thickness: f32) -> Self {
        self.map_ty(|ty| ty.thickness(thickness))
    }

    /// Describes the limit before miter lines will clip, as described in the SVG spec.
    ///
    /// Must be greater than or equal to `1.0`.
    pub fn miter_limit(self, limit: f32) -> Self {
        self.map_ty(|ty| ty.miter_limit(limit))
    }

    /// Maximum allowed distance to the path when building an approximation.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.tolerance(tolerance))
    }

    /// Specify the full set of stroke options for the path tessellation.
    pub fn stroke_options(self, opts: StrokeOptions) -> Self {
        self.map_ty(|ty| ty.options(opts))
    }

    /// Submit path events as a polyline of colored points.
    pub fn colored_points<I>(self, points: I) -> DrawingPath<'a, S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<ColoredPoint2<S>>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.colored_points(ctxt, points))
    }

    /// Submit path events as a polyline of colored points.
    ///
    /// The path with automatically close from the end point to the start point.
    pub fn colored_points_closed<I>(self, points: I) -> DrawingPath<'a, S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<ColoredPoint2<S>>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.colored_points_closed(ctxt, points))
    }
}

impl<'a, T, S> DrawingPathOptions<'a, T, S>
where
    S: BaseFloat,
    T: TessellationOptions,
    PathOptions<T>: IntoDrawn<S> + Into<Primitive<S>>,
    Primitive<S>: Into<Option<PathOptions<T>>>,
    for<'b, 'ctxt> PathGeometryBuilder<'b, 'ctxt, S>: GeometryBuilder<T::VertexInput>,
{
    /// Submit the path events to be tessellated.
    pub fn events<I>(self, events: I) -> DrawingPath<'a, S>
    where
        I: IntoIterator<Item = lyon::path::PathEvent>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.events(ctxt, events))
    }

    /// Submit the path events as a polyline of points.
    pub fn points<I>(self, points: I) -> DrawingPath<'a, S>
    where
        I: IntoIterator,
        I::Item: Into<Point2<S>>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points(ctxt, points))
    }

    /// Submit the path events as a polyline of points.
    ///
    /// An event will be generated that closes the start and end points.
    pub fn points_closed<I>(self, points: I) -> DrawingPath<'a, S>
    where
        I: IntoIterator,
        I::Item: Into<Point2<S>>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_closed(ctxt, points))
    }
}

impl TessellationOptions for FillOptions {
    type Tessellator = FillTessellator;
    type VertexInput = FillVertex;

    fn tessellator(tessellators: Tessellators) -> &mut Self::Tessellator {
        tessellators.fill
    }

    fn tessellate<I>(
        &self,
        tessellator: &mut Self::Tessellator,
        events: I,
        output: &mut dyn GeometryBuilder<Self::VertexInput>,
    ) -> TessellationResult
    where
        I: IntoIterator<Item = PathEvent>,
    {
        tessellator.tessellate_path(events, self, output)
    }
}

impl TessellationOptions for StrokeOptions {
    type Tessellator = StrokeTessellator;
    type VertexInput = StrokeVertex;

    fn tessellator(tessellators: Tessellators) -> &mut Self::Tessellator {
        tessellators.stroke
    }

    fn tessellate<I>(
        &self,
        tessellator: &mut Self::Tessellator,
        events: I,
        output: &mut dyn GeometryBuilder<Self::VertexInput>,
    ) -> TessellationResult
    where
        I: IntoIterator<Item = PathEvent>,
    {
        tessellator.tessellate_path(events, self, output)
    }
}

impl<'a, 'ctxt, S> GeometryBuilder<StrokeVertex> for PathGeometryBuilder<'a, 'ctxt, S>
where
    S: BaseFloat,
{
    fn begin_geometry(&mut self) {
        self.builder.begin_geom();
    }

    fn end_geometry(&mut self) -> geometry_builder::Count {
        self.builder.end_geom()
    }

    fn add_vertex(&mut self, v: StrokeVertex) -> Result<VertexId, GeometryBuilderError> {
        let point = pt2(v.position.x, v.position.y)
            .cast()
            .expect("failed to cast point");
        match self.color.get() {
            None => self.builder.add_vertex(point),
            Some(color) => {
                let colored_point: ColoredPoint2<S> = (point, color).into();
                self.builder.add_vertex(colored_point)
            }
        }
    }

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        self.builder.add_tri(a, b, c);
    }

    fn abort_geometry(&mut self) {
        self.builder.abort_geom();
    }
}

impl<'a, 'ctxt, S> GeometryBuilder<FillVertex> for PathGeometryBuilder<'a, 'ctxt, S>
where
    S: BaseFloat,
{
    fn begin_geometry(&mut self) {
        self.builder.begin_geom();
    }

    fn end_geometry(&mut self) -> geometry_builder::Count {
        self.builder.end_geom()
    }

    fn add_vertex(&mut self, v: FillVertex) -> Result<VertexId, GeometryBuilderError> {
        let point = pt2(v.position.x, v.position.y)
            .cast()
            .expect("failed to cast point");
        match self.color.get() {
            None => self.builder.add_vertex(point),
            Some(color) => {
                let colored_point: ColoredPoint2<S> = (point, color).into();
                self.builder.add_vertex(colored_point)
            }
        }
    }

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        self.builder.add_tri(a, b, c);
    }

    fn abort_geometry(&mut self) {
        self.builder.abort_geom();
    }
}

impl<S> IntoDrawn<S> for PathInit
where
    S: BaseFloat,
{
    type Vertices = iter::Empty<draw::mesh::Vertex<S>>;
    type Indices = iter::Empty<usize>;
    fn into_drawn(self, _draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let properties = Default::default();
        let vertices = iter::empty();
        let indices = iter::empty();
        (properties, vertices, indices)
    }
}

impl<S> IntoDrawn<S> for PathFill
where
    S: BaseFloat,
{
    type Vertices = iter::Empty<draw::mesh::Vertex<S>>;
    type Indices = iter::Empty<usize>;
    fn into_drawn(self, _draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let properties = Default::default();
        let vertices = iter::empty();
        let indices = iter::empty();
        (properties, vertices, indices)
    }
}

impl<S> IntoDrawn<S> for PathStroke
where
    S: BaseFloat,
{
    type Vertices = iter::Empty<draw::mesh::Vertex<S>>;
    type Indices = iter::Empty<usize>;
    fn into_drawn(self, _draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let properties = Default::default();
        let vertices = iter::empty();
        let indices = iter::empty();
        (properties, vertices, indices)
    }
}

impl<S> IntoDrawn<S> for Path<S>
where
    S: BaseFloat,
{
    type Vertices = draw::properties::VerticesFromRanges;
    type Indices = draw::properties::IndicesFromRange;
    fn into_drawn(self, draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Path {
            color,
            orientation,
            position,
            vertex_data_ranges,
            index_range,
        } = self;
        let dimensions = spatial::dimension::Properties::default();
        let spatial = spatial::Properties {
            dimensions,
            orientation,
            position,
        };
        let color = color.or_else(|| {
            if vertex_data_ranges.colors.len() >= vertex_data_ranges.points.len() {
                return None;
            }
            draw.theme(|theme| {
                theme
                    .color
                    .primitive
                    .get(&draw::theme::Primitive::Path)
                    .map(|&c| c.into_linear())
                    .or(Some(theme.color.default.into_linear()))
            })
        });
        let vertices = draw::properties::VerticesFromRanges {
            ranges: vertex_data_ranges,
            fill_color: color,
        };
        let indices = draw::properties::IndicesFromRange { range: index_range };
        (spatial, vertices, indices)
    }
}

impl<S> SetOrientation<S> for Path<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl<S> SetPosition<S> for Path<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.position)
    }
}

impl<S> SetColor<ColorScalar> for Path<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.color)
    }
}

impl<S> From<PathInit> for Primitive<S> {
    fn from(prim: PathInit) -> Self {
        Primitive::PathInit(prim)
    }
}

impl<S> From<PathStroke> for Primitive<S> {
    fn from(prim: PathStroke) -> Self {
        Primitive::PathStroke(prim)
    }
}

impl<S> From<PathFill> for Primitive<S> {
    fn from(prim: PathFill) -> Self {
        Primitive::PathFill(prim)
    }
}

impl<S> From<Path<S>> for Primitive<S> {
    fn from(prim: Path<S>) -> Self {
        Primitive::Path(prim)
    }
}

impl<S> Into<Option<PathInit>> for Primitive<S> {
    fn into(self) -> Option<PathInit> {
        match self {
            Primitive::PathInit(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<PathFill>> for Primitive<S> {
    fn into(self) -> Option<PathFill> {
        match self {
            Primitive::PathFill(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<PathStroke>> for Primitive<S> {
    fn into(self) -> Option<PathStroke> {
        match self {
            Primitive::PathStroke(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<Path<S>>> for Primitive<S> {
    fn into(self) -> Option<Path<S>> {
        match self {
            Primitive::Path(prim) => Some(prim),
            _ => None,
        }
    }
}

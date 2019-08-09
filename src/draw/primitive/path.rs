use crate::color::LinSrgba;
use crate::draw::mesh::vertex::ColoredPoint2;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{self, orientation, position};
use crate::draw::properties::{
    ColorScalar, Draw, Drawn, IntoDrawn, SetColor, SetFill, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Drawing, DrawingContext};
use crate::geom::{self, pt2, Point2};
use crate::math::BaseFloat;
use lyon::path::iterator::FlattenedIterator;
use lyon::path::PathEvent;
use lyon::tessellation::geometry_builder::{self, GeometryBuilder, GeometryBuilderError, VertexId};
use lyon::tessellation::{
    FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator, StrokeVertex,
    TessellationResult,
};
use std::cell::Cell;
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
#[derive(Clone, Debug)]
pub struct PathInit<S = geom::scalar::Default>(std::marker::PhantomData<S>);

/// A path drawing context ready to specify tessellation options.
#[derive(Clone, Debug)]
pub struct PathOptions<T, S = geom::scalar::Default> {
    opts: T,
    color: Option<LinSrgba>,
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
}

/// Mutable access to stroke and fill tessellators.
pub struct Tessellators<'a> {
    pub fill: &'a mut FillTessellator,
    pub stroke: &'a mut StrokeTessellator,
}

/// A filled path drawing context.
pub type PathFill<S = geom::scalar::Default> = PathOptions<FillOptions, S>;

/// A stroked path drawing context.
pub type PathStroke<S = geom::scalar::Default> = PathOptions<StrokeOptions, S>;

/// Properties related to drawing a **Path**.
#[derive(Clone, Debug)]
pub struct Path<S = geom::scalar::Default> {
    color: Option<LinSrgba>,
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    vertex_data_ranges: draw::IntermediaryVertexDataRanges,
    index_range: ops::Range<usize>,
    min_index: usize,
}

/// The initial drawing context for a path.
pub type DrawingPathInit<'a, S = geom::scalar::Default> = Drawing<'a, PathInit<S>, S>;

/// The drawing context for a path in the tessellation options state.
pub type DrawingPathOptions<'a, T, S = geom::scalar::Default> = Drawing<'a, PathOptions<T, S>, S>;

/// The drawing context for a stroked path, prior to path event submission.
pub type DrawingPathStroke<'a, S = geom::scalar::Default> = Drawing<'a, PathStroke<S>, S>;

/// The drawing context for a filled path, prior to path event submission.
pub type DrawingPathFill<'a, S = geom::scalar::Default> = Drawing<'a, PathFill<S>, S>;

/// The drawing context for a polyline whose vertices have been specified.
pub type DrawingPath<'a, S = geom::scalar::Default> = Drawing<'a, Path<S>, S>;

pub struct PathGeometryBuilder<'a, 'mesh, S = geom::scalar::Default> {
    builder: &'a mut draw::IntermediaryMeshBuilder<'mesh, S>,
    color: &'a Cell<Option<draw::mesh::vertex::Color>>,
}

impl<S> PathInit<S> {
    /// Specify that we want to use fill tessellation for the path.
    ///
    /// The returned building context allows for specifying the fill tessellation options.
    pub fn fill(self) -> PathFill<S> {
        let mut opts = FillOptions::default();
        opts.compute_normals = false;
        opts.on_error = lyon::tessellation::OnError::Recover;
        PathFill::new(opts)
    }

    /// Specify that we want to use stroke tessellation for the path.
    ///
    /// The returned building context allows for specifying the stroke tessellation options.
    pub fn stroke(self) -> PathStroke<S> {
        let opts = Default::default();
        PathStroke::new(opts)
    }
}

impl<T, S> PathOptions<T, S> {
    /// Initialise the `PathOptions` builder.
    pub fn new(opts: T) -> Self {
        let orientation = Default::default();
        let position = Default::default();
        let color = Default::default();
        PathOptions {
            opts,
            orientation,
            position,
            color,
        }
    }
}

impl<S> PathFill<S> {
    /// Maximum allowed distance to the path when building an approximation.
    ///
    /// This method is shorthand for the `fill_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.fill_tolerance(tolerance)
    }

    /// Specify the rule used to determine what is inside and what is outside of the shape.
    ///
    /// Currently, only the `EvenOdd` rule is implemented.
    ///
    /// This method is shorthand for the `fill_rule` method.
    pub fn rule(self, rule: lyon::tessellation::FillRule) -> Self {
        self.fill_rule(rule)
    }
}

impl<S> PathStroke<S> {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.stroke_weight(weight)
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.stroke_tolerance(tolerance)
    }

    /// Submit path events as a polyline of colored points.
    pub fn colored_points<I>(self, ctxt: DrawingContext<S>, points: I) -> Path<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<ColoredPoint2<S>>,
    {
        self.colored_points_inner(ctxt, false, points)
    }

    /// Submit path events as a polyline of colored points.
    pub fn colored_points_closed<I>(self, ctxt: DrawingContext<S>, points: I) -> Path<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<ColoredPoint2<S>>,
    {
        self.colored_points_inner(ctxt, true, points)
    }
}

impl<T, S> PathOptions<T, S>
where
    T: TessellationOptions,
{
    /// Submit the path events to be tessellated.
    pub(crate) fn events<'ctxt, I>(self, ctxt: DrawingContext<'ctxt, S>, events: I) -> Path<S>
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
            eprintln!("failed to tessellate path: {:?}", err);
        }
        Path::new(
            self.position,
            self.orientation,
            self.color,
            builder.vertex_data_ranges(),
            builder.index_range(),
            builder.min_index(),
        )
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points<'ctxt, I>(self, ctxt: DrawingContext<'ctxt, S>, points: I) -> Path<S>
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
    pub fn points_closed<'ctxt, I>(self, ctxt: DrawingContext<'ctxt, S>, points: I) -> Path<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<Point2<S>>,
        for<'a> PathGeometryBuilder<'a, 'ctxt, S>: GeometryBuilder<T::VertexInput>,
    {
        self.points_inner(ctxt, true, points)
    }

    // Consumes an iterator of points and converts them to an iterator yielding events.
    fn points_inner<'ctxt, I>(
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
        Path::new(
            self.position,
            self.orientation,
            self.color,
            builder.vertex_data_ranges(),
            builder.index_range(),
            builder.min_index(),
        )
    }

    // Consumes an iterator of points and converts them to an iterator yielding events.
    fn colored_points_inner<'ctxt, I>(
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
        Path::new(
            self.position,
            self.orientation,
            self.color,
            builder.vertex_data_ranges(),
            builder.index_range(),
            builder.min_index(),
        )
    }
}

impl<S> Path<S>
where
    S: BaseFloat,
{
    // Initialise a new `Path` with its ranges into the intermediary mesh, ready for drawing.
    fn new(
        position: position::Properties<S>,
        orientation: orientation::Properties<S>,
        color: Option<LinSrgba>,
        vertex_data_ranges: draw::IntermediaryVertexDataRanges,
        index_range: ops::Range<usize>,
        min_index: usize,
    ) -> Self {
        Path {
            color,
            orientation,
            position,
            vertex_data_ranges,
            index_range,
            min_index,
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
}

impl<'a, S> DrawingPathStroke<'a, S>
where
    S: BaseFloat,
{
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_ty(|ty| ty.stroke_weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.stroke_tolerance(tolerance))
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
    PathOptions<T, S>: Into<Primitive<S>>,
    Primitive<S>: Into<Option<PathOptions<T, S>>>,
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

impl<S> SetFill for PathFill<S> {
    fn fill_options_mut(&mut self) -> &mut FillOptions {
        &mut self.opts
    }
}

impl<S> SetStroke for PathStroke<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        &mut self.opts
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
            min_index,
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
            Some(draw.theme().fill_lin_srgba(&draw::theme::Primitive::Path))
        });
        let vertices = draw::properties::VerticesFromRanges::new(vertex_data_ranges, color);
        let indices = draw::properties::IndicesFromRange::new(index_range, min_index);
        (spatial, vertices, indices)
    }
}

impl<S> Default for PathInit<S> {
    fn default() -> Self {
        PathInit(std::marker::PhantomData)
    }
}

impl<T, S> Default for PathOptions<T, S>
where
    T: Default,
{
    fn default() -> Self {
        let opts = Default::default();
        PathOptions::new(opts)
    }
}

impl<T, S> SetOrientation<S> for PathOptions<T, S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl<T, S> SetPosition<S> for PathOptions<T, S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.position)
    }
}

impl<T, S> SetColor<ColorScalar> for PathOptions<T, S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.color)
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

impl<S> From<PathInit<S>> for Primitive<S> {
    fn from(prim: PathInit<S>) -> Self {
        Primitive::PathInit(prim)
    }
}

impl<S> From<PathStroke<S>> for Primitive<S> {
    fn from(prim: PathStroke<S>) -> Self {
        Primitive::PathStroke(prim)
    }
}

impl<S> From<PathFill<S>> for Primitive<S> {
    fn from(prim: PathFill<S>) -> Self {
        Primitive::PathFill(prim)
    }
}

impl<S> From<Path<S>> for Primitive<S> {
    fn from(prim: Path<S>) -> Self {
        Primitive::Path(prim)
    }
}

impl<S> Into<Option<PathInit<S>>> for Primitive<S> {
    fn into(self) -> Option<PathInit<S>> {
        match self {
            Primitive::PathInit(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<PathFill<S>>> for Primitive<S> {
    fn into(self) -> Option<PathFill<S>> {
        match self {
            Primitive::PathFill(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<PathStroke<S>>> for Primitive<S> {
    fn into(self) -> Option<PathStroke<S>> {
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

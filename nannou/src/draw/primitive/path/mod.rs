pub mod render;

pub(crate) use self::render::PathEventSource;
use crate::color::conv::IntoLinSrgba;
use crate::color::LinSrgba;
use crate::draw::mesh::vertex::TexCoords;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{
    ColorScalar, SetColor, SetFill, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Drawing, DrawingContext};
use crate::geom::Point2;
use crate::wgpu;
use lyon::path::PathEvent;
use lyon::tessellation::{FillOptions, FillTessellator, StrokeOptions, StrokeTessellator};

/// A set of path tessellation options (FillOptions or StrokeOptions).
pub trait TessellationOptions {
    /// The tessellator for which the options are built.
    type Tessellator;
    /// Convert the typed options instance to a dynamic one.
    fn into_options(self) -> Options;
}

/// The beginning of the path building process, prior to choosing the tessellation mode (fill or
/// stroke).
#[derive(Clone, Debug, Default)]
pub struct PathInit;

/// A path drawing context ready to specify tessellation options.
#[derive(Clone, Debug, Default)]
pub struct PathOptions<T> {
    pub(crate) opts: T,
    pub(crate) color: Option<LinSrgba>,
    pub(crate) position: position::Properties,
    pub(crate) orientation: orientation::Properties,
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
pub struct Path {
    color: Option<LinSrgba>,
    position: position::Properties,
    orientation: orientation::Properties,
    path_event_src: PathEventSource,
    options: Options,
    vertex_mode: draw::renderer::VertexMode,
    texture_view: Option<wgpu::TextureView>,
}

/// The initial drawing context for a path.
pub type DrawingPathInit<'a> = Drawing<'a, PathInit>;

/// The drawing context for a path in the tessellation options state.
pub type DrawingPathOptions<'a, T> = Drawing<'a, PathOptions<T>>;

/// The drawing context for a stroked path, prior to path event submission.
pub type DrawingPathStroke<'a> = Drawing<'a, PathStroke>;

/// The drawing context for a filled path, prior to path event submission.
pub type DrawingPathFill<'a> = Drawing<'a, PathFill>;

/// The drawing context for a polyline whose vertices have been specified.
pub type DrawingPath<'a> = Drawing<'a, Path>;

/// Dynamically distinguish between fill and stroke tessellation options.
#[derive(Clone, Debug)]
pub enum Options {
    Fill(FillOptions),
    Stroke(StrokeOptions),
}

impl PathInit {
    /// Specify that we want to use fill tessellation for the path.
    ///
    /// The returned building context allows for specifying the fill tessellation options.
    pub fn fill(self) -> PathFill {
        let opts = Default::default();
        PathFill::new(opts)
    }

    /// Specify that we want to use stroke tessellation for the path.
    ///
    /// The returned building context allows for specifying the stroke tessellation options.
    pub fn stroke(self) -> PathStroke {
        let opts = Default::default();
        PathStroke::new(opts)
    }
}

impl<T> PathOptions<T> {
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

impl PathFill {
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

impl PathStroke {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.stroke_weight(weight)
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.stroke_tolerance(tolerance)
    }
}

impl<T> PathOptions<T>
where
    T: TessellationOptions,
{
    /// Submit the path events to be tessellated.
    pub(crate) fn events<I>(self, ctxt: DrawingContext, events: I) -> Path
    where
        I: IntoIterator<Item = PathEvent>,
    {
        let DrawingContext {
            path_event_buffer, ..
        } = ctxt;
        let start = path_event_buffer.len();
        path_event_buffer.extend(events);
        let end = path_event_buffer.len();
        Path::new(
            self.position,
            self.orientation,
            self.color,
            PathEventSource::Buffered(start..end),
            self.opts.into_options(),
            draw::renderer::VertexMode::Color,
            None,
        )
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points<I>(self, ctxt: DrawingContext, points: I) -> Path
    where
        I: IntoIterator,
        I::Item: Into<Point2>,
    {
        self.points_inner(ctxt, false, points)
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    ///
    /// Closes the start and end points.
    pub fn points_closed<I>(self, ctxt: DrawingContext, points: I) -> Path
    where
        I: IntoIterator,
        I::Item: Into<Point2>,
    {
        self.points_inner(ctxt, true, points)
    }

    /// Submit path events as a polyline of colored points.
    pub fn points_colored<I, P, C>(self, ctxt: DrawingContext, points: I) -> Path
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Point2>,
        C: IntoLinSrgba<ColorScalar>,
    {
        self.points_colored_inner(ctxt, false, points)
    }

    /// Submit path events as a polyline of colored points.
    pub fn points_colored_closed<I, P, C>(self, ctxt: DrawingContext, points: I) -> Path
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Point2>,
        C: IntoLinSrgba<ColorScalar>,
    {
        self.points_colored_inner(ctxt, true, points)
    }

    /// Submit path events as a polyline of textured points.
    pub fn points_textured<I, P, TC>(
        self,
        ctxt: DrawingContext,
        texture_view: &dyn wgpu::ToTextureView,
        points: I,
    ) -> Path
    where
        I: IntoIterator<Item = (P, TC)>,
        P: Into<Point2>,
        TC: Into<TexCoords>,
    {
        self.points_textured_inner(ctxt, texture_view.to_texture_view(), false, points)
    }

    /// Submit path events as a polyline of textured points.
    pub fn points_textured_closed<I, P, TC>(
        self,
        ctxt: DrawingContext,
        texture_view: &dyn wgpu::ToTextureView,
        points: I,
    ) -> Path
    where
        I: IntoIterator<Item = (P, TC)>,
        P: Into<Point2>,
        TC: Into<TexCoords>,
    {
        self.points_textured_inner(ctxt, texture_view.to_texture_view(), true, points)
    }

    // Consumes an iterator of points and converts them to an iterator yielding events.
    fn points_inner<I>(self, ctxt: DrawingContext, close: bool, points: I) -> Path
    where
        I: IntoIterator,
        I::Item: Into<Point2>,
    {
        let iter = points
            .into_iter()
            .map(Into::into)
            .map(|p| lyon::math::point(p.x, p.y));
        let events = lyon::path::iterator::FromPolyline::new(close, iter);
        self.events(ctxt, events)
    }

    // Consumes an iterator of points and converts them to an iterator yielding events.
    fn points_colored_inner<I, P, C>(self, ctxt: DrawingContext, close: bool, points: I) -> Path
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Point2>,
        C: IntoLinSrgba<ColorScalar>,
    {
        let DrawingContext {
            path_points_colored_buffer,
            ..
        } = ctxt;
        let start = path_points_colored_buffer.len();
        let points = points
            .into_iter()
            .map(|(p, c)| (p.into(), c.into_lin_srgba()));
        path_points_colored_buffer.extend(points);
        let end = path_points_colored_buffer.len();
        let path_event_src = PathEventSource::ColoredPoints {
            range: start..end,
            close,
        };
        Path::new(
            self.position,
            self.orientation,
            self.color,
            path_event_src,
            self.opts.into_options(),
            draw::renderer::VertexMode::Color,
            None,
        )
    }

    // Consumes an iterator of textured points and buffers them for rendering.
    fn points_textured_inner<I, P, TC>(
        self,
        ctxt: DrawingContext,
        texture_view: wgpu::TextureView,
        close: bool,
        points: I,
    ) -> Path
    where
        I: IntoIterator<Item = (P, TC)>,
        P: Into<Point2>,
        TC: Into<TexCoords>,
    {
        let DrawingContext {
            path_points_textured_buffer,
            ..
        } = ctxt;
        let start = path_points_textured_buffer.len();
        let points = points.into_iter().map(|(p, tc)| (p.into(), tc.into()));
        path_points_textured_buffer.extend(points);
        let end = path_points_textured_buffer.len();
        let path_event_src = PathEventSource::TexturedPoints {
            range: start..end,
            close,
        };
        Path::new(
            self.position,
            self.orientation,
            self.color,
            path_event_src,
            self.opts.into_options(),
            draw::renderer::VertexMode::Texture,
            Some(texture_view),
        )
    }
}

impl Path {
    // Initialise a new `Path` with its ranges into the intermediary mesh, ready for drawing.
    fn new(
        position: position::Properties,
        orientation: orientation::Properties,
        color: Option<LinSrgba>,
        path_event_src: PathEventSource,
        options: Options,
        vertex_mode: draw::renderer::VertexMode,
        texture_view: Option<wgpu::TextureView>,
    ) -> Self {
        Path {
            color,
            orientation,
            position,
            path_event_src,
            options,
            vertex_mode,
            texture_view,
        }
    }
}

impl<'a> DrawingPathInit<'a> {
    /// Specify that we want to use fill tessellation for the path.
    ///
    /// The returned building context allows for specifying the fill tessellation options.
    pub fn fill(self) -> DrawingPathFill<'a> {
        self.map_ty(|ty| ty.fill())
    }

    /// Specify that we want to use stroke tessellation for the path.
    ///
    /// The returned building context allows for specifying the stroke tessellation options.
    pub fn stroke(self) -> DrawingPathStroke<'a> {
        self.map_ty(|ty| ty.stroke())
    }
}

impl<'a> DrawingPathFill<'a> {
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

impl<'a> DrawingPathStroke<'a> {
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_ty(|ty| ty.stroke_weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.stroke_tolerance(tolerance))
    }
}

impl<'a, T> DrawingPathOptions<'a, T>
where
    T: TessellationOptions,
    PathOptions<T>: Into<Primitive>,
    Primitive: Into<Option<PathOptions<T>>>,
{
    /// Submit the path events to be tessellated.
    pub fn events<I>(self, events: I) -> DrawingPath<'a>
    where
        I: IntoIterator<Item = lyon::path::PathEvent>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.events(ctxt, events))
    }

    /// Submit the path events as a polyline of points.
    pub fn points<I>(self, points: I) -> DrawingPath<'a>
    where
        I: IntoIterator,
        I::Item: Into<Point2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points(ctxt, points))
    }

    /// Submit the path events as a polyline of points.
    ///
    /// An event will be generated that closes the start and end points.
    pub fn points_closed<I>(self, points: I) -> DrawingPath<'a>
    where
        I: IntoIterator,
        I::Item: Into<Point2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_closed(ctxt, points))
    }

    /// Submit path events as a polyline of colored points.
    pub fn points_colored<I, P, C>(self, points: I) -> DrawingPath<'a>
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Point2>,
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_colored(ctxt, points))
    }

    /// Submit path events as a polyline of colored points.
    ///
    /// The path with automatically close from the end point to the start point.
    pub fn points_colored_closed<I, P, C>(self, points: I) -> DrawingPath<'a>
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Point2>,
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_colored_closed(ctxt, points))
    }

    /// Submit path events as a polyline of textured points.
    pub fn points_textured<I, P, TC>(
        self,
        view: &dyn wgpu::ToTextureView,
        points: I,
    ) -> DrawingPath<'a>
    where
        I: IntoIterator<Item = (P, TC)>,
        P: Into<Point2>,
        TC: Into<TexCoords>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_textured(ctxt, view, points))
    }

    /// Submit path events as a polyline of textured points.
    ///
    /// The path with automatically close from the end point to the start point.
    pub fn points_textured_closed<I, P, TC>(
        self,
        view: &dyn wgpu::ToTextureView,
        points: I,
    ) -> DrawingPath<'a>
    where
        I: IntoIterator<Item = (P, TC)>,
        P: Into<Point2>,
        TC: Into<TexCoords>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_textured_closed(ctxt, view, points))
    }
}

impl SetFill for PathFill {
    fn fill_options_mut(&mut self) -> &mut FillOptions {
        &mut self.opts
    }
}

impl SetStroke for PathStroke {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        &mut self.opts
    }
}

impl TessellationOptions for FillOptions {
    type Tessellator = FillTessellator;
    fn into_options(self) -> Options {
        Options::Fill(self)
    }
}

impl TessellationOptions for StrokeOptions {
    type Tessellator = StrokeTessellator;
    fn into_options(self) -> Options {
        Options::Stroke(self)
    }
}

impl<T> SetOrientation for PathOptions<T> {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl<T> SetPosition for PathOptions<T> {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.position)
    }
}

impl<T> SetColor<ColorScalar> for PathOptions<T> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.color)
    }
}

impl SetOrientation for Path {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl SetPosition for Path {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.position)
    }
}

impl SetColor<ColorScalar> for Path {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.color)
    }
}

impl From<PathInit> for Primitive {
    fn from(prim: PathInit) -> Self {
        Primitive::PathInit(prim)
    }
}

impl From<PathStroke> for Primitive {
    fn from(prim: PathStroke) -> Self {
        Primitive::PathStroke(prim)
    }
}

impl From<PathFill> for Primitive {
    fn from(prim: PathFill) -> Self {
        Primitive::PathFill(prim)
    }
}

impl From<Path> for Primitive {
    fn from(prim: Path) -> Self {
        Primitive::Path(prim)
    }
}

impl Into<Option<PathInit>> for Primitive {
    fn into(self) -> Option<PathInit> {
        match self {
            Primitive::PathInit(prim) => Some(prim),
            _ => None,
        }
    }
}

impl Into<Option<PathFill>> for Primitive {
    fn into(self) -> Option<PathFill> {
        match self {
            Primitive::PathFill(prim) => Some(prim),
            _ => None,
        }
    }
}

impl Into<Option<PathStroke>> for Primitive {
    fn into(self) -> Option<PathStroke> {
        match self {
            Primitive::PathStroke(prim) => Some(prim),
            _ => None,
        }
    }
}

impl Into<Option<Path>> for Primitive {
    fn into(self) -> Option<Path> {
        match self {
            Primitive::Path(prim) => Some(prim),
            _ => None,
        }
    }
}

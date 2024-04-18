use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{SetColor, SetFill, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing, DrawingContext};
use bevy::prelude::*;
use lyon::path::PathEvent;
use lyon::tessellation::{FillOptions, FillTessellator, StrokeOptions, StrokeTessellator};

/// A set of path tessellation options (FillOptions or StrokeOptions).
pub trait TessellationOptions {
    /// The tessellator for which the options are built.
    type Tessellator;
    /// Convert the typed options instance to a dynamic one.
    fn into_options(self) -> Options;
}

#[derive(Clone, Debug)]
pub(crate) enum PathEventSource {
    /// Fetch events from `path_events_buffer`.
    Buffered(std::ops::Range<usize>),
    /// Generate events from the `path_points_colored_buffer`.
    ColoredPoints {
        range: std::ops::Range<usize>,
        close: bool,
    },
    /// Generate events from the `path_points_textured_buffer`.
    TexturedPoints {
        range: std::ops::Range<usize>,
        close: bool,
    },
}

pub(crate) enum PathEventSourceIter<'a> {
    Events(&'a mut dyn Iterator<Item = lyon::path::PathEvent>),
    ColoredPoints {
        points: &'a mut dyn Iterator<Item = (Vec2, Color)>,
        close: bool,
    },
    TexturedPoints {
        points: &'a mut dyn Iterator<Item = (Vec2, Vec2)>,
        close: bool,
    },
}

/// The beginning of the path building process, prior to choosing the tessellation mode (fill or
/// stroke).
#[derive(Clone, Debug, Default)]
pub struct PathInit;

/// A path drawing context ready to specify tessellation options.
#[derive(Clone, Debug, Default)]
pub struct PathOptions<T> {
    pub(crate) opts: T,
    pub(crate) color: Option<Color>,
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
    color: Option<Color>,
    position: position::Properties,
    orientation: orientation::Properties,
    path_event_src: PathEventSource,
    options: Options,
    texture_handle: Option<Handle<Image>>,
}

/// The initial drawing context for a path.
pub type DrawingPathInit<'a, 'w, M> = Drawing<'a, 'w, PathInit, M>;

/// The drawing context for a path in the tessellation options state.
pub type DrawingPathOptions<'a, 'w, T, M> = Drawing<'a, 'w, PathOptions<T>, M>;

/// The drawing context for a stroked path, prior to path event submission.
pub type DrawingPathStroke<'a, 'w, M> = Drawing<'a, 'w, PathStroke, M>;

/// The drawing context for a filled path, prior to path event submission.
pub type DrawingPathFill<'a, 'w, M> = Drawing<'a, 'w, PathFill, M>;

/// The drawing context for a polyline whose vertices have been specified.
pub type DrawingPath<'a, 'w, M> = Drawing<'a, 'w, Path, M>;

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
        let opts = FillOptions::default();
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
            None,
        )
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points<I>(self, ctxt: DrawingContext, points: I) -> Path
    where
        I: IntoIterator,
        I::Item: Into<Vec2>,
    {
        self.points_inner(ctxt, false, points)
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    ///
    /// Closes the start and end points.
    pub fn points_closed<I>(self, ctxt: DrawingContext, points: I) -> Path
    where
        I: IntoIterator,
        I::Item: Into<Vec2>,
    {
        self.points_inner(ctxt, true, points)
    }

    /// Submit path events as a polyline of colored points.
    pub fn points_colored<I, P, C>(self, ctxt: DrawingContext, points: I) -> Path
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Vec2>,
        C: Into<Color>,
    {
        self.points_colored_inner(ctxt, false, points)
    }

    /// Submit path events as a polyline of colored points.
    pub fn points_colored_closed<I, P, C>(self, ctxt: DrawingContext, points: I) -> Path
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Vec2>,
        C: Into<Color>,
    {
        self.points_colored_inner(ctxt, true, points)
    }

    /// Submit path events as a polyline of textured points.
    pub fn points_textured<I, P, TC>(
        self,
        ctxt: DrawingContext,
        texture_handle: Handle<Image>,
        points: I,
    ) -> Path
    where
        I: IntoIterator<Item = (P, TC)>,
        P: Into<Vec2>,
        TC: Into<Vec2>,
    {
        self.points_textured_inner(ctxt, texture_handle, false, points)
    }

    /// Submit path events as a polyline of textured points.
    pub fn points_textured_closed<I, P, TC>(
        self,
        ctxt: DrawingContext,
        texture_handle: Handle<Image>,
        points: I,
    ) -> Path
    where
        I: IntoIterator<Item = (P, TC)>,
        P: Into<Vec2>,
        TC: Into<Vec2>,
    {
        self.points_textured_inner(ctxt, texture_handle, true, points)
    }

    // Consumes an iterator of points and converts them to an iterator yielding events.
    fn points_inner<I>(self, ctxt: DrawingContext, close: bool, points: I) -> Path
    where
        I: IntoIterator,
        I::Item: Into<Vec2>,
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
        P: Into<Vec2>,
        C: Into<Color>,
    {
        let DrawingContext {
            path_points_colored_buffer,
            ..
        } = ctxt;
        let start = path_points_colored_buffer.len();
        let points = points.into_iter().map(|(p, c)| (p.into(), c.into()));

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
            None,
        )
    }

    // Consumes an iterator of textured points and buffers them for rendering.
    fn points_textured_inner<I, P, TC>(
        self,
        ctxt: DrawingContext,
        texture_handle: Handle<Image>,
        close: bool,
        points: I,
    ) -> Path
    where
        I: IntoIterator<Item = (P, TC)>,
        P: Into<Vec2>,
        TC: Into<Vec2>,
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
            Some(texture_handle),
        )
    }
}

pub(crate) fn render_path_events<I>(
    events: I,
    color: Option<Color>,
    transform: Mat4,
    options: Options,
    theme: &draw::Theme,
    theme_prim: &draw::theme::Primitive,
    fill_tessellator: &mut lyon::tessellation::FillTessellator,
    stroke_tessellator: &mut lyon::tessellation::StrokeTessellator,
    mesh: &mut Mesh,
) where
    I: IntoIterator<Item = lyon::path::PathEvent>,
{
    let res = match options {
        Options::Fill(options) => {
            let color = color.unwrap_or_else(|| theme.fill(theme_prim));
            let mut mesh_builder = draw::mesh::MeshBuilder::single_color(mesh, transform, color);
            fill_tessellator.tessellate(events, &options, &mut mesh_builder)
        }
        Options::Stroke(options) => {
            let color = color.unwrap_or_else(|| theme.stroke(theme_prim));
            let mut mesh_builder = draw::mesh::MeshBuilder::single_color(mesh, transform, color);
            stroke_tessellator.tessellate(events, &options, &mut mesh_builder)
        }
    };
    if let Err(err) = res {
        eprintln!("failed to tessellate path: {:?}", err);
    }
}

pub(crate) fn render_path_points_colored<I>(
    points_colored: I,
    close: bool,
    transform: Mat4,
    options: Options,
    fill_tessellator: &mut lyon::tessellation::FillTessellator,
    stroke_tessellator: &mut lyon::tessellation::StrokeTessellator,
    mesh: &mut Mesh,
) where
    I: IntoIterator<Item = (Vec2, Color)>,
{
    let path = match points_colored_to_lyon_path(points_colored, close) {
        None => return,
        Some(p) => p,
    };

    // Extend the mesh with the built path.
    let mut mesh_builder = draw::mesh::MeshBuilder::color_per_point(mesh, transform);
    let res = match options {
        Options::Fill(options) => fill_tessellator.tessellate_with_ids(
            path.id_iter(),
            &path,
            Some(&path),
            &options,
            &mut mesh_builder,
        ),
        Options::Stroke(options) => stroke_tessellator.tessellate_with_ids(
            path.id_iter(),
            &path,
            Some(&path),
            &options,
            &mut mesh_builder,
        ),
    };
    if let Err(err) = res {
        eprintln!("failed to tessellate path: {:?}", err);
    }
}

pub(crate) fn render_path_points_textured<I>(
    points_textured: I,
    close: bool,
    transform: Mat4,
    options: Options,
    fill_tessellator: &mut lyon::tessellation::FillTessellator,
    stroke_tessellator: &mut lyon::tessellation::StrokeTessellator,
    mesh: &mut Mesh,
) where
    I: IntoIterator<Item = (Vec2, Vec2)>,
{
    let path = match points_textured_to_lyon_path(points_textured, close) {
        None => return,
        Some(p) => p,
    };

    // Extend the mesh with the built path.
    let mut mesh_builder = draw::mesh::MeshBuilder::tex_coords_per_point(mesh, transform);
    let res = match options {
        Options::Fill(options) => fill_tessellator.tessellate_with_ids(
            path.id_iter(),
            &path,
            Some(&path),
            &options,
            &mut mesh_builder,
        ),
        Options::Stroke(options) => stroke_tessellator.tessellate_with_ids(
            path.id_iter(),
            &path,
            Some(&path),
            &options,
            &mut mesh_builder,
        ),
    };
    if let Err(err) = res {
        eprintln!("failed to tessellate path: {:?}", err);
    }
}

pub(crate) fn render_path_source(
    // TODO:
    path_src: PathEventSourceIter,
    color: Option<Color>,
    transform: Mat4,
    options: Options,
    theme: &draw::Theme,
    theme_prim: &draw::theme::Primitive,
    fill_tessellator: &mut lyon::tessellation::FillTessellator,
    stroke_tessellator: &mut lyon::tessellation::StrokeTessellator,
    mesh: &mut Mesh,
) {
    match path_src {
        PathEventSourceIter::Events(events) => render_path_events(
            events,
            color,
            transform,
            options,
            theme,
            theme_prim,
            fill_tessellator,
            stroke_tessellator,
            mesh,
        ),
        PathEventSourceIter::ColoredPoints { points, close } => render_path_points_colored(
            points,
            close,
            transform,
            options,
            fill_tessellator,
            stroke_tessellator,
            mesh,
        ),
        PathEventSourceIter::TexturedPoints { points, close } => render_path_points_textured(
            points,
            close,
            transform,
            options,
            fill_tessellator,
            stroke_tessellator,
            mesh,
        ),
    }
}

impl draw::render::RenderPrimitive for Path {
    fn render_primitive(
        self,
        mut ctxt: draw::render::RenderContext,
        mesh: &mut Mesh,
    ) -> draw::render::PrimitiveRender {
        let Path {
            color,
            position,
            orientation,
            path_event_src,
            options,
            texture_handle,
        } = self;

        // Determine the transform to apply to all points.
        let global_transform = *ctxt.transform;
        let local_transform = position.transform() * orientation.transform();
        let transform = global_transform * local_transform;

        // A function for rendering the path.
        let render =
            |src: PathEventSourceIter,
             theme: &draw::Theme,
             fill_tessellator: &mut lyon::tessellation::FillTessellator,
             stroke_tessellator: &mut lyon::tessellation::StrokeTessellator| {
                render_path_source(
                    src,
                    color,
                    transform,
                    options,
                    theme,
                    &draw::theme::Primitive::Path,
                    fill_tessellator,
                    stroke_tessellator,
                    mesh,
                )
            };

        match path_event_src {
            PathEventSource::Buffered(range) => {
                let mut events = ctxt.path_event_buffer[range].iter().cloned();
                let src = PathEventSourceIter::Events(&mut events);
                render(
                    src,
                    &ctxt.theme,
                    &mut ctxt.fill_tessellator,
                    &mut ctxt.stroke_tessellator,
                );
            }
            PathEventSource::ColoredPoints { range, close } => {
                let mut points_colored = ctxt.path_points_colored_buffer[range].iter().cloned();
                let src = PathEventSourceIter::ColoredPoints {
                    points: &mut points_colored,
                    close,
                };
                render(
                    src,
                    &ctxt.theme,
                    &mut ctxt.fill_tessellator,
                    &mut ctxt.stroke_tessellator,
                );
            }
            PathEventSource::TexturedPoints { range, close } => {
                let mut points_textured = ctxt.path_points_textured_buffer[range].iter().cloned();
                let src = PathEventSourceIter::TexturedPoints {
                    points: &mut points_textured,
                    close,
                };
                render(
                    src,
                    &ctxt.theme,
                    &mut ctxt.fill_tessellator,
                    &mut ctxt.stroke_tessellator,
                );
            }
        }

        draw::render::PrimitiveRender {
            texture_handle,
        }
    }
}

/// Create a lyon path for the given iterator of colored points.
pub fn points_colored_to_lyon_path<I>(points_colored: I, close: bool) -> Option<lyon::path::Path>
where
    I: IntoIterator<Item = (Vec2, Color)>,
{
    // Build a path with a color attribute for each channel.
    let channels = 4;
    let mut path_builder = lyon::path::Path::builder_with_attributes(channels);

    // Begin the path.
    let mut iter = points_colored.into_iter();
    let (first_point, first_color) = iter.next()?;
    let p = first_point.to_array().into();
    path_builder.begin(p, &first_color.linear().to_f32_array());

    // Add the lines, keeping track of the last
    for (point, color) in iter {
        let p = point.to_array().into();
        path_builder.line_to(p, &color.linear().to_f32_array());
    }

    // End the path, closing if necessary.
    path_builder.end(close);

    // Build it!
    Some(path_builder.build())
}

/// Create a lyon path for the given iterator of textured points.
pub fn points_textured_to_lyon_path<I>(points_textured: I, close: bool) -> Option<lyon::path::Path>
where
    I: IntoIterator<Item = (Vec2, Vec2)>,
{
    // Build a path with a texture coords attribute for each channel.
    let channels = 2;
    let mut path_builder = lyon::path::Path::builder_with_attributes(channels);

    // Begin the path.
    let mut iter = points_textured.into_iter();
    let (first_point, first_tex_coords) = iter.next()?;
    let p = first_point.to_array().into();
    let (tc_x, tc_y) = first_tex_coords.into();
    path_builder.begin(p, &[tc_x, tc_y]);

    // Add the lines, keeping track of the last
    for (point, tex_coords) in iter {
        let p = point.to_array().into();
        let (tc_x, tc_y) = tex_coords.into();
        path_builder.line_to(p, &[tc_x, tc_y]);
    }

    // End the path, closing if necessary.
    path_builder.end(close);

    // Build it!
    Some(path_builder.build())
}

impl Path {
    // Initialise a new `Path` with its ranges into the intermediary mesh, ready for drawing.
    fn new(
        position: position::Properties,
        orientation: orientation::Properties,
        color: Option<Color>,
        path_event_src: PathEventSource,
        options: Options,
        texture_handle: Option<Handle<Image>>,
    ) -> Self {
        Path {
            color,
            orientation,
            position,
            path_event_src,
            options,
            texture_handle,
        }
    }
}

impl<'a, 'w, M> DrawingPathInit<'a, 'w, M>
    where M: Material + Default
{
    /// Specify that we want to use fill tessellation for the path.
    ///
    /// The returned building context allows for specifying the fill tessellation options.
    pub fn fill(self) -> DrawingPathFill<'a, 'w, M> {
        self.map_ty(|ty| ty.fill())
    }

    /// Specify that we want to use stroke tessellation for the path.
    ///
    /// The returned building context allows for specifying the stroke tessellation options.
    pub fn stroke(self) -> DrawingPathStroke<'a, 'w, M> {
        self.map_ty(|ty| ty.stroke())
    }
}

impl<'a, 'w, M> DrawingPathFill<'a, 'w, M>
    where M: Material + Default
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

impl<'a, 'w, M> DrawingPathStroke<'a, 'w, M>
    where M: Material + Default
{
    /// Short-hand for the `stroke_weight` method.
    pub fn weight(self, weight: f32) -> Self {
        self.map_ty(|ty| ty.stroke_weight(weight))
    }

    /// Short-hand for the `stroke_tolerance` method.
    pub fn tolerance(self, tolerance: f32) -> Self {
        self.map_ty(|ty| ty.stroke_tolerance(tolerance))
    }
}

impl<'a, 'w, T, M> DrawingPathOptions<'a, 'w, T, M>
where
    T: TessellationOptions,
    M: Material + Default,
    PathOptions<T>: Into<Primitive> + Clone,
    Primitive: Into<Option<PathOptions<T>>>,
{
    /// Submit the path events to be tessellated.
    pub fn events<I>(self, events: I) -> DrawingPath<'a, 'w, M>
    where
        I: IntoIterator<Item = lyon::path::PathEvent>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.events(ctxt, events))
    }

    /// Submit the path events as a polyline of points.
    pub fn points<I>(self, points: I) -> DrawingPath<'a, 'w, M>
    where
        I: IntoIterator,
        I::Item: Into<Vec2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points(ctxt, points))
    }

    /// Submit the path events as a polyline of points.
    ///
    /// An event will be generated that closes the start and end points.
    pub fn points_closed<I>(self, points: I) -> DrawingPath<'a, 'w, M>
    where
        I: IntoIterator,
        I::Item: Into<Vec2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_closed(ctxt, points))
    }

    /// Submit path events as a polyline of colored points.
    pub fn points_colored<I, P, C>(self, points: I) -> DrawingPath<'a, 'w, M>
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Vec2>,
        C: Into<Color>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_colored(ctxt, points))
    }

    /// Submit path events as a polyline of colored points.
    ///
    /// The path with automatically close from the end point to the start point.
    pub fn points_colored_closed<I, P, C>(self, points: I) -> DrawingPath<'a, 'w, M>
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Vec2>,
        C: Into<Color>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_colored_closed(ctxt, points))
    }

    /// Submit path events as a polyline of textured points.
    pub fn points_textured<I, P, TC>(
        self,
        texture_handle: Handle<Image>,
        points: I,
    ) -> DrawingPath<'a, 'w, M>
    where
        I: IntoIterator<Item = (P, TC)>,
        P: Into<Vec2>,
        TC: Into<Vec2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_textured(ctxt, texture_handle, points))
    }

    /// Submit path events as a polyline of textured points.
    ///
    /// The path with automatically close from the end point to the start point.
    pub fn points_textured_closed<I, P, TC>(
        self,
        texture_handle: Handle<Image>,
        points: I,
    ) -> DrawingPath<'a, 'w, M>
    where
        I: IntoIterator<Item = (P, TC)>,
        P: Into<Vec2>,
        TC: Into<Vec2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_textured_closed(ctxt, texture_handle, points))
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

impl<T> SetColor for PathOptions<T> {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.color)
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

impl SetColor for Path {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.color)
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

use crate::color::conv::IntoLinSrgba;
use crate::draw::drawing::DrawingContext;
use crate::draw::primitive::path::{self, PathEventSource};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{
    ColorScalar, LinSrgba, SetColor, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Drawing};
use crate::geom::{self, Point2};
use crate::math::{BaseFloat, Zero};
use lyon::path::PathEvent;
use lyon::tessellation::StrokeOptions;

/// A trait implemented for all polygon draw primitives.
pub trait SetPolygon<S>: Sized {
    /// Access to the polygon builder parameters.
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions<S>;

    /// Specify no fill color and in turn no fill tessellation for the polygon.
    fn no_fill(mut self) -> Self {
        self.polygon_options_mut().no_fill = true;
        self
    }

    /// Specify a color to use for stroke tessellation.
    ///
    /// Stroke tessellation will only be performed if this method or one of the `SetStroke` methods
    /// are called.
    fn stroke_color<C>(mut self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.polygon_options_mut().stroke_color = Some(color.into_lin_srgba());
        self
    }

    /// Specify the whole set of polygon options.
    fn polygon_options(mut self, opts: PolygonOptions<S>) -> Self {
        *self.polygon_options_mut() = opts;
        self
    }
}

/// State related to drawing a **Polygon**.
#[derive(Clone, Debug)]
pub struct PolygonInit<S = geom::scalar::Default> {
    pub(crate) opts: PolygonOptions<S>,
}

/// The set of options shared by all polygon types.
#[derive(Clone, Debug)]
pub struct PolygonOptions<S = geom::scalar::Default> {
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    no_fill: bool,
    stroke_color: Option<LinSrgba>,
    color: Option<LinSrgba>,
    stroke: Option<StrokeOptions>,
}

/// A polygon with vertices already submitted.
#[derive(Clone, Debug)]
pub struct Polygon<S = geom::scalar::Default> {
    opts: PolygonOptions<S>,
    path_event_src: PathEventSource,
}

/// Initialised drawing state for a polygon.
pub type DrawingPolygonInit<'a, S = geom::scalar::Default> = Drawing<'a, PolygonInit<S>, S>;

/// Initialised drawing state for a polygon.
pub type DrawingPolygon<'a, S = geom::scalar::Default> = Drawing<'a, Polygon<S>, S>;

impl<S> PolygonInit<S> {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.stroke_color(color)
    }

    /// Submit the path events to be tessellated.
    pub(crate) fn events<I>(self, ctxt: DrawingContext<S>, events: I) -> Polygon<S>
    where
        S: BaseFloat,
        I: IntoIterator<Item = PathEvent>,
    {
        let DrawingContext {
            path_event_buffer, ..
        } = ctxt;
        let start = path_event_buffer.len();
        path_event_buffer.extend(events);
        let end = path_event_buffer.len();
        Polygon {
            opts: self.opts,
            path_event_src: PathEventSource::Buffered(start..end),
        }
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points<I>(self, ctxt: DrawingContext<S>, points: I) -> Polygon<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<Point2<S>>,
    {
        let points = points.into_iter().map(|p| {
            let p: Point2<f32> = p.into().cast().expect("failed to cast point");
            p.into()
        });
        let close = true;
        let events = lyon::path::iterator::FromPolyline::new(close, points);
        self.events(ctxt, events)
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn colored_points<I>(self, ctxt: DrawingContext<S>, points: I) -> Polygon<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<draw::mesh::vertex::ColoredPoint2<S>>,
    {
        let DrawingContext {
            path_colored_points_buffer,
            ..
        } = ctxt;
        let start = path_colored_points_buffer.len();
        path_colored_points_buffer.extend(points.into_iter().map(Into::into));
        let end = path_colored_points_buffer.len();
        Polygon {
            opts: self.opts,
            path_event_src: PathEventSource::ColoredPoints {
                range: start..end,
                close: true,
            },
        }
    }
}

pub fn render_points_themed<I>(
    opts: PolygonOptions,
    points: I,
    mut ctxt: draw::renderer::RenderContext,
    theme_primitive: &draw::theme::Primitive,
    mesh: &mut draw::Mesh,
) where
    I: Clone + Iterator<Item = Point2>,
{
    let PolygonOptions {
        position,
        orientation,
        no_fill,
        stroke_color,
        color,
        stroke,
    } = opts;

    // Determine the transform to apply to all points.
    let global_transform = ctxt.transform;
    let local_transform = position.transform() * orientation.transform();
    let transform = global_transform * local_transform;

    // A function for rendering the path.
    let mut render =
        |opts: path::Options,
         color: Option<LinSrgba>,
         theme: &draw::Theme,
         fill_tessellator: &mut lyon::tessellation::FillTessellator,
         stroke_tessellator: &mut lyon::tessellation::StrokeTessellator| {
            path::render_path_events(
                lyon::path::iterator::FromPolyline::closed(points.clone().map(|p| p.into())),
                color,
                transform,
                opts,
                theme,
                theme_primitive,
                fill_tessellator,
                stroke_tessellator,
                mesh,
            )
        };

    // Do the fill tessellation first.
    if !no_fill {
        let opts = path::Options::Fill(lyon::tessellation::FillOptions::default());
        render(
            opts,
            color,
            &ctxt.theme,
            &mut ctxt.fill_tessellator,
            &mut ctxt.stroke_tessellator,
        );
    }

    // Do the stroke tessellation on top.
    if let Some(stroke_opts) = stroke {
        let opts = path::Options::Stroke(stroke_opts);
        let color = stroke_color;
        render(
            opts,
            color,
            &ctxt.theme,
            &mut ctxt.fill_tessellator,
            &mut ctxt.stroke_tessellator,
        );
    }
}

impl Polygon<f32> {
    pub(crate) fn render_themed(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
        theme_primitive: &draw::theme::Primitive,
    ) {
        let Polygon {
            path_event_src,
            opts:
                PolygonOptions {
                    position,
                    orientation,
                    no_fill,
                    stroke_color,
                    color,
                    stroke,
                },
        } = self;
        let draw::renderer::RenderContext {
            fill_tessellator,
            stroke_tessellator,
            path_event_buffer,
            path_colored_points_buffer,
            transform,
            theme,
            ..
        } = ctxt;

        // Determine the transform to apply to all points.
        let global_transform = transform;
        let local_transform = position.transform() * orientation.transform();
        let transform = global_transform * local_transform;

        // A function for rendering the path.
        let mut render =
            |src: path::PathEventSourceIter,
             opts: path::Options,
             color: Option<LinSrgba>,
             theme: &draw::Theme,
             fill_tessellator: &mut lyon::tessellation::FillTessellator,
             stroke_tessellator: &mut lyon::tessellation::StrokeTessellator| {
                path::render_path_source(
                    src,
                    color,
                    transform,
                    opts,
                    theme,
                    theme_primitive,
                    fill_tessellator,
                    stroke_tessellator,
                    mesh,
                )
            };

        // Do the fill tessellation first.
        if !no_fill {
            let opts = path::Options::Fill(lyon::tessellation::FillOptions::default());
            match path_event_src {
                PathEventSource::Buffered(ref range) => {
                    let mut events = path_event_buffer[range.clone()].iter().cloned();
                    let src = path::PathEventSourceIter::Events(&mut events);
                    render(
                        src,
                        opts,
                        color,
                        theme,
                        fill_tessellator,
                        stroke_tessellator,
                    );
                }
                PathEventSource::ColoredPoints { ref range, close } => {
                    let mut colored_points =
                        path_colored_points_buffer[range.clone()].iter().cloned();
                    let src = path::PathEventSourceIter::ColoredPoints {
                        points: &mut colored_points,
                        close,
                    };
                    render(
                        src,
                        opts,
                        color,
                        theme,
                        fill_tessellator,
                        stroke_tessellator,
                    );
                }
            }
        }

        // Then the the stroked outline.
        if let Some(stroke_opts) = stroke {
            let opts = path::Options::Stroke(stroke_opts);
            match path_event_src {
                PathEventSource::Buffered(range) => {
                    let mut events = path_event_buffer[range].iter().cloned();
                    let src = path::PathEventSourceIter::Events(&mut events);
                    render(
                        src,
                        opts,
                        stroke_color,
                        theme,
                        fill_tessellator,
                        stroke_tessellator,
                    );
                }
                PathEventSource::ColoredPoints { range, close } => {
                    // Move all this into another function that takes theme as an argument.
                    let color =
                        stroke_color.unwrap_or_else(|| theme.stroke_lin_srgba(theme_primitive));
                    let mut colored_points =
                        path_colored_points_buffer[range]
                            .iter()
                            .cloned()
                            .map(|mut v| {
                                v.color = color;
                                v
                            });
                    let src = path::PathEventSourceIter::ColoredPoints {
                        points: &mut colored_points,
                        close,
                    };
                    render(
                        src,
                        opts,
                        stroke_color,
                        theme,
                        fill_tessellator,
                        stroke_tessellator,
                    );
                }
            }
        }
    }
}

impl draw::renderer::RenderPrimitive for Polygon<f32> {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::VertexMode {
        self.render_themed(ctxt, mesh, &draw::theme::Primitive::Polygon);
        // TODO: Allow for textured paths
        draw::renderer::VertexMode::Color
    }
}

impl<'a, S, T> Drawing<'a, T, S>
where
    S: BaseFloat,
    T: SetPolygon<S> + Into<Primitive<S>>,
    Primitive<S>: Into<Option<T>>,
{
    /// Specify no fill color and in turn no fill tessellation for the polygon.
    pub fn no_fill(self) -> Self {
        self.map_ty(|ty| ty.no_fill())
    }

    /// Specify a color to use for stroke tessellation.
    ///
    /// Stroke tessellation will only be performed if this method or one of the `SetStroke` methods
    /// are called.
    pub fn stroke_color<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke_color(color))
    }

    /// Specify the whole set of polygon options.
    pub fn polygon_options(self, opts: PolygonOptions<S>) -> Self {
        self.map_ty(|ty| ty.polygon_options(opts))
    }
}

impl<'a, S> DrawingPolygonInit<'a, S>
where
    S: BaseFloat,
{
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    /// Describe the polygon with a sequence of path events.
    pub fn events<I>(self, events: I) -> DrawingPolygon<'a, S>
    where
        S: BaseFloat,
        I: IntoIterator<Item = PathEvent>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.events(ctxt, events))
    }

    /// Describe the polygon with a sequence of points.
    pub fn points<I>(self, points: I) -> DrawingPolygon<'a, S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<Point2<S>>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points(ctxt, points))
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn colored_points<I>(self, points: I) -> DrawingPolygon<'a, S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: Into<draw::mesh::vertex::ColoredPoint2<S>>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.colored_points(ctxt, points))
    }
}

impl<S> Default for PolygonInit<S>
where
    S: Zero,
{
    fn default() -> Self {
        let opts = Default::default();
        PolygonInit { opts }
    }
}

impl<S> Default for PolygonOptions<S>
where
    S: Zero,
{
    fn default() -> Self {
        let position = Default::default();
        let orientation = Default::default();
        let no_fill = false;
        let color = None;
        let stroke_color = None;
        let stroke = None;
        PolygonOptions {
            position,
            orientation,
            no_fill,
            color,
            stroke_color,
            stroke,
        }
    }
}

impl<S> SetPolygon<S> for PolygonOptions<S> {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions<S> {
        self
    }
}

impl<S> SetOrientation<S> for PolygonInit<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.opts.orientation)
    }
}

impl<S> SetPosition<S> for PolygonInit<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.opts.position)
    }
}

impl<S> SetColor<ColorScalar> for PolygonInit<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.opts.color)
    }
}

impl<S> SetPolygon<S> for PolygonInit<S> {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions<S> {
        SetPolygon::polygon_options_mut(&mut self.opts)
    }
}

impl<S> SetStroke for PolygonInit<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.opts.stroke)
    }
}

impl<S> SetOrientation<S> for Polygon<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.opts.orientation)
    }
}

impl<S> SetPosition<S> for Polygon<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.opts.position)
    }
}

impl<S> SetColor<ColorScalar> for Polygon<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.opts.color)
    }
}

impl<S> From<PolygonInit<S>> for Primitive<S> {
    fn from(prim: PolygonInit<S>) -> Self {
        Primitive::PolygonInit(prim)
    }
}

impl<S> From<Polygon<S>> for Primitive<S> {
    fn from(prim: Polygon<S>) -> Self {
        Primitive::Polygon(prim)
    }
}

impl<S> Into<Option<PolygonInit<S>>> for Primitive<S> {
    fn into(self) -> Option<PolygonInit<S>> {
        match self {
            Primitive::PolygonInit(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<Polygon<S>>> for Primitive<S> {
    fn into(self) -> Option<Polygon<S>> {
        match self {
            Primitive::Polygon(prim) => Some(prim),
            _ => None,
        }
    }
}

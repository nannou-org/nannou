use crate::color::conv::IntoLinSrgba;
use crate::draw::drawing::DrawingContext;
use crate::draw::mesh::vertex::TexCoords;
use crate::draw::primitive::path::{self, PathEventSource};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{
    ColorScalar, LinSrgba, SetColor, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Drawing};
use crate::geom::{self, Point2};
use crate::math::{BaseFloat, Zero};
use crate::wgpu;
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
    pub position: position::Properties<S>,
    pub orientation: orientation::Properties<S>,
    pub no_fill: bool,
    pub stroke_color: Option<LinSrgba>,
    pub color: Option<LinSrgba>,
    pub stroke: Option<StrokeOptions>,
}

/// A polygon with vertices already submitted.
#[derive(Clone, Debug)]
pub struct Polygon<S = geom::scalar::Default> {
    opts: PolygonOptions<S>,
    path_event_src: PathEventSource,
    texture_view: Option<wgpu::TextureView>,
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
            texture_view: None,
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
    pub fn points_colored<I, P, C>(self, ctxt: DrawingContext<S>, points: I) -> Polygon<S>
    where
        S: BaseFloat,
        I: IntoIterator<Item = (P, C)>,
        P: Into<Point2<S>>,
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
        Polygon {
            opts: self.opts,
            path_event_src: PathEventSource::ColoredPoints {
                range: start..end,
                close: true,
            },
            texture_view: None,
        }
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points_textured<I, P, T>(
        self,
        ctxt: DrawingContext<S>,
        view: &dyn wgpu::ToTextureView,
        points: I,
    ) -> Polygon<S>
    where
        S: BaseFloat,
        I: IntoIterator<Item = (P, T)>,
        P: Into<Point2<S>>,
        T: Into<TexCoords<S>>,
    {
        let DrawingContext {
            path_points_textured_buffer,
            ..
        } = ctxt;
        let start = path_points_textured_buffer.len();
        let points = points.into_iter().map(|(p, c)| (p.into(), c.into()));
        path_points_textured_buffer.extend(points);
        let end = path_points_textured_buffer.len();
        Polygon {
            opts: self.opts,
            path_event_src: PathEventSource::TexturedPoints {
                range: start..end,
                close: true,
            },
            texture_view: Some(view.to_texture_view()),
        }
    }
}

pub fn render_events_themed<F, I>(
    opts: PolygonOptions,
    events: F,
    mut ctxt: draw::renderer::RenderContext,
    theme_primitive: &draw::theme::Primitive,
    mesh: &mut draw::Mesh,
) where
    F: Fn() -> I,
    I: Iterator<Item = lyon::path::PathEvent>,
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
                events(),
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

pub fn render_points_themed<I>(
    opts: PolygonOptions,
    points: I,
    ctxt: draw::renderer::RenderContext,
    theme_primitive: &draw::theme::Primitive,
    mesh: &mut draw::Mesh,
) where
    I: Clone + Iterator<Item = Point2>,
{
    render_events_themed(
        opts,
        || lyon::path::iterator::FromPolyline::closed(points.clone().map(|p| p.into())),
        ctxt,
        theme_primitive,
        mesh,
    );
}

impl Polygon<f32> {
    pub(crate) fn render_themed(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
        theme_primitive: &draw::theme::Primitive,
    ) -> draw::renderer::PrimitiveRender {
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
            texture_view,
        } = self;
        let draw::renderer::RenderContext {
            fill_tessellator,
            stroke_tessellator,
            path_event_buffer,
            path_points_colored_buffer,
            path_points_textured_buffer,
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
                    let mut points_colored =
                        path_points_colored_buffer[range.clone()].iter().cloned();
                    let src = path::PathEventSourceIter::ColoredPoints {
                        points: &mut points_colored,
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
                PathEventSource::TexturedPoints { ref range, close } => {
                    let mut textured_points =
                        path_points_textured_buffer[range.clone()].iter().cloned();
                    let src = path::PathEventSourceIter::TexturedPoints {
                        points: &mut textured_points,
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
                    let color =
                        stroke_color.unwrap_or_else(|| theme.stroke_lin_srgba(theme_primitive));
                    let mut points_colored = path_points_colored_buffer[range]
                        .iter()
                        .cloned()
                        .map(|(point, _)| (point, color));
                    let src = path::PathEventSourceIter::ColoredPoints {
                        points: &mut points_colored,
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
                PathEventSource::TexturedPoints { range, close } => {
                    let mut textured_points = path_points_textured_buffer[range].iter().cloned();
                    let src = path::PathEventSourceIter::TexturedPoints {
                        points: &mut textured_points,
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

        match texture_view {
            None => draw::renderer::PrimitiveRender::default(),
            Some(texture_view) => draw::renderer::PrimitiveRender {
                texture_view: Some(texture_view),
                vertex_mode: draw::renderer::VertexMode::Texture,
            },
        }
    }
}

impl draw::renderer::RenderPrimitive for Polygon<f32> {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender {
        self.render_themed(ctxt, mesh, &draw::theme::Primitive::Polygon)
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
    pub fn points_colored<I, P, C>(self, points: I) -> DrawingPolygon<'a, S>
    where
        S: BaseFloat,
        I: IntoIterator<Item = (P, C)>,
        P: Into<Point2<S>>,
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_colored(ctxt, points))
    }

    /// Describe the polygon with an iterator yielding textured poings.
    pub fn points_textured<I, P, T>(
        self,
        view: &dyn wgpu::ToTextureView,
        points: I,
    ) -> DrawingPolygon<'a, S>
    where
        S: BaseFloat,
        I: IntoIterator<Item = (P, T)>,
        P: Into<Point2<S>>,
        T: Into<TexCoords<S>>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_textured(ctxt, view, points))
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

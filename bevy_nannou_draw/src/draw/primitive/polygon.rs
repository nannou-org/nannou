use crate::draw::drawing::DrawingContext;
use crate::draw::primitive::path::{self, PathEventSource};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{SetColor, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};
use bevy::prelude::*;
use lyon::path::PathEvent;
use lyon::tessellation::StrokeOptions;

/// A trait implemented for all polygon draw primitives.
pub trait SetPolygon: Sized {
    /// Access to the polygon builder parameters.
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions;

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
        C: Into<Color>,
    {
        self.polygon_options_mut().stroke_color = Some(color.into());
        self
    }

    /// Specify the whole set of polygon options.
    fn polygon_options(mut self, opts: PolygonOptions) -> Self {
        *self.polygon_options_mut() = opts;
        self
    }
}

/// State related to drawing a **Polygon**.
#[derive(Clone, Debug, Default)]
pub struct PolygonInit {
    pub(crate) opts: PolygonOptions,
}

/// The set of options shared by all polygon types.
#[derive(Clone, Debug, Default)]
pub struct PolygonOptions {
    pub position: position::Properties,
    pub orientation: orientation::Properties,
    pub no_fill: bool,
    pub stroke_color: Option<Color>,
    pub color: Option<Color>,
    pub stroke: Option<StrokeOptions>,
}

/// A polygon with vertices already submitted.
#[derive(Clone, Debug)]
pub struct Polygon {
    opts: PolygonOptions,
    path_event_src: PathEventSource,
    texture_handle: Option<Handle<Image>>,
}

/// Initialised drawing state for a polygon.
pub type DrawingPolygonInit<'a> = Drawing<'a, PolygonInit>;

/// Initialised drawing state for a polygon.
pub type DrawingPolygon<'a> = Drawing<'a, Polygon>;

impl PolygonInit {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.stroke_color(color)
    }

    /// Submit the path events to be tessellated.
    pub(crate) fn events<I>(self, ctxt: DrawingContext, events: I) -> Polygon
    where
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
            texture_handle: None,
        }
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points<I>(self, ctxt: DrawingContext, points: I) -> Polygon
    where
        I: IntoIterator,
        I::Item: Into<Vec2>,
    {
        let points = points.into_iter().map(|p| {
            let p: Vec2 = p.into();
            p.to_array().into()
        });
        let close = true;
        let events = lyon::path::iterator::FromPolyline::new(close, points);
        self.events(ctxt, events)
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points_colored<I, P, C>(self, ctxt: DrawingContext, points: I) -> Polygon
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
        Polygon {
            opts: self.opts,
            path_event_src: PathEventSource::ColoredPoints {
                range: start..end,
                close: true,
            },
            texture_handle: None,
        }
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points_textured<I, P, T>(
        self,
        ctxt: DrawingContext,
        texture_handle: Handle<Image>,
        points: I,
    ) -> Polygon
    where
        I: IntoIterator<Item = (P, T)>,
        P: Into<Vec2>,
        T: Into<Vec2>,
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
            texture_handle: Some(texture_handle),
        }
    }
}

pub fn render_events_themed<F, I>(
    opts: PolygonOptions,
    events: F,
    mut ctxt: draw::render::RenderContext,
    theme_primitive: &draw::theme::Primitive,
    mesh: &mut Mesh,
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
    let global_transform = *ctxt.transform;
    let local_transform = position.transform() * orientation.transform();
    let transform = global_transform * local_transform;

    // A function for rendering the path.
    let mut render =
        |opts: path::Options,
         color: Option<Color>,
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
    ctxt: draw::render::RenderContext,
    theme_primitive: &draw::theme::Primitive,
    mesh: &mut Mesh,
) where
    I: Clone + Iterator<Item = Vec2>,
{
    render_events_themed(
        opts,
        || lyon::path::iterator::FromPolyline::closed(points.clone().map(|p| p.to_array().into())),
        ctxt,
        theme_primitive,
        mesh,
    );
}

impl Polygon {
    pub(crate) fn render_themed(
        self,
        ctxt: draw::render::RenderContext,
        mesh: &mut Mesh,
        theme_primitive: &draw::theme::Primitive,
    ) -> draw::render::PrimitiveRender {
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
            texture_handle,
        } = self;
        let draw::render::RenderContext {
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
        let global_transform = *transform;
        let local_transform = position.transform() * orientation.transform();
        let transform = global_transform * local_transform;

        // A function for rendering the path.
        let mut render =
            |src: path::PathEventSourceIter,
             opts: path::Options,
             color: Option<Color>,
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
                    let color = stroke_color.unwrap_or_else(|| theme.stroke(theme_primitive));
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

        match texture_handle {
            None => draw::render::PrimitiveRender::default(),
            Some(texture_handle) => draw::render::PrimitiveRender {
                texture_handle: Some(texture_handle),
                vertex_mode: draw::render::VertexMode::Texture,
            },
        }
    }
}

impl draw::render::RenderPrimitive for Polygon {
    fn render_primitive(
        self,
        ctxt: draw::render::RenderContext,
        mesh: &mut Mesh,
    ) -> draw::render::PrimitiveRender {
        self.render_themed(ctxt, mesh, &draw::theme::Primitive::Polygon)
    }
}

impl<'a, T> Drawing<'a, T>
where
    T: SetPolygon + Into<Primitive>,
    Primitive: Into<Option<T>>,
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
        C: Into<Color>,
    {
        self.map_ty(|ty| ty.stroke_color(color))
    }

    /// Specify the whole set of polygon options.
    pub fn polygon_options(self, opts: PolygonOptions) -> Self {
        self.map_ty(|ty| ty.polygon_options(opts))
    }
}

impl<'a> DrawingPolygonInit<'a> {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    /// Describe the polygon with a sequence of path events.
    pub fn events<I>(self, events: I) -> DrawingPolygon<'a>
    where
        I: IntoIterator<Item = PathEvent>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.events(ctxt, events))
    }

    /// Describe the polygon with a sequence of points.
    pub fn points<I>(self, points: I) -> DrawingPolygon<'a>
    where
        I: IntoIterator,
        I::Item: Into<Vec2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points(ctxt, points))
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points_colored<I, P, C>(self, points: I) -> DrawingPolygon<'a>
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Vec2>,
        C: Into<Color>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_colored(ctxt, points))
    }

    /// Describe the polygon with an iterator yielding textured poings.
    pub fn points_textured<I, P, T>(
        self,
        texture_handle: Handle<Image>,
        points: I,
    ) -> DrawingPolygon<'a>
    where
        I: IntoIterator<Item = (P, T)>,
        P: Into<Vec2>,
        T: Into<Vec2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_textured(ctxt, texture_handle, points))
    }
}

impl SetPolygon for PolygonOptions {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions {
        self
    }
}

impl SetOrientation for PolygonInit {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.opts.orientation)
    }
}

impl SetPosition for PolygonInit {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.opts.position)
    }
}

impl SetColor for PolygonInit {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.opts.color)
    }
}

impl SetPolygon for PolygonInit {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions {
        SetPolygon::polygon_options_mut(&mut self.opts)
    }
}

impl SetStroke for PolygonInit {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.opts.stroke)
    }
}

impl SetOrientation for Polygon {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.opts.orientation)
    }
}

impl SetPosition for Polygon {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.opts.position)
    }
}

impl SetColor for Polygon {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.opts.color)
    }
}

impl From<PolygonInit> for Primitive {
    fn from(prim: PolygonInit) -> Self {
        Primitive::PolygonInit(prim)
    }
}

impl From<Polygon> for Primitive {
    fn from(prim: Polygon) -> Self {
        Primitive::Polygon(prim)
    }
}

impl Into<Option<PolygonInit>> for Primitive {
    fn into(self) -> Option<PolygonInit> {
        match self {
            Primitive::PolygonInit(prim) => Some(prim),
            _ => None,
        }
    }
}

impl Into<Option<Polygon>> for Primitive {
    fn into(self) -> Option<Polygon> {
        match self {
            Primitive::Polygon(prim) => Some(prim),
            _ => None,
        }
    }
}

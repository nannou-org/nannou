use bevy::prelude::*;
use lyon::path::PathEvent;
use lyon::tessellation::StrokeOptions;

use crate::draw::drawing::DrawingContext;
use crate::draw::primitive::path::{self, PathEventSource};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{SetColor, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};

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
}

/// Initialised drawing state for a polygon.
pub type DrawingPolygonInit<'a, M> = Drawing<'a, PolygonInit, M>;

/// Initialised drawing state for a polygon.
pub type DrawingPolygon<'a, M> = Drawing<'a, Polygon, M>;

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
    pub fn points_vertex<I, P, C, U>(self, ctxt: DrawingContext, points: I) -> Polygon
    where
        I: IntoIterator<Item = (P, C, U)>,
        P: Into<Vec2>,
        C: Into<Color>,
        U: Into<Vec2>,
    {
        let DrawingContext {
            path_points_vertex_buffer: path_points_colored_buffer,
            ..
        } = ctxt;
        let start = path_points_colored_buffer.len();
        let points = points.into_iter().map(|(p, c, u)| (p.into(), c.into(), u.into()));
        path_points_colored_buffer.extend(points);

        let end = path_points_colored_buffer.len();
        Polygon {
            opts: self.opts,
            path_event_src: PathEventSource::Vertex {
                range: start..end,
                close: true,
            },
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
    close: bool,
    points: I,
    mut ctxt: draw::render::RenderContext,
    theme_primitive: &draw::theme::Primitive,
    mesh: &mut Mesh,
) where
    I: Clone + Iterator<Item = (Vec2, Vec2)>,
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
            path::render_path_points_themed(
                points.clone(),
                close,
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

impl Polygon {
    pub(crate) fn render_themed(
        self,
        ctxt: draw::render::RenderContext,
        mesh: &mut Mesh,
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
        let draw::render::RenderContext {
            fill_tessellator,
            stroke_tessellator,
            path_event_buffer,
            path_points_vertex_buffer,
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
                PathEventSource::Vertex { ref range, close } => {
                    let mut points_colored =
                        path_points_vertex_buffer[range.clone()].iter().cloned();
                    let src = path::PathEventSourceIter::Vertex {
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
                PathEventSource::Vertex { range, close } => {
                    let color = stroke_color.unwrap_or_else(|| theme.stroke(theme_primitive));
                    let mut points_vertex = path_points_vertex_buffer[range]
                        .iter()
                        .cloned()
                        .map(|(point, _, tex_coord)| (point, color, tex_coord));
                    let src = path::PathEventSourceIter::Vertex {
                        points: &mut points_vertex,
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

impl draw::render::RenderPrimitive for Polygon {
    fn render_primitive(self, ctxt: draw::render::RenderContext, mesh: &mut Mesh) {
        self.render_themed(ctxt, mesh, &draw::theme::Primitive::Polygon)
    }
}

impl<'a, T, M> Drawing<'a, T, M>
where
    T: SetPolygon + Into<Primitive> + Clone,
    M: Material + Default,
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

impl<'a, M> DrawingPolygonInit<'a, M>
where
    M: Material + Default,
{
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    /// Describe the polygon with a sequence of path events.
    pub fn events<I>(self, events: I) -> DrawingPolygon<'a, M>
    where
        I: IntoIterator<Item = PathEvent>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.events(ctxt, events))
    }

    /// Describe the polygon with a sequence of points.
    pub fn points<I>(self, points: I) -> DrawingPolygon<'a, M>
    where
        I: IntoIterator,
        I::Item: Into<Vec2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points(ctxt, points))
    }

    pub fn points_colored<I, P, C>(self, points: I) -> DrawingPolygon<'a, M>
    where
        I: IntoIterator<Item = (P, C)>,
        P: Into<Vec2>,
        C: Into<Color>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_vertex(ctxt, points.into_iter().map(|(p, c)| (p, c, Vec2::ZERO))))
    }

    /// Consumes an iterator of points and converts them to an iterator yielding path events.
    pub fn points_vertex<I, P, C, U>(self, points: I) -> DrawingPolygon<'a, M>
    where
        I: IntoIterator<Item = (P, C, U)>,
        P: Into<Vec2>,
        C: Into<Color>,
        U: Into<Vec2>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.points_vertex(ctxt, points))
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

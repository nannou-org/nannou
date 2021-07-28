use crate::draw;
use crate::draw::primitive::path::{self, PathEventSource};
use crate::draw::primitive::polygon::{Polygon, PolygonOptions};
use crate::draw::properties::LinSrgba;
use crate::geom::Point2;

pub fn render_events_themed<F, I>(
    opts: PolygonOptions,
    events: F,
    mut ctxt: draw::renderer::RenderContext,
    theme_primitive: draw::theme::Primitive,
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
    let global_transform = *ctxt.transform;
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
    theme_primitive: draw::theme::Primitive,
    mesh: &mut draw::Mesh,
) where
    I: Clone + Iterator<Item = Point2>,
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
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
        theme_primitive: draw::theme::Primitive,
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
        let global_transform = *transform;
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

impl draw::renderer::RenderPrimitive for Polygon {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender {
        self.render_themed(ctxt, mesh, draw::theme::Primitive::Polygon)
    }
}

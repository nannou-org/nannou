use crate::draw;
use crate::draw::primitive::path::{self, PathEventSource};
use crate::draw::primitive::polygon::{Polygon, PolygonOptions};
use crate::geom::Point2;

pub fn render_events_themed<F, I, R>(
    options: PolygonOptions,
    events: F,
    theme_primitive: draw::theme::Primitive,
    mut renderer: R,
) where
    F: Fn() -> I,
    I: Iterator<Item = lyon::path::PathEvent>,
    R: draw::renderer::PrimitiveRenderer,
{
    let PolygonOptions {
        position,
        orientation,
        no_fill,
        stroke_color,
        color,
        stroke,
    } = options;

    let local_transform = position.transform() * orientation.transform();

    // Do the fill tessellation first.
    if !no_fill {
        let options = path::Options::Fill(lyon::tessellation::FillOptions::default());
        renderer.path_flat_color(local_transform, events(), color, theme_primitive, options);
    }

    // Do the stroke tessellation on top.
    if let Some(stroke_opts) = stroke {
        let options = path::Options::Stroke(stroke_opts);
        let color = stroke_color;
        renderer.path_flat_color(local_transform, events(), color, theme_primitive, options);
    }
}

pub fn render_points_themed<I, R>(
    options: PolygonOptions,
    points: I,
    theme_primitive: draw::theme::Primitive,
    renderer: R,
) where
    I: Clone + Iterator<Item = Point2>,
    R: draw::renderer::PrimitiveRenderer,
{
    render_events_themed(
        options,
        || lyon::path::iterator::FromPolyline::closed(points.clone().map(|p| p.to_array().into())),
        theme_primitive,
        renderer,
    );
}

impl draw::renderer::RenderPrimitive for Polygon {
    fn render_primitive<R>(
        self,
        ctxt: draw::renderer::RenderContext,
        mut renderer: R,
    ) -> draw::renderer::PrimitiveRender
    where
        R: draw::renderer::PrimitiveRenderer,
    {
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
            path_event_buffer,
            path_points_colored_buffer,
            path_points_textured_buffer,
            theme,
            ..
        } = ctxt;

        let theme_primitive = draw::theme::Primitive::Polygon;
        let local_transform = position.transform() * orientation.transform();

        // Do the fill tessellation first.
        if !no_fill {
            let options = path::Options::Fill(lyon::tessellation::FillOptions::default());
            match path_event_src {
                PathEventSource::Buffered(ref range) => {
                    let events = path_event_buffer[range.clone()].iter().cloned();
                    renderer.path_flat_color(
                        local_transform,
                        events,
                        color,
                        theme_primitive,
                        options,
                    );
                }
                PathEventSource::ColoredPoints { ref range, close } => {
                    let points_colored = path_points_colored_buffer[range.clone()].iter().cloned();
                    renderer.path_colored_points(local_transform, points_colored, close, options);
                }
                PathEventSource::TexturedPoints { ref range, close } => {
                    let points_textured =
                        path_points_textured_buffer[range.clone()].iter().cloned();
                    renderer.path_textured_points(local_transform, points_textured, close, options);
                }
            }
        }

        // Then the the stroked outline.
        if let Some(stroke_opts) = stroke {
            let options = path::Options::Stroke(stroke_opts);
            match path_event_src {
                PathEventSource::Buffered(range) => {
                    let events = path_event_buffer[range].iter().cloned();
                    renderer.path_flat_color(
                        local_transform,
                        events,
                        color,
                        theme_primitive,
                        options,
                    );
                }
                PathEventSource::ColoredPoints { range, close } => {
                    let color = theme.resolve_color(stroke_color, theme_primitive, &options);
                    let points_colored = path_points_colored_buffer[range]
                        .iter()
                        .cloned()
                        .map(|(point, _)| (point, color));
                    renderer.path_colored_points(local_transform, points_colored, close, options);
                }
                PathEventSource::TexturedPoints { range, close } => {
                    let points_textured = path_points_textured_buffer[range].iter().cloned();
                    renderer.path_textured_points(local_transform, points_textured, close, options);
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

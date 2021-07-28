use crate::color::LinSrgba;
use crate::draw;
use crate::draw::mesh::vertex::{Color, TexCoords};
use crate::draw::primitive::path::{Options, Path};
use crate::geom::Point2;
use crate::glam::Mat4;

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

pub(crate) fn render_path_events<I>(
    events: I,
    color: LinSrgba,
    transform: Mat4,
    options: Options,
    fill_tessellator: &mut lyon::tessellation::FillTessellator,
    stroke_tessellator: &mut lyon::tessellation::StrokeTessellator,
    mesh: &mut draw::Mesh,
) where
    I: IntoIterator<Item = lyon::path::PathEvent>,
{
    let res = match options {
        Options::Fill(options) => {
            let mut mesh_builder = draw::mesh::MeshBuilder::single_color(mesh, transform, color);
            fill_tessellator.tessellate(events, &options, &mut mesh_builder)
        }
        Options::Stroke(options) => {
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
    mesh: &mut draw::Mesh,
) where
    I: IntoIterator<Item = (Point2, Color)>,
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
    mesh: &mut draw::Mesh,
) where
    I: IntoIterator<Item = (Point2, TexCoords)>,
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

impl draw::renderer::RenderPrimitive2 for Path {
    fn render_primitive<R>(
        self,
        ctxt: draw::renderer::RenderContext2,
        mut renderer: R,
    ) -> draw::renderer::PrimitiveRender
    where
        R: draw::renderer::PrimitiveRenderer,
    {
        let Path {
            color,
            position,
            orientation,
            path_event_src,
            options,
            vertex_mode,
            texture_view,
        } = self;

        let local_transform = position.transform() * orientation.transform();

        match path_event_src {
            PathEventSource::Buffered(range) => {
                let events = ctxt.path_event_buffer[range].iter().cloned();
                renderer.path_flat_color(
                    local_transform,
                    events,
                    color,
                    draw::theme::Primitive::Path,
                    options,
                );
            }
            PathEventSource::ColoredPoints { range, close } => {
                let points_colored = ctxt.path_points_colored_buffer[range].iter().cloned();
                renderer.path_colored_points(local_transform, points_colored, close, options);
            }
            PathEventSource::TexturedPoints { range, close } => {
                let points_textured = ctxt.path_points_textured_buffer[range].iter().cloned();
                renderer.path_textured_points(local_transform, points_textured, close, options);
            }
        }

        draw::renderer::PrimitiveRender {
            texture_view,
            vertex_mode,
        }
    }
}

/// Create a lyon path for the given iterator of colored points.
pub fn points_colored_to_lyon_path<I>(points_colored: I, close: bool) -> Option<lyon::path::Path>
where
    I: IntoIterator<Item = (Point2, Color)>,
{
    // Build a path with a color attribute for each channel.
    let channels = draw::mesh::vertex::COLOR_CHANNEL_COUNT;
    let mut path_builder = lyon::path::Path::builder_with_attributes(channels);

    // Begin the path.
    let mut iter = points_colored.into_iter();
    let (first_point, first_color) = iter.next()?;
    let p = first_point.to_array().into();
    let (r, g, b, a) = first_color.into();
    path_builder.move_to(p, &[r, g, b, a]);

    // Add the lines, keeping track of the last
    for (point, color) in iter {
        let p = point.to_array().into();
        let (r, g, b, a) = color.into();
        path_builder.line_to(p, &[r, g, b, a]);
    }

    // Close if necessary.
    if close {
        path_builder.close();
    }

    // Build it!
    Some(path_builder.build())
}

/// Create a lyon path for the given iterator of textured points.
pub fn points_textured_to_lyon_path<I>(points_textured: I, close: bool) -> Option<lyon::path::Path>
where
    I: IntoIterator<Item = (Point2, TexCoords)>,
{
    // Build a path with a texture coords attribute for each channel.
    let channels = 2;
    let mut path_builder = lyon::path::Path::builder_with_attributes(channels);

    // Begin the path.
    let mut iter = points_textured.into_iter();
    let (first_point, first_tex_coords) = iter.next()?;
    let p = first_point.to_array().into();
    let (tc_x, tc_y) = first_tex_coords.into();
    path_builder.move_to(p, &[tc_x, tc_y]);

    // Add the lines, keeping track of the last
    for (point, tex_coords) in iter {
        let p = point.to_array().into();
        let (tc_x, tc_y) = tex_coords.into();
        path_builder.line_to(p, &[tc_x, tc_y]);
    }

    // Close if necessary.
    if close {
        path_builder.close();
    }

    // Build it!
    Some(path_builder.build())
}

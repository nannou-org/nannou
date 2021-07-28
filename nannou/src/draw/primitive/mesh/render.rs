use crate::draw;
use crate::draw::primitive::mesh::{FillColor, Mesh};
use crate::draw::properties::LinSrgba;
use crate::draw::theme::ColorType;
use crate::geom;
use crate::glam::Mat4;

pub fn render_mesh(
    transform: Mat4,
    vertex_range: std::ops::Range<usize>,
    index_range: std::ops::Range<usize>,
    fill_color: Option<LinSrgba>,
    intermediary_mesh: &draw::Mesh,
    mesh: &mut draw::Mesh,
) {
    // We need to update the indices to point to where vertices will be in the new mesh.
    let old_mesh_vertex_start = vertex_range.start as u32;
    let new_mesh_vertex_start = mesh.raw_vertex_count() as u32;
    let indices = index_range
        .map(|i| intermediary_mesh.indices()[i])
        .map(|i| new_mesh_vertex_start + i - old_mesh_vertex_start);

    // A small function for transforming a point via the transform matrix.
    let transform_point = |p: geom::Point3| -> geom::Point3 { transform.transform_point3(p) };

    // Color the vertices based on whether or not we should fill, then extend the mesh!
    match fill_color {
        Some(color) => {
            let vertices = vertex_range.map(|i| {
                let point = transform_point(intermediary_mesh.points()[i]);
                let tex_coords = intermediary_mesh.tex_coords()[i];
                ((point, color), tex_coords).into()
            });
            mesh.extend(vertices, indices);
        }
        None => {
            let vertices = vertex_range.map(|i| {
                let point = transform_point(intermediary_mesh.points()[i]);
                let color = intermediary_mesh.colors()[i];
                let tex_coords = intermediary_mesh.tex_coords()[i];
                ((point, color), tex_coords).into()
            });
            mesh.extend(vertices, indices);
        }
    }
}

impl draw::renderer::RenderPrimitive2 for Mesh {
    fn render_primitive<R>(
        self,
        ctxt: draw::renderer::RenderContext2,
        mut renderer: R,
    ) -> draw::renderer::PrimitiveRender
    where
        R: draw::renderer::PrimitiveRenderer,
    {
        let Mesh {
            orientation,
            position,
            vertex_range,
            index_range,
            vertex_mode,
            fill_color,
            texture_view,
        } = self;

        let theme_primitive = draw::theme::Primitive::Mesh;
        let fill_color = fill_color.map(|FillColor(color)| {
            ctxt.theme
                .resolve_color(color, theme_primitive, ColorType::Fill)
        });
        let local_transform = position.transform() * orientation.transform();

        renderer.mesh(local_transform, vertex_range, index_range, fill_color);

        draw::renderer::PrimitiveRender {
            texture_view,
            vertex_mode,
        }
    }
}

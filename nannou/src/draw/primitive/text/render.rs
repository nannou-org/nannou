use crate::draw;
use crate::draw::primitive::text::{Style, Text};
use crate::draw::properties::LinSrgba;
use crate::draw::renderer::GlyphCache;
use crate::geom::{self, Point2};
use crate::glam::{Mat4, Vec2};
use crate::text;

pub fn render_text(
    transform: Mat4,
    text: crate::text::Text,
    color: LinSrgba,
    glyph_colors: Vec<LinSrgba>,
    output_attachment_size: Vec2,
    output_attachment_scale_factor: f32,
    glyph_cache: &mut GlyphCache,
    mesh: &mut draw::Mesh,
) {
    // Queue the glyphs to be cached
    let font_id = text::font::id(text.font());
    let positioned_glyphs: Vec<_> = text
        .rt_glyphs(output_attachment_size, output_attachment_scale_factor)
        .collect();
    for glyph in positioned_glyphs.iter() {
        glyph_cache.queue_glyph(font_id.index(), glyph.clone());
    }

    // Cache the enqueued glyphs within the pixel buffer.
    let (glyph_cache_w, _) = glyph_cache.dimensions();
    {
        let GlyphCache {
            ref mut cache,
            ref mut pixel_buffer,
            ref mut requires_upload,
            ..
        } = glyph_cache;
        let glyph_cache_w = glyph_cache_w as usize;
        let res = cache.cache_queued(|rect, data| {
            let width = (rect.max.x - rect.min.x) as usize;
            let height = (rect.max.y - rect.min.y) as usize;
            let mut dst_ix = rect.min.y as usize * glyph_cache_w + rect.min.x as usize;
            let mut src_ix = 0;
            for _ in 0..height {
                let dst_range = dst_ix..dst_ix + width;
                let src_range = src_ix..src_ix + width;
                let dst_slice = &mut pixel_buffer[dst_range];
                let src_slice = &data[src_range];
                dst_slice.copy_from_slice(src_slice);
                dst_ix += glyph_cache_w;
                src_ix += width;
            }
            *requires_upload = true;
        });
        if let Err(err) = res {
            eprintln!("failed to cache queued glyphs: {}", err);
        }
    }

    // A function for converting RustType rects to nannou rects.
    let scale_factor = output_attachment_scale_factor;
    let (out_w, out_h) = output_attachment_size.into();
    let [half_out_w, half_out_h] = [out_w as f32 / 2.0, out_h as f32 / 2.0];
    let to_nannou_rect = |screen_rect: text::rt::Rect<i32>| {
        let l = screen_rect.min.x as f32 / scale_factor - half_out_w;
        let r = screen_rect.max.x as f32 / scale_factor - half_out_w;
        let t = -(screen_rect.min.y as f32 / scale_factor - half_out_h);
        let b = -(screen_rect.max.y as f32 / scale_factor - half_out_h);
        geom::Rect::from_corners([l, b].into(), [r, t].into())
    };

    let glyph_colors_iter = glyph_colors_iter(&text, &glyph_colors, color);

    // Extend the mesh with a rect for each displayed glyph.
    for (g, g_color) in positioned_glyphs.iter().zip(glyph_colors_iter) {
        if let Ok(Some((uv_rect, screen_rect))) = glyph_cache.rect_for(font_id.index(), &g) {
            let rect = to_nannou_rect(screen_rect);

            // Create a mesh-compatible vertex from the position and tex_coords.
            let v = |p: Point2, tex_coords: [f32; 2]| -> draw::mesh::Vertex {
                let p = transform.transform_point3([p.x, p.y, 0.0].into());
                let point = draw::mesh::vertex::Point::from(p);
                draw::mesh::vertex::new(point, g_color, tex_coords.into())
            };

            // The sides of the UV rect.
            let uv_l = uv_rect.min.x;
            let uv_t = uv_rect.min.y;
            let uv_r = uv_rect.max.x;
            let uv_b = uv_rect.max.y;

            // Insert the vertices.
            let bottom_left = v(rect.bottom_left(), [uv_l, uv_b]);
            let bottom_right = v(rect.bottom_right(), [uv_r, uv_b]);
            let top_left = v(rect.top_left(), [uv_l, uv_t]);
            let top_right = v(rect.top_right(), [uv_r, uv_t]);
            let start_ix = mesh.points().len() as u32;
            mesh.push_vertex(top_left);
            mesh.push_vertex(bottom_left);
            mesh.push_vertex(bottom_right);
            mesh.push_vertex(top_right);

            // Now the indices.
            let tl_ix = start_ix;
            let bl_ix = start_ix + 1;
            let br_ix = start_ix + 2;
            let tr_ix = start_ix + 3;
            mesh.push_index(tl_ix);
            mesh.push_index(bl_ix);
            mesh.push_index(br_ix);
            mesh.push_index(tl_ix);
            mesh.push_index(br_ix);
            mesh.push_index(tr_ix);
        }
    }
}

impl draw::renderer::RenderPrimitive for Text {
    fn render_primitive<R>(
        self,
        ctxt: draw::renderer::RenderContext,
        mut renderer: R,
    ) -> draw::renderer::PrimitiveRender
    where
        R: draw::renderer::PrimitiveRenderer,
    {
        let Text {
            spatial,
            style,
            text,
        } = self;
        let Style {
            color,
            glyph_colors,
            layout,
        } = style;
        let layout = layout.build();
        let (maybe_x, maybe_y, maybe_z) = (
            spatial.dimensions.x,
            spatial.dimensions.y,
            spatial.dimensions.z,
        );
        assert!(
            maybe_z.is_none(),
            "z dimension support for text is unimplemented"
        );
        let w = maybe_x.unwrap_or(200.0);
        let h = maybe_y.unwrap_or(200.0);
        let rect: geom::Rect = geom::Rect::from_wh([w, h].into());
        let color = ctxt.theme.resolve_color(
            color,
            draw::theme::Primitive::Text,
            draw::theme::ColorType::Fill,
        );

        let local_transform = spatial.position.transform() * spatial.orientation.transform();

        let text_str = &ctxt.text_buffer[text.clone()];
        let text = text::text(text_str).layout(&layout).build(rect);

        renderer.text(local_transform, text, color, glyph_colors);

        draw::renderer::PrimitiveRender::text()
    }
}

pub(crate) fn glyph_colors_iter<'a>(
    text: &'a crate::text::Text,
    glyph_colors: &'a [LinSrgba],
    color: LinSrgba,
) -> impl Iterator<Item = LinSrgba> + 'a {
    // Skips non-rendered colors (e.g. due to line breaks),
    //   assuming LineInfos are ordered by ascending character position.
    text.line_infos()
        .iter()
        .flat_map(|li| li.char_range())
        .take_while(move |&i| i < glyph_colors.len())
        .map(move |i| glyph_colors[i].to_owned())
        // Repeat `color` if more glyphs than glyph_colors
        .chain(std::iter::repeat(color))
}

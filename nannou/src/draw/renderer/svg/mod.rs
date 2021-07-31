mod encode;

use crate::draw::mesh::vertex::TexCoords;
use crate::draw::primitive::path::Options;
use crate::draw::properties::LinSrgba;
use crate::draw::renderer::{PrimitiveRenderer, RenderContext, RenderPrimitive};
use crate::draw::{self, Context, DrawCommand};
use crate::glam::{Mat4, Vec2};
use crate::Draw;
use nannou_core::geom::Rect;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use svg::node::element::Group;
use svg::Document;

pub fn render_and_save(dims: Rect, draw: &Draw, path: impl AsRef<Path>) {
    let document = render(dims, draw);
    write_file(path, &document).unwrap();
}

fn write_file(path: impl AsRef<Path>, document: &Document) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>\n".as_bytes())?;
    file.write_all(&document.to_string().into_bytes())?;
    file.sync_all()
}

pub fn render(dims: Rect, draw: &Draw) -> Document {
    let draw_commands = draw.drain_commands();
    let draw_state = draw.state.borrow();
    let intermediary_state = draw_state.intermediary_state.borrow();

    let mut svg = Group::new();
    let mut curr_ctxt = Default::default();

    for command in draw_commands {
        match command {
            DrawCommand::Context(new_context) => {
                let default_context = Context::default();
                if new_context.blend != default_context.blend
                    || new_context.scissor != default_context.scissor
                    || new_context.topology != default_context.topology
                    || new_context.sampler != default_context.sampler
                {
                    unimplemented!();
                }
                curr_ctxt = new_context;
            }
            DrawCommand::Primitive(primitive) => {
                let ctxt = RenderContext {
                    path_event_buffer: &intermediary_state.path_event_buffer,
                    path_points_colored_buffer: &intermediary_state.path_points_colored_buffer,
                    path_points_textured_buffer: &intermediary_state.path_points_textured_buffer,
                    text_buffer: &intermediary_state.text_buffer,
                    theme: &draw_state.theme,
                };

                let renderer = SvgPrimitiveRenderer {
                    transform: &curr_ctxt.transform,
                    theme: &draw_state.theme,
                    svg: &mut svg,
                };

                let _ = primitive.render_primitive(ctxt, renderer);
            }
        }
    }

    encode::svg_document(dims, svg, draw_state.background_color)
}

struct SvgPrimitiveRenderer<'a> {
    transform: &'a Mat4,
    theme: &'a draw::Theme,
    svg: &'a mut Group,
}

impl<'a> PrimitiveRenderer for SvgPrimitiveRenderer<'a> {
    fn path_flat_color(
        &mut self,
        local_transform: Mat4,
        events: impl Iterator<Item = lyon::path::PathEvent>,
        color: Option<LinSrgba>,
        theme_primitive: draw::theme::Primitive,
        options: Options,
    ) {
        let transform = *self.transform * local_transform;
        let color = self.theme.resolve_color(color, theme_primitive, &options);
        encode::render_path(self.svg, events, transform, color, options);
    }

    fn path_colored_points(
        &mut self,
        _local_transform: Mat4,
        _points_colored: impl Iterator<Item = (Vec2, LinSrgba)>,
        _close: bool,
        _options: Options,
    ) {
        unimplemented!();
    }

    fn path_textured_points(
        &mut self,
        _local_transform: Mat4,
        _points_textured: impl Iterator<Item = (Vec2, TexCoords)>,
        _close: bool,
        _options: Options,
    ) {
        unimplemented!();
    }

    fn mesh(
        &mut self,
        _local_transform: Mat4,
        _vertex_range: std::ops::Range<usize>,
        _index_range: std::ops::Range<usize>,
        _fill_color: Option<LinSrgba>,
    ) {
        unimplemented!();
    }

    fn text(
        &mut self,
        local_transform: Mat4,
        _text: crate::text::Text,
        _color: LinSrgba,
        _glyph_colors: Vec<LinSrgba>,
    ) {
        let _transform = *self.transform * local_transform;
        // TODO: render each glyph as a path_flat_color
        unimplemented!();
    }
}

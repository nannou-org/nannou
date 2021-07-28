use crate::color::LinSrgba;
use crate::draw;
use crate::draw::mesh::vertex::{Color, TexCoords};
use crate::draw::primitive::mesh::render_mesh;
use crate::draw::primitive::path::render::{
    render_path_events, render_path_points_colored, render_path_points_textured,
};
use crate::draw::primitive::text::render_text;
use crate::draw::renderer::PrimitiveRendererImpl;
use crate::geom::Point2;
use crate::glam::{Mat4, Vec2};
use draw::primitive::path::Options;
use draw::renderer::PrimitiveRender;
use lyon::path::PathEvent;

/// The context provided to primitives to assist with the rendering process.
pub struct RenderContext<'a> {
    pub path_event_buffer: &'a [PathEvent],
    pub path_points_colored_buffer: &'a [(Point2, Color)],
    pub path_points_textured_buffer: &'a [(Point2, TexCoords)],
    pub text_buffer: &'a str,
    pub theme: &'a draw::Theme,
}

pub trait RenderPrimitive {
    /// Render self into the given PrimitiveRenderer.
    fn render_primitive<R>(self, ctxt: RenderContext, renderer: R) -> PrimitiveRender
    where
        R: PrimitiveRenderer;
}

pub trait PrimitiveRenderer {
    fn path_flat_color(
        &mut self,
        local_transform: Mat4,
        events: impl Iterator<Item = lyon::path::PathEvent>,
        color: Option<LinSrgba>,
        theme_primitive: draw::theme::Primitive,
        options: Options,
    );

    fn path_colored_points(
        &mut self,
        local_transform: Mat4,
        points_colored: impl Iterator<Item = (Vec2, LinSrgba)>,
        close: bool,
        options: Options,
    );

    fn path_textured_points(
        &mut self,
        local_transform: Mat4,
        points_textured: impl Iterator<Item = (Vec2, TexCoords)>,
        close: bool,
        options: Options,
    );

    fn mesh(
        &mut self,
        local_transform: Mat4,
        vertex_range: std::ops::Range<usize>,
        index_range: std::ops::Range<usize>,
        fill_color: Option<LinSrgba>,
    );

    fn text(
        &mut self,
        local_transform: Mat4,
        text: crate::text::Text,
        color: LinSrgba,
        glyph_colors: Vec<LinSrgba>,
    );
}

impl RenderPrimitive for draw::Primitive {
    fn render_primitive<R>(self, ctxt: RenderContext, renderer: R) -> PrimitiveRender
    where
        R: PrimitiveRenderer,
    {
        match self {
            draw::Primitive::Arrow(prim) => prim.render_primitive(ctxt, renderer),
            draw::Primitive::Mesh(prim) => prim.render_primitive(ctxt, renderer),
            draw::Primitive::Path(prim) => prim.render_primitive(ctxt, renderer),
            draw::Primitive::Polygon(prim) => prim.render_primitive(ctxt, renderer),
            draw::Primitive::Tri(prim) => prim.render_primitive(ctxt, renderer),
            draw::Primitive::Ellipse(prim) => prim.render_primitive(ctxt, renderer),
            draw::Primitive::Quad(prim) => prim.render_primitive(ctxt, renderer),
            draw::Primitive::Rect(prim) => prim.render_primitive(ctxt, renderer),
            draw::Primitive::Line(prim) => prim.render_primitive(ctxt, renderer),
            draw::Primitive::Text(prim) => prim.render_primitive(ctxt, renderer),
            draw::Primitive::Texture(prim) => prim.render_primitive(ctxt, renderer),
            _ => PrimitiveRender::default(),
        }
    }
}

impl<'a> PrimitiveRenderer for PrimitiveRendererImpl<'a> {
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
        render_path_events(
            events,
            color,
            transform,
            options,
            self.fill_tessellator,
            self.stroke_tessellator,
            self.mesh,
        );
    }

    fn path_colored_points(
        &mut self,
        local_transform: Mat4,
        points_colored: impl Iterator<Item = (Vec2, LinSrgba)>,
        close: bool,
        options: Options,
    ) {
        let transform = *self.transform * local_transform;
        render_path_points_colored(
            points_colored,
            close,
            transform,
            options,
            self.fill_tessellator,
            self.stroke_tessellator,
            self.mesh,
        );
    }

    fn path_textured_points(
        &mut self,
        local_transform: Mat4,
        points_textured: impl Iterator<Item = (Vec2, TexCoords)>,
        close: bool,
        options: Options,
    ) {
        let transform = *self.transform * local_transform;
        render_path_points_textured(
            points_textured,
            close,
            transform,
            options,
            self.fill_tessellator,
            self.stroke_tessellator,
            self.mesh,
        );
    }

    fn mesh(
        &mut self,
        local_transform: Mat4,
        vertex_range: std::ops::Range<usize>,
        index_range: std::ops::Range<usize>,
        fill_color: Option<LinSrgba>,
    ) {
        let transform = *self.transform * local_transform;
        render_mesh(
            transform,
            vertex_range,
            index_range,
            fill_color,
            self.intermediary_mesh,
            self.mesh,
        );
    }

    fn text(
        &mut self,
        local_transform: Mat4,
        text: crate::text::Text,
        color: LinSrgba,
        glyph_colors: Vec<LinSrgba>,
    ) {
        let transform = *self.transform * local_transform;
        render_text(
            transform,
            text,
            color,
            glyph_colors,
            self.output_attachment_size,
            self.output_attachment_scale_factor,
            self.glyph_cache,
            self.mesh,
        );
    }
}

impl From<&Options> for draw::theme::ColorType {
    fn from(options: &Options) -> Self {
        match options {
            Options::Fill(_) => Self::Fill,
            Options::Stroke(_) => Self::Stroke,
        }
    }
}

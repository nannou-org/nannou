use crate::{draw, text};
use bevy::prelude::*;
use lyon::path::PathEvent;
use lyon::tessellation::{FillTessellator, StrokeTessellator};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

/// Draw API primitives that may be rendered via the **Renderer** type.
pub trait RenderPrimitive {
    /// Render self into the given mesh.
    fn render_primitive(self, ctxt: RenderContext, mesh: &mut Mesh) -> PrimitiveRender;
}

/// Information about the way in which a primitive was rendered.
pub struct PrimitiveRender {
    /// Whether or not a specific texture must be available when this primitive is drawn.
    ///
    /// If `Some` and the given texture is different than the currently set texture, a render
    /// command will be encoded that switches from the previous texture's bind group to the new
    /// one.
    pub texture_handle: Option<Handle<Image>>,
}

/// The context provided to primitives to assist with the rendering process.
pub struct RenderContext<'a> {
    pub transform: &'a Mat4,
    pub intermediary_mesh: &'a Mesh,
    pub path_event_buffer: &'a [PathEvent],
    pub path_points_colored_buffer: &'a [(Vec2, Color)],
    pub path_points_textured_buffer: &'a [(Vec2, Vec2)],
    pub text_buffer: &'a str,
    pub theme: &'a draw::Theme,
    pub glyph_cache: &'a mut GlyphCache,
    pub fill_tessellator: &'a mut FillTessellator,
    pub stroke_tessellator: &'a mut StrokeTessellator,
    pub output_attachment_size: Vec2, // logical coords
    pub output_attachment_scale_factor: f32,
}

#[derive(Resource)]
pub struct GlyphCache {
    /// Tracks glyphs and their location within the cache.
    pub cache: text::GlyphCache<'static>,
    /// The buffer used to store the pixels of the glyphs.
    pub pixel_buffer: Vec<u8>,
    /// Will be set to `true` after the cache has been updated if the texture requires re-uploading.
    pub requires_upload: bool,
}

impl Deref for GlyphCache {
    type Target = text::GlyphCache<'static>;
    fn deref(&self) -> &Self::Target {
        &self.cache
    }
}

impl DerefMut for GlyphCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cache
    }
}

/// The position and dimensions of the scissor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scissor {
    pub left: u32,
    pub bottom: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for PrimitiveRender {
    fn default() -> Self {
        PrimitiveRender { texture_handle: None }
    }
}

impl RenderPrimitive for draw::Primitive {
    fn render_primitive(self, ctxt: RenderContext, mesh: &mut Mesh) -> PrimitiveRender {
        match self {
            draw::Primitive::Arrow(prim) => prim.render_primitive(ctxt, mesh),
            draw::Primitive::Mesh(prim) => prim.render_primitive(ctxt, mesh),
            draw::Primitive::Path(prim) => prim.render_primitive(ctxt, mesh),
            draw::Primitive::Polygon(prim) => prim.render_primitive(ctxt, mesh),
            draw::Primitive::Tri(prim) => prim.render_primitive(ctxt, mesh),
            draw::Primitive::Ellipse(prim) => prim.render_primitive(ctxt, mesh),
            draw::Primitive::Quad(prim) => prim.render_primitive(ctxt, mesh),
            draw::Primitive::Rect(prim) => prim.render_primitive(ctxt, mesh),
            draw::Primitive::Line(prim) => prim.render_primitive(ctxt, mesh),
            draw::Primitive::Text(prim) => prim.render_primitive(ctxt, mesh),
            draw::Primitive::Texture(prim) => prim.render_primitive(ctxt, mesh),
            _ => PrimitiveRender::default(),
        }
    }
}

impl fmt::Debug for GlyphCache {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("GlyphCache")
            .field("cache", &self.cache.dimensions())
            .field("pixel_buffer", &self.pixel_buffer.len())
            .field("requires_upload", &self.requires_upload)
            .finish()
    }
}

impl PrimitiveRender {
    pub fn texture(texture_handle: Handle<Image>) -> Self {
        PrimitiveRender {
            texture_handle: Some(texture_handle),
        }
    }
}

impl GlyphCache {
    pub fn new(size: [u32; 2], scale_tolerance: f32, position_tolerance: f32) -> Self {
        let [w, h] = size;
        let cache = text::GlyphCache::builder()
            .dimensions(w, h)
            .scale_tolerance(scale_tolerance)
            .position_tolerance(position_tolerance)
            .build()
            .into();
        let pixel_buffer = vec![0u8; w as usize * h as usize];
        let requires_upload = false;
        GlyphCache {
            cache,
            pixel_buffer,
            requires_upload,
        }
    }
}

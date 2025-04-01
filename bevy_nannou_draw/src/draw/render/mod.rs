use bevy::prelude::*;
use lyon::{
    path::PathEvent,
    tessellation::{FillTessellator, StrokeTessellator},
};

use crate::draw;

/// Draw API primitives that may be rendered via the **Renderer** type.
pub trait RenderPrimitive {
    /// Render self into the given mesh.
    fn render_primitive(self, ctxt: RenderContext, mesh: &mut Mesh);
}

/// The context provided to primitives to assist with the rendering process.
pub struct RenderContext<'a> {
    pub transform: &'a Mat4,
    pub intermediary_mesh: &'a Mesh,
    pub path_event_buffer: &'a [PathEvent],
    pub path_points_vertex_buffer: &'a [(Vec2, Color, Vec2)],
    pub text_buffer: &'a str,
    pub theme: &'a draw::Theme,
    pub fill_tessellator: &'a mut FillTessellator,
    pub stroke_tessellator: &'a mut StrokeTessellator,
    pub output_attachment_size: Vec2, // logical coords
    pub output_attachment_scale_factor: f32,
}

/// The position and dimensions of the scissor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scissor {
    pub left: u32,
    pub bottom: u32,
    pub width: u32,
    pub height: u32,
}

impl RenderPrimitive for draw::Primitive {
    fn render_primitive(self, ctxt: RenderContext, mesh: &mut Mesh) {
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
            _ => {}
        }
    }
}

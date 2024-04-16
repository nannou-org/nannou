use crate::draw::primitive::path;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{self, dimension, orientation, position};
use crate::draw::properties::{SetDimensions, SetOrientation, SetPosition};
use crate::draw::{self, Drawing};
use bevy::prelude::*;
use nannou_core::geom;

/// Properties related to drawing a **Rect**.
#[derive(Clone, Debug)]
pub struct Texture<M :Material> {
    texture_handle: Handle<Image>,
    spatial: spatial::Properties,
    area: geom::Rect,
    material: M,
}

/// The drawing context for a Rect.
pub type DrawingTexture<'a, M: Material> = Drawing<'a, Texture<M>>;

// Trait implementations.

impl <M: Material> Texture<M> {
    pub(crate) fn new(texture_handle: Handle<Image>, texture: Image) -> Self {
        let w = texture.width() as f32;
        let h = texture.height() as f32;
        let spatial = spatial::Properties::default().w_h(w, h);
        let x = geom::Range {
            start: 0.0,
            end: 1.0,
        };
        let y = geom::Range {
            start: 0.0,
            end: 1.0,
        };
        let area = geom::Rect { x, y };
        Self {
            texture_handle,
            spatial,
            area,
        }
    }
}

impl <M: Material> Texture<M> {
    /// Specify the area of the texture to draw.
    ///
    /// The bounds of the rectangle should represent the desired area as texture coordinates of the
    /// underlying texture.
    ///
    /// Texture coordinates range from (0.0, 0.0) in the bottom left of the texture, to (1.0, 1.0)
    /// in the top right of the texture.
    ///
    /// By default, the area represents the full extent of the texture.
    pub fn area(mut self, rect: geom::Rect) -> Self {
        self.area = rect;
        self
    }
}

impl<'a, M: Material> DrawingTexture<'a, M> {
    /// Specify the area of the texture to draw.
    ///
    /// The bounds of the rectangle should represent the desired area as texture coordinates of the
    /// underlying texture.
    ///
    /// Texture coordinates range from (0.0, 0.0) in the bottom left of the texture, to (1.0, 1.0)
    /// in the top right of the texture.
    ///
    /// By default, the area represents the full extent of the texture.
    pub fn area(self, rect: geom::Rect) -> Self {
        self.map_ty(|ty| ty.area(rect))
    }
}

impl <M: Material> draw::render::RenderPrimitive for Texture<M> {
    fn render_primitive(
        self,
        mut ctxt: draw::render::RenderContext,
        mesh: &mut Mesh,
    ) -> draw::render::PrimitiveRender {
        let Texture {
            texture_handle,
            spatial,
            area,
        } = self;
        let spatial::Properties {
            dimensions,
            position,
            orientation,
        } = spatial;

        // If dimensions were specified, scale the points to those dimensions.
        let (maybe_x, maybe_y, maybe_z) = (dimensions.x, dimensions.y, dimensions.z);
        assert!(
            maybe_z.is_none(),
            "z dimension support for rect is unimplemented"
        );
        let w = maybe_x.unwrap_or(100.0);
        let h = maybe_y.unwrap_or(100.0);
        let rect = geom::Rect::from_w_h(w, h);

        // Determine the transform to apply to all points.
        let global_transform = *ctxt.transform;
        let local_transform = position.transform() * orientation.transform();
        let transform = global_transform * local_transform;

        // Create an iterator yielding texture points.
        let points_textured = rect
            .corners()
            .vertices()
            .map(Vec2::from)
            .zip(area.invert_y().corners().vertices().map(Vec2::from));

        path::render_path_points_textured(
            points_textured,
            true,
            transform,
            path::Options::Fill(Default::default()),
            &mut ctxt.fill_tessellator,
            &mut ctxt.stroke_tessellator,
            mesh,
        );

        draw::render::PrimitiveRender::texture(texture_handle)
    }
}

impl <M: Material> SetOrientation for Texture<M> {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.spatial)
    }
}

impl <M: Material> SetPosition for Texture<M> {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.spatial)
    }
}

impl <M: Material> SetDimensions for Texture<M> {
    fn properties(&mut self) -> &mut dimension::Properties {
        SetDimensions::properties(&mut self.spatial)
    }
}

// Primitive conversions.

impl <M: Material> From<Texture<M>> for Primitive {
    fn from(prim: Texture<M>) -> Self {
        Primitive::Texture(prim)
    }
}

impl <M: Material> Into<Option<Texture<M>>> for Primitive {
    fn into(self) -> Option<Texture<M>> {
        match self {
            Primitive::Texture(prim) => Some(prim),
            _ => None,
        }
    }
}

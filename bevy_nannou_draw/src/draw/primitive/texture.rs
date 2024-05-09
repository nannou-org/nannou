use bevy::prelude::*;

use nannou_core::geom;

use crate::draw::primitive::path;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{self, dimension, orientation, position};
use crate::draw::properties::{SetDimensions, SetOrientation, SetPosition};
use crate::draw::{self, Drawing};

/// Properties related to drawing a **Rect**.
#[derive(Clone, Debug)]
pub struct Texture {
    texture_handle: Handle<Image>,
    spatial: spatial::Properties,
    area: geom::Rect,
}

/// The drawing context for a Rect.
pub type DrawingTexture<'a, M> = Drawing<'a, Texture, M>;

// Trait implementations.

impl Texture {
    pub(crate) fn new(texture_handle: Handle<Image>, (w, h): (f32, f32)) -> Self {
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

impl Texture {
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

impl<'a, M> DrawingTexture<'a, M>
where
    M: Material + Default,
{
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

impl draw::render::RenderPrimitive for Texture {
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

impl SetOrientation for Texture {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.spatial)
    }
}

impl SetPosition for Texture {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.spatial)
    }
}

impl SetDimensions for Texture {
    fn properties(&mut self) -> &mut dimension::Properties {
        SetDimensions::properties(&mut self.spatial)
    }
}

// Primitive conversions.

impl From<Texture> for Primitive {
    fn from(prim: Texture) -> Self {
        Primitive::Texture(prim)
    }
}

impl Into<Option<Texture>> for Primitive {
    fn into(self) -> Option<Texture> {
        match self {
            Primitive::Texture(prim) => Some(prim),
            _ => None,
        }
    }
}

use crate::draw::primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};
use bevy::prelude::*;
use lyon::tessellation::StrokeOptions;
use nannou_core::geom;
use crate::draw::properties::material::SetMaterial;
use crate::render::NannouMaterialOptions;

/// Properties related to drawing a **Rect**.
#[derive(Clone, Debug)]
pub struct Rect<M :Material> {
    dimensions: dimension::Properties,
    polygon: PolygonInit<M>,
    material: M
}

impl <M: Material> SetMaterial<M> for Rect<M> {
    fn material_mut(&mut self) -> &mut M {
        &mut self.material
    }
}

/// The drawing context for a Rect.
pub type DrawingRect<'a,M : Material> = Drawing<'a, Rect<M>>;

// Trait implementations.

impl <M: Material> Rect<M> {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.stroke_color(color)
    }
}

impl<'a, M : Material> DrawingRect<'a, M> {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }
}

impl <M: Material> draw::render::RenderPrimitive for Rect<M> {
    fn render_primitive(
        self,
        ctxt: draw::render::RenderContext,
        mesh: &mut Mesh,
    ) -> draw::render::PrimitiveRender {
        let Rect {
            polygon,
            dimensions,
            material,
        } = self;

        // If dimensions were specified, scale the points to those dimensions.
        let (maybe_x, maybe_y, maybe_z) = (dimensions.x, dimensions.y, dimensions.z);
        assert!(
            maybe_z.is_none(),
            "z dimension support for rect is unimplemented"
        );
        let w = maybe_x.unwrap_or(100.0);
        let h = maybe_y.unwrap_or(100.0);
        let rect = geom::Rect::from_wh([w, h].into());
        let points = rect.corners().vertices().map(Vec2::from);
        polygon::render_points_themed(
            polygon.opts,
            points,
            ctxt,
            &draw::theme::Primitive::Rect,
            mesh,
        );

        draw::render::PrimitiveRender::default()
    }
}

impl <M: Material> From<geom::Rect<f32>> for Rect<M> {
    fn from(r: geom::Rect<f32>) -> Self {
        let (x, y, w, h) = r.x_y_w_h();
        Self::default().x_y(x, y).w_h(w, h)
    }
}

impl <M: Material> Default for Rect<M> {
    fn default() -> Self {
        let dimensions = <_>::default();
        let polygon = <_>::default();
        Rect {
            dimensions,
            polygon,
            material: NannouMaterialOptions::default(),
        }
    }
}

impl <M: Material> SetOrientation for Rect<M> {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl <M: Material> SetPosition for Rect<M> {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.polygon)
    }
}

impl <M: Material> SetDimensions for Rect<M> {
    fn properties(&mut self) -> &mut dimension::Properties {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl <M: Material> SetColor for Rect<M> {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.polygon)
    }
}

impl <M: Material> SetStroke for Rect<M> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl <M: Material> SetPolygon for Rect<M> {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

// Primitive conversions.

impl <M: Material> From<Rect<M>> for Primitive {
    fn from(prim: Rect<M>) -> Self {
        Primitive::Rect(prim)
    }
}

impl <M: Material> Into<Option<Rect<M>>> for Primitive {
    fn into(self) -> Option<Rect<M>> {
        match self {
            Primitive::Rect(prim) => Some(prim),
            _ => None,
        }
    }
}

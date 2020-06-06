use crate::color::conv::IntoLinSrgba;
use crate::draw::primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{
    ColorScalar, LinSrgba, SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Drawing};
use crate::geom::{self, Vector2};
use crate::math::BaseFloat;
use lyon::tessellation::StrokeOptions;

/// Properties related to drawing a **Rect**.
#[derive(Clone, Debug)]
pub struct Rect<S = geom::scalar::Default> {
    dimensions: dimension::Properties<S>,
    polygon: PolygonInit<S>,
}

/// The drawing context for a Rect.
pub type DrawingRect<'a, S = geom::scalar::Default> = Drawing<'a, Rect<S>, S>;

// Trait implementations.

impl<S> Rect<S> {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.stroke_color(color)
    }
}

impl<'a, S> DrawingRect<'a, S>
where
    S: BaseFloat,
{
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }
}

impl draw::renderer::RenderPrimitive for Rect<f32> {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender {
        let Rect {
            polygon,
            dimensions,
        } = self;

        // If dimensions were specified, scale the points to those dimensions.
        let (maybe_x, maybe_y, maybe_z) = (dimensions.x, dimensions.y, dimensions.z);
        assert!(
            maybe_z.is_none(),
            "z dimension support for rect is unimplemented"
        );
        let w = maybe_x.unwrap_or(100.0);
        let h = maybe_y.unwrap_or(100.0);
        let rect = geom::Rect::from_wh(Vector2 { x: w, y: h });
        let points = rect.corners().vertices();
        polygon::render_points_themed(
            polygon.opts,
            points,
            ctxt,
            &draw::theme::Primitive::Rect,
            mesh,
        );

        draw::renderer::PrimitiveRender::default()
    }
}

impl<S> From<geom::Rect<S>> for Rect<S>
where
    S: BaseFloat,
{
    fn from(r: geom::Rect<S>) -> Self {
        let (x, y, w, h) = r.x_y_w_h();
        Self::default().x_y(x, y).w_h(w, h)
    }
}

impl<S> Default for Rect<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        let dimensions = <_>::default();
        let polygon = <_>::default();
        Rect {
            dimensions,
            polygon,
        }
    }
}

impl<S> SetOrientation<S> for Rect<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl<S> SetPosition<S> for Rect<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.polygon)
    }
}

impl<S> SetDimensions<S> for Rect<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl<S> SetColor<ColorScalar> for Rect<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.polygon)
    }
}

impl<S> SetStroke for Rect<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl<S> SetPolygon<S> for Rect<S> {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions<S> {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

// Primitive conversions.

impl<S> From<Rect<S>> for Primitive<S> {
    fn from(prim: Rect<S>) -> Self {
        Primitive::Rect(prim)
    }
}

impl<S> Into<Option<Rect<S>>> for Primitive<S> {
    fn into(self) -> Option<Rect<S>> {
        match self {
            Primitive::Rect(prim) => Some(prim),
            _ => None,
        }
    }
}

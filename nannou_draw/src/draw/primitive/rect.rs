use bevy::prelude::*;
use lyon::tessellation::StrokeOptions;

use nannou_core::geom;

use crate::draw::primitive::Primitive;
use crate::draw::primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon};
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::tex_coords::SetTexCoords;
use crate::draw::properties::{SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke};
use crate::draw::{self, Drawing};
use crate::render::ShaderModel;

/// Properties related to drawing a **Rect**.
#[derive(Clone, Debug)]
pub struct Rect {
    dimensions: dimension::Properties,
    tex_coords: Option<geom::Rect>,
    polygon: PolygonInit,
}

/// The drawing context for a Rect.
pub type DrawingRect<'a, SM> = Drawing<'a, Rect, SM>;

// Trait implementations.

impl Rect {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.stroke_color(color)
    }
}

impl<'a, SM> DrawingRect<'a, SM>
where
    SM: ShaderModel + Default,
{
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    pub fn area(self, area: geom::Rect) -> Self {
        self.map_ty(|ty| ty.area(area))
    }
}

impl draw::render::RenderPrimitive for Rect {
    fn render_primitive(self, ctxt: draw::render::RenderContext, mesh: &mut Mesh) {
        let Rect {
            tex_coords,
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
        let rect = geom::Rect::from_wh([w, h].into());

        let tex_coords = tex_coords
            .map(|area| {
                [
                    area.bottom_left(),
                    area.bottom_right(),
                    area.top_right(),
                    area.top_left(),
                ]
            })
            .unwrap_or([
                Vec2::new(0.0, 0.0), // Bottom-left
                Vec2::new(1.0, 0.0), // Bottom-right
                Vec2::new(1.0, 1.0), // Top-right
                Vec2::new(0.0, 1.0), // Top-left
            ]);

        let points = rect
            .corners()
            .vertices()
            .map(Vec2::from)
            .zip(tex_coords.iter().copied());

        polygon::render_points_themed(
            polygon.opts,
            true,
            points,
            ctxt,
            &draw::theme::Primitive::Rect,
            mesh,
        );
    }
}

impl From<geom::Rect<f32>> for Rect {
    fn from(r: geom::Rect<f32>) -> Self {
        let (x, y, w, h) = r.x_y_w_h();
        Self::default().x_y(x, y).w_h(w, h)
    }
}

impl Default for Rect {
    fn default() -> Self {
        let dimensions = <_>::default();
        let polygon = <_>::default();
        Rect {
            dimensions,
            tex_coords: None,
            polygon,
        }
    }
}

impl SetOrientation for Rect {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl SetPosition for Rect {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.polygon)
    }
}

impl SetDimensions for Rect {
    fn properties(&mut self) -> &mut dimension::Properties {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl SetColor for Rect {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.polygon)
    }
}

impl SetStroke for Rect {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl SetPolygon for Rect {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

impl SetTexCoords for Rect {
    fn tex_coords_mut(&mut self) -> &mut Option<geom::Rect> {
        SetTexCoords::tex_coords_mut(&mut self.tex_coords)
    }
}

// Primitive conversions.

impl From<Rect> for Primitive {
    fn from(prim: Rect) -> Self {
        Primitive::Rect(prim)
    }
}

impl Into<Option<Rect>> for Primitive {
    fn into(self) -> Option<Rect> {
        match self {
            Primitive::Rect(prim) => Some(prim),
            _ => None,
        }
    }
}

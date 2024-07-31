use bevy::prelude::*;
use lyon::tessellation::StrokeOptions;

use nannou_core::geom;

use crate::draw::primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{
    spatial, SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Drawing};

/// Properties related to drawing a **Quad**.
#[derive(Clone, Debug)]
pub struct Quad {
    quad: geom::Quad<Vec2>,
    polygon: PolygonInit,
    dimensions: spatial::dimension::Properties,
}

/// The drawing context for a `Quad`.
pub type DrawingQuad<'a, M> = Drawing<'a, Quad, M>;

// Quad-specific methods.

impl Quad {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.stroke_color(color)
    }

    /// Use the given four points as the vertices (corners) of the quad.
    pub fn points<P>(mut self, a: P, b: P, c: P, d: P) -> Self
    where
        P: Into<Vec2>,
    {
        let a = a.into();
        let b = b.into();
        let c = c.into();
        let d = d.into();
        self.quad = geom::Quad([a, b, c, d]);
        self
    }
}

// Trait implementations.
impl draw::render::RenderPrimitive for Quad {
    fn render_primitive(self, ctxt: draw::render::RenderContext, mesh: &mut Mesh) {
        let Quad {
            mut quad,
            polygon,
            dimensions,
        } = self;

        // If dimensions were specified, scale the points to those dimensions.
        let (maybe_x, maybe_y, _maybe_z) = (dimensions.x, dimensions.y, dimensions.z);
        if maybe_x.is_some() || maybe_y.is_some() {
            let cuboid = quad.bounding_rect();
            let centroid = quad.centroid();
            let x_scale = maybe_x.map(|x| x / cuboid.w()).unwrap_or(1.0);
            let y_scale = maybe_y.map(|y| y / cuboid.h()).unwrap_or(1.0);
            let scale = Vec2::new(x_scale, y_scale);
            let (a, b, c, d) = quad.into();
            let translate = |v: Vec2| centroid + ((v - centroid) * scale);
            let new_a = translate(a);
            let new_b = translate(b);
            let new_c = translate(c);
            let new_d = translate(d);
            quad = geom::Quad([new_a, new_b, new_c, new_d]);
        }

        let tex_coords = [
            Vec2::new(0.0, 0.0), // Bottom-left
            Vec2::new(1.0, 0.0), // Bottom-right
            Vec2::new(1.0, 1.0), // Top-right
            Vec2::new(0.0, 1.0), // Top-left
        ];

        let points = quad.vertices().zip(tex_coords.iter().copied());
        polygon::render_points_themed(
            polygon.opts,
            true,
            points,
            ctxt,
            &draw::theme::Primitive::Quad,
            mesh,
        );
    }
}

impl From<geom::Quad<Vec2>> for Quad {
    fn from(quad: geom::Quad<Vec2>) -> Self {
        let polygon = Default::default();
        let dimensions = Default::default();
        Quad {
            polygon,
            dimensions,
            quad,
        }
    }
}

impl Default for Quad {
    fn default() -> Self {
        // Create a quad pointing towards 0.0 radians.
        let fifty = 50.0;
        let left = -fifty;
        let bottom = -fifty;
        let right = fifty;
        let top = fifty;
        let a = Vec2::new(left, bottom);
        let b = Vec2::new(left, top);
        let c = Vec2::new(right, top);
        let d = Vec2::new(right, bottom);
        Quad::from(geom::Quad([a, b, c, d]))
    }
}

impl SetOrientation for Quad {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl SetPosition for Quad {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.polygon)
    }
}

impl SetDimensions for Quad {
    fn properties(&mut self) -> &mut dimension::Properties {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl SetColor for Quad {
    fn color_mut(&mut self) -> &mut Option<Color> {
        SetColor::color_mut(&mut self.polygon)
    }
}

impl SetStroke for Quad {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl SetPolygon for Quad {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

// Primitive conversions.

impl From<Quad> for Primitive {
    fn from(prim: Quad) -> Self {
        Primitive::Quad(prim)
    }
}

impl Into<Option<Quad>> for Primitive {
    fn into(self) -> Option<Quad> {
        match self {
            Primitive::Quad(prim) => Some(prim),
            _ => None,
        }
    }
}

// Drawing methods.

impl<'a, M> DrawingQuad<'a, M>
where
    M: Material + Default,
{
    /// Use the given points as the vertices (corners) of the quad.
    pub fn points<P>(self, a: P, b: P, c: P, d: P) -> Self
    where
        P: Into<Vec2>,
    {
        self.map_ty(|ty| ty.points(a, b, c, d))
    }
}

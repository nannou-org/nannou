use crate::color::conv::IntoLinSrgba;
use crate::draw::primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{
    ColorScalar, LinSrgba, SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Drawing};
use crate::geom::{self, pt2, Point2};
use crate::glam::vec2;
use lyon::tessellation::StrokeOptions;

/// Properties related to drawing a **Tri**.
#[derive(Clone, Debug)]
pub struct Tri {
    tri: geom::Tri<Point2>,
    dimensions: dimension::Properties,
    polygon: PolygonInit,
}

/// The drawing context for a `Tri`.
pub type DrawingTri<'a> = Drawing<'a, Tri>;

// Tri-specific methods.

impl Tri {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.stroke_color(color)
    }

    /// Use the given three points as the vertices (corners) of the triangle.
    pub fn points<P>(mut self, a: P, b: P, c: P) -> Self
    where
        P: Into<Point2>,
    {
        let a = a.into();
        let b = b.into();
        let c = c.into();
        self.tri = geom::Tri([a, b, c]);
        self
    }
}

// Drawing methods.

impl<'a> DrawingTri<'a> {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.map_ty(|ty| ty.stroke(color))
    }

    /// Use the given points as the vertices (corners) of the triangle.
    pub fn points<P>(self, a: P, b: P, c: P) -> Self
    where
        P: Into<Point2>,
    {
        self.map_ty(|ty| ty.points(a, b, c))
    }
}

// Trait implementations.

impl draw::renderer::RenderPrimitive for Tri {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender {
        let Tri {
            mut tri,
            dimensions,
            polygon,
        } = self;
        let (maybe_x, maybe_y, _maybe_z) = (dimensions.x, dimensions.y, dimensions.z);
        // If dimensions were specified, scale the points to those dimensions.
        if maybe_x.is_some() || maybe_y.is_some() {
            let cuboid = tri.bounding_rect();
            let centroid = tri.centroid();
            let x_scale = maybe_x.map(|x| x / cuboid.w()).unwrap_or(1.0);
            let y_scale = maybe_y.map(|y| y / cuboid.h()).unwrap_or(1.0);
            let scale = vec2(x_scale, y_scale);
            let (a, b, c) = tri.into();
            let translate = |v: Point2| centroid + ((v - centroid) * scale);
            let new_a = translate(a);
            let new_b = translate(b);
            let new_c = translate(c);
            tri = geom::Tri([new_a, new_b, new_c]);
        }
        let points = tri.vertices();
        polygon::render_points_themed(
            polygon.opts,
            points,
            ctxt,
            &draw::theme::Primitive::Tri,
            mesh,
        );

        draw::renderer::PrimitiveRender::default()
    }
}

impl From<geom::Tri<Point2>> for Tri {
    fn from(tri: geom::Tri<Point2>) -> Self {
        let dimensions = <_>::default();
        let polygon = <_>::default();
        Tri {
            tri,
            dimensions,
            polygon,
        }
    }
}

impl Default for Tri {
    fn default() -> Self {
        // Create a triangle pointing towards 0.0 radians.
        let fifty = 50.0;
        let thirty_three = 33.0;
        let a = pt2(-fifty, thirty_three);
        let b = pt2(fifty, 0.0);
        let c = pt2(-fifty, -thirty_three);
        Tri::from(geom::Tri([a, b, c]))
    }
}

impl SetOrientation for Tri {
    fn properties(&mut self) -> &mut orientation::Properties {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl SetPosition for Tri {
    fn properties(&mut self) -> &mut position::Properties {
        SetPosition::properties(&mut self.polygon)
    }
}

impl SetDimensions for Tri {
    fn properties(&mut self) -> &mut dimension::Properties {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl SetColor<ColorScalar> for Tri {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.polygon)
    }
}

impl SetStroke for Tri {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl SetPolygon for Tri {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

// Primitive conversions.

impl From<Tri> for Primitive {
    fn from(prim: Tri) -> Self {
        Primitive::Tri(prim)
    }
}

impl Into<Option<Tri>> for Primitive {
    fn into(self) -> Option<Tri> {
        match self {
            Primitive::Tri(prim) => Some(prim),
            _ => None,
        }
    }
}

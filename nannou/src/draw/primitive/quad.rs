use crate::color::conv::IntoLinSrgba;
use crate::draw::primitive::polygon::{self, PolygonInit, PolygonOptions, SetPolygon};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{
    spatial, ColorScalar, LinSrgba, SetColor, SetDimensions, SetOrientation, SetPosition, SetStroke,
};
use crate::draw::{self, Drawing};
use crate::geom::{self, Point2, Vector2};
use crate::math::{BaseFloat, ElementWise};
use lyon::tessellation::StrokeOptions;

/// Properties related to drawing a **Quad**.
#[derive(Clone, Debug)]
pub struct Quad<S = geom::scalar::Default> {
    quad: geom::Quad<Point2<S>>,
    polygon: PolygonInit<S>,
    dimensions: spatial::dimension::Properties<S>,
}

/// The drawing context for a `Quad`.
pub type DrawingQuad<'a, S = geom::scalar::Default> = Drawing<'a, Quad<S>, S>;

// Quad-specific methods.

impl<S> Quad<S> {
    /// Stroke the outline with the given color.
    pub fn stroke<C>(self, color: C) -> Self
    where
        C: IntoLinSrgba<ColorScalar>,
    {
        self.stroke_color(color)
    }

    /// Use the given four points as the vertices (corners) of the quad.
    pub fn points<P>(mut self, a: P, b: P, c: P, d: P) -> Self
    where
        P: Into<Point2<S>>,
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
impl<'r> draw::renderer::RenderPrimitive<'r> for Quad<f32> {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::PrimitiveRender<'r> {
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
            let scale = Vector2 {
                x: x_scale,
                y: y_scale,
            };
            let (a, b, c, d) = quad.into();
            let translate = |v: Point2| centroid + ((v - centroid).mul_element_wise(scale));
            let new_a = translate(a);
            let new_b = translate(b);
            let new_c = translate(c);
            let new_d = translate(d);
            quad = geom::Quad([new_a, new_b, new_c, new_d]);
        }

        let points = quad.vertices();
        polygon::render_points_themed(
            polygon.opts,
            points,
            ctxt,
            &draw::theme::Primitive::Quad,
            mesh,
        );

        draw::renderer::PrimitiveRender::default()
    }
}

impl<S> From<geom::Quad<Point2<S>>> for Quad<S>
where
    S: BaseFloat,
{
    fn from(quad: geom::Quad<Point2<S>>) -> Self {
        let polygon = Default::default();
        let dimensions = Default::default();
        Quad {
            polygon,
            dimensions,
            quad,
        }
    }
}

impl<S> Default for Quad<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        // Create a quad pointing towards 0.0 radians.
        let fifty = S::from(50.0).unwrap();
        let left = -fifty;
        let bottom = -fifty;
        let right = fifty;
        let top = fifty;
        let a = Point2 { x: left, y: bottom };
        let b = Point2 { x: left, y: top };
        let c = Point2 { x: right, y: top };
        let d = Point2 {
            x: right,
            y: bottom,
        };
        Quad::from(geom::Quad([a, b, c, d]))
    }
}

impl<S> SetOrientation<S> for Quad<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl<S> SetPosition<S> for Quad<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.polygon)
    }
}

impl<S> SetDimensions<S> for Quad<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl<S> SetColor<ColorScalar> for Quad<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.polygon)
    }
}

impl<S> SetStroke for Quad<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl<S> SetPolygon<S> for Quad<S> {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions<S> {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

// Primitive conversions.

impl<'q, S> From<Quad<S>> for Primitive<'q, S> {
    fn from(prim: Quad<S>) -> Self {
        Primitive::Quad(prim)
    }
}

impl<'q, S> Into<Option<Quad<S>>> for Primitive<'q, S> {
    fn into(self) -> Option<Quad<S>> {
        match self {
            Primitive::Quad(prim) => Some(prim),
            _ => None,
        }
    }
}

// Drawing methods.

impl<'a, S> DrawingQuad<'a, S>
where
    S: BaseFloat,
{
    /// Use the given points as the vertices (corners) of the quad.
    pub fn points<P>(self, a: P, b: P, c: P, d: P) -> Self
    where
        P: Into<Point2<S>>,
    {
        self.map_ty(|ty| ty.points(a, b, c, d))
    }
}

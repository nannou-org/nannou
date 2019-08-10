use crate::color::conv::IntoLinSrgba;
use crate::draw::primitive::polygon::{
    PolygonIndices, PolygonInit, PolygonOptions, PolygonVertices, SetPolygon,
};
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{
    ColorScalar, Draw, Drawn, IntoDrawn, LinSrgba, SetColor, SetDimensions, SetOrientation,
    SetPosition, SetStroke,
};
use crate::draw::{theme, Drawing};
use crate::geom::{self, Point2, Vector2};
use crate::math::{BaseFloat, ElementWise};
use lyon::tessellation::StrokeOptions;

/// Properties related to drawing a **Tri**.
#[derive(Clone, Debug)]
pub struct Tri<S = geom::scalar::Default> {
    tri: geom::Tri<Point2<S>>,
    dimensions: dimension::Properties<S>,
    polygon: PolygonInit<S>,
}

/// The drawing context for a `Tri`.
pub type DrawingTri<'a, S = geom::scalar::Default> = Drawing<'a, Tri<S>, S>;

// Tri-specific methods.

impl<S> Tri<S> {
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
        P: Into<Point2<S>>,
    {
        let a = a.into();
        let b = b.into();
        let c = c.into();
        self.tri = geom::Tri([a, b, c]);
        self
    }
}

// Drawing methods.

impl<'a, S> DrawingTri<'a, S>
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

    /// Use the given points as the vertices (corners) of the triangle.
    pub fn points<P>(self, a: P, b: P, c: P) -> Self
    where
        P: Into<Point2<S>>,
    {
        self.map_ty(|ty| ty.points(a, b, c))
    }
}

// Trait implementations.

impl<S> IntoDrawn<S> for Tri<S>
where
    S: BaseFloat,
{
    type Vertices = PolygonVertices;
    type Indices = PolygonIndices;
    fn into_drawn(self, mut draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Tri {
            mut tri,
            dimensions,
            polygon,
        } = self;

        let (maybe_x, maybe_y, _maybe_z) = dimensions.to_scalars(&draw);

        // If dimensions were specified, scale the points to those dimensions.
        if maybe_x.is_some() || maybe_y.is_some() {
            let cuboid = tri.bounding_rect();
            let centroid = tri.centroid();
            let x_scale = maybe_x.map(|x| x / cuboid.w()).unwrap_or_else(S::one);
            let y_scale = maybe_y.map(|y| y / cuboid.h()).unwrap_or_else(S::one);
            let scale = Vector2 {
                x: x_scale,
                y: y_scale,
            };
            let (a, b, c) = tri.into();
            let translate = |v: Point2<S>| centroid + ((v - centroid).mul_element_wise(scale));
            let new_a = translate(a);
            let new_b = translate(b);
            let new_c = translate(c);
            tri = geom::Tri([new_a, new_b, new_c]);
        }

        // The color.
        let points = tri.vertices();
        let polygon = draw.drawing_context(|ctxt| polygon.points(ctxt, points));
        polygon.into_drawn_themed(draw, &theme::Primitive::Tri)
    }
}

impl<S> From<geom::Tri<Point2<S>>> for Tri<S>
where
    S: BaseFloat,
{
    fn from(tri: geom::Tri<Point2<S>>) -> Self {
        let dimensions = <_>::default();
        let polygon = <_>::default();
        Tri {
            tri,
            dimensions,
            polygon,
        }
    }
}

impl<S> Default for Tri<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        // Create a triangle pointing towards 0.0 radians.
        let zero = S::zero();
        let fifty = S::from(50.0).unwrap();
        let thirty_three = S::from(33.0).unwrap();
        let a = Point2 {
            x: -fifty,
            y: thirty_three,
        };
        let b = Point2 { x: fifty, y: zero };
        let c = Point2 {
            x: -fifty,
            y: -thirty_three,
        };
        Tri::from(geom::Tri([a, b, c]))
    }
}

impl<S> SetOrientation<S> for Tri<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.polygon)
    }
}

impl<S> SetPosition<S> for Tri<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.polygon)
    }
}

impl<S> SetDimensions<S> for Tri<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.dimensions)
    }
}

impl<S> SetColor<ColorScalar> for Tri<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.polygon)
    }
}

impl<S> SetStroke for Tri<S> {
    fn stroke_options_mut(&mut self) -> &mut StrokeOptions {
        SetStroke::stroke_options_mut(&mut self.polygon)
    }
}

impl<S> SetPolygon<S> for Tri<S> {
    fn polygon_options_mut(&mut self) -> &mut PolygonOptions<S> {
        SetPolygon::polygon_options_mut(&mut self.polygon)
    }
}

// Primitive conversions.

impl<S> From<Tri<S>> for Primitive<S> {
    fn from(prim: Tri<S>) -> Self {
        Primitive::Tri(prim)
    }
}

impl<S> Into<Option<Tri<S>>> for Primitive<S> {
    fn into(self) -> Option<Tri<S>> {
        match self {
            Primitive::Tri(prim) => Some(prim),
            _ => None,
        }
    }
}

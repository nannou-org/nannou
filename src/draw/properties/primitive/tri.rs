use draw::{self, Drawing};
use draw::mesh::vertex::IntoPoint;
use draw::properties::{spatial, ColorScalar, Draw, Drawn, IntoDrawn, Primitive, Rgba, SetColor, SetDimensions, SetPosition};
use draw::properties::spatial::{dimension, position};
use geom;
use math::{BaseFloat, ElementWise, Point3, Vector3};
use std::ops;

/// Properties related to drawing a **Tri**.
#[derive(Clone, Debug)]
pub struct Tri<S = geom::DefaultScalar> {
    tri: geom::Tri<Point3<S>>,
    spatial: spatial::Properties<S>,
    color: Option<Rgba>,
}

// Tri-specific methods.

impl<S> Tri<S> {
    /// Use the given three points as the vertices (corners) of the triangle.
    pub fn points<P>(mut self, a: P, b: P, c: P) -> Self
    where
        P: IntoPoint<S>,
    {
        let a = a.into_point();
        let b = b.into_point();
        let c = c.into_point();
        self.tri = geom::Tri([a, b, c]);
        self
    }
}

// Trait implementations.

impl<S> IntoDrawn<S> for Tri<S>
where
    S: BaseFloat,
{
    type Vertices = draw::mesh::vertex::IterFromPoints<geom::tri::Vertices<Point3<S>>, S>;
    type Indices = ops::Range<usize>;
    fn into_drawn(self, draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Tri {
            mut tri,
            spatial,
            color,
        } = self;

        let (maybe_x, maybe_y, maybe_z) = spatial.dimensions.to_scalars(&draw);

        // If dimensions were specified, scale the points to those dimensions.
        if maybe_x.is_some() || maybe_y.is_some() || maybe_z.is_some() {
            let cuboid = tri.bounding_cuboid();
            let centroid = tri.centroid();
            let x_scale = maybe_x.map(|x| x / cuboid.w()).unwrap_or_else(S::one);
            let y_scale = maybe_y.map(|y| y / cuboid.h()).unwrap_or_else(S::one);
            let z_scale = maybe_z.map(|z| z / cuboid.d()).unwrap_or_else(S::one);
            let scale = Vector3 { x: x_scale, y: y_scale, z: z_scale };
            let (a, b, c) = tri.into();
            let translate = |v: Point3<S>| centroid + ((v - centroid).mul_element_wise(scale));
            let new_a = translate(a);
            let new_b = translate(b);
            let new_c = translate(c);
            tri = geom::Tri([new_a, new_b, new_c]);
        }

        // The color.
        let color = color
            .or_else(|| {
                draw.theme(|theme| {
                    theme
                        .color
                        .primitive
                        .get(&draw::theme::Primitive::Tri)
                        .map(|&c| c)
                })
            })
            .unwrap_or(draw.theme(|t| t.color.default));

        let points = tri.vertices();
        let vertices = draw::mesh::vertex::IterFromPoints::new(points, color);
        let indices = 0..geom::tri::NUM_VERTICES as usize;

        (spatial, vertices, indices)
    }
}

impl<S> From<geom::Tri<Point3<S>>> for Tri<S>
where
    S: BaseFloat,
{
    fn from(tri: geom::Tri<Point3<S>>) -> Self {
        let spatial = <_>::default();
        let color = <_>::default();
        Tri { tri, spatial, color }
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
        let a = Point3 { x: -fifty, y: thirty_three, z: zero };
        let b = Point3 { x: fifty, y: zero, z: zero };
        let c = Point3 { x: -fifty, y: -thirty_three, z: zero };
        Tri::from(geom::Tri([a, b, c]))
    }
}

impl<S> SetPosition<S> for Tri<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.spatial)
    }
}

impl<S> SetDimensions<S> for Tri<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.spatial)
    }
}

impl<S> SetColor<ColorScalar> for Tri<S> {
    fn rgba_mut(&mut self) -> &mut Option<Rgba> {
        SetColor::rgba_mut(&mut self.color)
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

// Drawing methods.

impl<'a, S> Drawing<'a, Tri<S>, S>
where
    S: BaseFloat,
{
    /// Use the given points as the vertices (corners) of the triangle.
    pub fn points<P>(self, a: P, b: P, c: P) -> Self
    where
        P: IntoPoint<S>,
    {
        self.map_ty(|ty| ty.points(a, b, c))
    }
}

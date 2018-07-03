use draw::mesh::vertex::IntoPoint;
use draw::properties::spatial::{dimension, orientation, position};
use draw::properties::{
    spatial, ColorScalar, Draw, Drawn, IntoDrawn, Primitive, Rgba, SetColor, SetDimensions,
    SetOrientation, SetPosition,
};
use draw::{self, Drawing};
use geom::{self, Point3, Vector3};
use math::{BaseFloat, ElementWise};
use std::{iter, slice};

/// Properties related to drawing a **Quad**.
#[derive(Clone, Debug)]
pub struct Quad<S = geom::scalar::Default> {
    quad: geom::Quad<Point3<S>>,
    spatial: spatial::Properties<S>,
    color: Option<Rgba>,
}
// Quad-specific methods.

impl<S> Quad<S> {
    /// Use the given four points as the vertices (corners) of the quad.
    pub fn points<P>(mut self, a: P, b: P, c: P, d: P) -> Self
    where
        P: IntoPoint<S>,
    {
        let a = a.into_point();
        let b = b.into_point();
        let c = c.into_point();
        let d = d.into_point();
        self.quad = geom::Quad([a, b, c, d]);
        self
    }
}

// Trait implementations.

impl<S> IntoDrawn<S> for Quad<S>
where
    S: BaseFloat,
{
    type Vertices = draw::mesh::vertex::IterFromPoints<geom::quad::Vertices<Point3<S>>, S>;
    type Indices = iter::Cloned<slice::Iter<'static, usize>>;
    fn into_drawn(self, draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Quad {
            mut quad,
            spatial,
            color,
        } = self;

        // If dimensions were specified, scale the points to those dimensions.
        let (maybe_x, maybe_y, maybe_z) = spatial.dimensions.to_scalars(&draw);
        if maybe_x.is_some() || maybe_y.is_some() || maybe_z.is_some() {
            let cuboid = quad.bounding_cuboid();
            let centroid = quad.centroid();
            let x_scale = maybe_x.map(|x| x / cuboid.w()).unwrap_or_else(S::one);
            let y_scale = maybe_y.map(|y| y / cuboid.h()).unwrap_or_else(S::one);
            let z_scale = maybe_z.map(|z| z / cuboid.d()).unwrap_or_else(S::one);
            let scale = Vector3 {
                x: x_scale,
                y: y_scale,
                z: z_scale,
            };
            let (a, b, c, d) = quad.into();
            let translate = |v: Point3<S>| centroid + ((v - centroid).mul_element_wise(scale));
            let new_a = translate(a);
            let new_b = translate(b);
            let new_c = translate(c);
            let new_d = translate(d);
            quad = geom::Quad([new_a, new_b, new_c, new_d]);
        }

        // The color.
        let color = color
            .or_else(|| {
                draw.theme(|theme| {
                    theme
                        .color
                        .primitive
                        .get(&draw::theme::Primitive::Quad)
                        .map(|&c| c)
                })
            })
            .unwrap_or(draw.theme(|t| t.color.default));

        let points = quad.vertices();
        let vertices = draw::mesh::vertex::IterFromPoints::new(points, color);
        let indices = geom::quad::TRIANGLE_INDICES.iter().cloned();

        (spatial, vertices, indices)
    }
}

impl<S> From<geom::Quad<Point3<S>>> for Quad<S>
where
    S: BaseFloat,
{
    fn from(quad: geom::Quad<Point3<S>>) -> Self {
        let spatial = <_>::default();
        let color = <_>::default();
        Quad {
            quad,
            spatial,
            color,
        }
    }
}

impl<S> Default for Quad<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        // Create a quad pointing towards 0.0 radians.
        let zero = S::zero();
        let fifty = S::from(50.0).unwrap();
        let left = -fifty;
        let bottom = -fifty;
        let right = fifty;
        let top = fifty;
        let a = Point3 {
            x: left,
            y: bottom,
            z: zero,
        };
        let b = Point3 {
            x: left,
            y: top,
            z: zero,
        };
        let c = Point3 {
            x: right,
            y: top,
            z: zero,
        };
        let d = Point3 {
            x: right,
            y: bottom,
            z: zero,
        };
        Quad::from(geom::Quad([a, b, c, d]))
    }
}

impl<S> SetOrientation<S> for Quad<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.spatial)
    }
}

impl<S> SetPosition<S> for Quad<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.spatial)
    }
}

impl<S> SetDimensions<S> for Quad<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.spatial)
    }
}

impl<S> SetColor<ColorScalar> for Quad<S> {
    fn rgba_mut(&mut self) -> &mut Option<Rgba> {
        SetColor::rgba_mut(&mut self.color)
    }
}

// Primitive conversions.

impl<S> From<Quad<S>> for Primitive<S> {
    fn from(prim: Quad<S>) -> Self {
        Primitive::Quad(prim)
    }
}

impl<S> Into<Option<Quad<S>>> for Primitive<S> {
    fn into(self) -> Option<Quad<S>> {
        match self {
            Primitive::Quad(prim) => Some(prim),
            _ => None,
        }
    }
}

// Drawing methods.

impl<'a, S> Drawing<'a, Quad<S>, S>
where
    S: BaseFloat,
{
    /// Use the given points as the vertices (corners) of the quad.
    pub fn points<P>(self, a: P, b: P, c: P, d: P) -> Self
    where
        P: IntoPoint<S>,
    {
        self.map_ty(|ty| ty.points(a, b, c, d))
    }
}

use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::{
    spatial, ColorScalar, Draw, Drawn, IntoDrawn, LinSrgba, SetColor, SetDimensions,
    SetOrientation, SetPosition,
};
use crate::draw::{self, theme};
use crate::geom::{self, Point2, Vector2};
use crate::math::BaseFloat;
use std::{iter, slice};

/// Properties related to drawing a **Rect**.
#[derive(Clone, Debug)]
pub struct Rect<S = geom::scalar::Default> {
    spatial: spatial::Properties<S>,
    color: Option<LinSrgba>,
}

// Trait implementations.

impl<S> IntoDrawn<S> for Rect<S>
where
    S: BaseFloat,
{
    type Vertices = draw::mesh::vertex::IterFromPoint2s<geom::quad::Vertices<Point2<S>>, S>;
    type Indices = iter::Cloned<slice::Iter<'static, usize>>;
    fn into_drawn(self, draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Rect { spatial, color } = self;

        // If dimensions were specified, scale the points to those dimensions.
        let (maybe_x, maybe_y, maybe_z) = spatial.dimensions.to_scalars(&draw);
        assert!(
            maybe_z.is_none(),
            "z dimension support for rect is unimplemented"
        );
        let default_w = || S::from(100.0).unwrap();
        let default_h = || S::from(100.0).unwrap();
        let w = maybe_x.unwrap_or_else(default_w);
        let h = maybe_y.unwrap_or_else(default_h);
        let rect = geom::Rect::from_wh(Vector2 { x: w, y: h });
        let color = color.unwrap_or_else(|| draw.theme().fill_lin_srgba(&theme::Primitive::Rect));
        let points = rect.corners().vertices();
        let vertices = draw::mesh::vertex::IterFromPoint2s::new(points, color);
        let indices = geom::quad::TRIANGLE_INDICES.iter().cloned();
        (spatial, vertices, indices)
    }
}

impl<S> From<geom::Rect<S>> for Rect<S>
where
    S: BaseFloat,
{
    fn from(r: geom::Rect<S>) -> Self {
        let spatial = <_>::default();
        let color = <_>::default();
        let (x, y, w, h) = r.x_y_w_h();
        Rect { spatial, color }.x_y(x, y).w_h(w, h)
    }
}

impl<S> Default for Rect<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        let spatial = <_>::default();
        let color = <_>::default();
        Rect { spatial, color }
    }
}

impl<S> SetOrientation<S> for Rect<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.spatial)
    }
}

impl<S> SetPosition<S> for Rect<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.spatial)
    }
}

impl<S> SetDimensions<S> for Rect<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.spatial)
    }
}

impl<S> SetColor<ColorScalar> for Rect<S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.color)
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

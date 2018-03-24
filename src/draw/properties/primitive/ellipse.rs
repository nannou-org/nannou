use draw::{self, Drawing};
use draw::properties::{spatial, ColorScalar, Draw, Drawn, IntoDrawn, Primitive, Rgba, SetColor, SetDimensions, SetPosition};
use draw::properties::spatial::{dimension, position};
use geom;
use math::{BaseFloat, Vector2};

/// Properties related to drawing an **Ellipse**.
#[derive(Clone, Debug)]
pub struct Ellipse<S = geom::DefaultScalar> {
    spatial: spatial::Properties<S>,
    color: Option<Rgba>,
    resolution: Option<usize>,
}

// Ellipse-specific methods.

impl<S> Ellipse<S>
where
    S: BaseFloat,
{
    /// Specify the width and height of the **Ellipse** via a given **radius**.
    pub fn radius(self, radius: S) -> Self {
        let side = radius * (S::one() + S::one());
        self.w_h(side, side)
    }

    /// The number of sides used to draw the ellipse.
    pub fn resolution(mut self, resolution: usize) -> Self {
        self.resolution = Some(resolution);
        self
    }
}

// Trait implementations.

impl<S> IntoDrawn<S> for Ellipse<S>
where
    S: BaseFloat,
{
    type Vertices = draw::mesh::vertex::IterFromPoint2s<geom::ellipse::TriangleVertices<S>, S>;
    type Indices = geom::ellipse::TriangleIndices;
    fn into_drawn(self, draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Ellipse {
            spatial,
            color,
            resolution,
        } = self;

        // First get the dimensions of the ellipse.
        let (maybe_x, maybe_y, maybe_z) = spatial.dimensions.to_scalars(&draw);
        assert!(
            maybe_z.is_none(),
            "z dimension support for ellipse is unimplemented"
        );

        // TODO: These should probably be adjustable via Theme.
        const DEFAULT_RESOLUTION: usize = 50;
        let default_w = || S::from(100.0).unwrap();
        let default_h = || S::from(100.0).unwrap();
        let w = maybe_x.unwrap_or_else(default_w);
        let h = maybe_y.unwrap_or_else(default_h);
        let rect = geom::Rect::from_wh(Vector2 { x: w, y: h });
        let resolution = resolution.unwrap_or(DEFAULT_RESOLUTION);
        let color = color
            .or_else(|| {
                draw.theme(|theme| {
                    theme
                        .color
                        .primitive
                        .get(&draw::theme::Primitive::Ellipse)
                        .map(|&c| c)
                })
            })
            .unwrap_or(draw.theme(|t| t.color.default));

        // TODO: Optimise this using the Circumference and ellipse indices iterators.
        let ellipse = geom::Ellipse::new(rect, resolution);
        let (points, indices) = ellipse.triangle_indices();
        let vertices = draw::mesh::vertex::IterFromPoint2s::new(points, color);
        (spatial, vertices, indices)
    }
}

impl<S> Default for Ellipse<S> {
    fn default() -> Self {
        let spatial = Default::default();
        let color = Default::default();
        let resolution = Default::default();
        Ellipse {
            spatial,
            color,
            resolution,
        }
    }
}

impl<S> SetPosition<S> for Ellipse<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.spatial)
    }
}

impl<S> SetDimensions<S> for Ellipse<S> {
    fn properties(&mut self) -> &mut dimension::Properties<S> {
        SetDimensions::properties(&mut self.spatial)
    }
}

impl<S> SetColor<ColorScalar> for Ellipse<S> {
    fn rgba_mut(&mut self) -> &mut Option<Rgba> {
        SetColor::rgba_mut(&mut self.color)
    }
}

// Primitive conversion.

impl<S> From<Ellipse<S>> for Primitive<S> {
    fn from(prim: Ellipse<S>) -> Self {
        Primitive::Ellipse(prim)
    }
}

impl<S> Into<Option<Ellipse<S>>> for Primitive<S> {
    fn into(self) -> Option<Ellipse<S>> {
        match self {
            Primitive::Ellipse(prim) => Some(prim),
            _ => None,
        }
    }
}

// Drawing methods.

impl<'a, S> Drawing<'a, Ellipse<S>, S>
where
    S: BaseFloat,
{
    /// Specify the width and height of the **Ellipse** via a given **radius**.
    pub fn radius(self, radius: S) -> Self {
        self.map_ty(|ty| ty.radius(radius))
    }

    /// The number of sides used to draw the ellipse.
    pub fn resolution(self, resolution: usize) -> Self {
        self.map_ty(|ty| ty.resolution(resolution))
    }
}

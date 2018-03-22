use draw;
use draw::properties::{spatial, ColorScalar, Draw, Drawn, IntoDrawn, Rgba, SetColor, SetDimensions, SetPosition};
use draw::properties::spatial::{dimension, position};
use geom;
use math::{BaseFloat, Point2, Vector2};
use std::ops;

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
    type Vertices = draw::mesh::vertex::IterFromPoint2s<
        geom::tri::VerticesFromIter<geom::ellipse::Triangles<S>, Point2<S>>,
        S,
    >;
    type Indices = ops::Range<usize>;
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
        let tris = geom::Ellipse::new(rect, resolution).triangles();
        let points = geom::tri::vertices_from_iter(tris);
        let num_points = points.len();
        let vertices = draw::mesh::vertex::IterFromPoint2s::new(points, color);
        let indices = 0..num_points;

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

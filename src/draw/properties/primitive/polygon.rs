use draw::{self, mesh, Drawing};
use draw::properties::{spatial, ColorScalar, Draw, Drawn, IntoDrawn, Primitive, Rgba, SetColor, SetDimensions, SetOrientation, SetPosition};
use draw::properties::spatial::{dimension, orientation, position};
use geom;
use math::{BaseFloat, Vector2};
use std::iter::Empty;

/// Properties related to drawing a **Polygon**.
#[derive(Clone, Debug)]
pub struct Polygon<S = geom::DefaultScalar> {
    spatial: spatial::Properties<S>,
    color: Option<Rgba>,
    point_range: ops::Range<usize>,
    points: Rc<RefCell<Vec<mesh::vertex::Point<S>>>>,
}

impl<I, S> Polygon<I, S>
where
    I: Iterator,
    S: BaseFloat,
{
    /// Create a new `Polygon` whose convex edges are described by the given points.
    pub fn new<P>(points: P) -> Self
    where
        P: IntoIterator<IntoIter = I, Item = I::Item>,
    {
        let spatial = Default::default();
        let color = Default::default();
        let polygon = geom::Polygon::new(points);
        Polygon {
            spatial,
            color,
            polygon,
        }
    }

    /// Use the given points to describe the outer edge of the polygon.
    pub fn points(self, points: P) -> Polygon<P::IntoIter, S>
    where
        P: IntoIterator<Item = I::Item>,
    {
        let Polygon { spatial, color, .. } = self;
        let polygon = geom::Polygon::new(points);
        Polygon {
            spatial,
            color,
            polygon,
        }
    }
}

impl<S> Default for Polygon<Empty<mesh::vertex::Point<S>>, S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        Polygon::new(Empty::default())
    }
}

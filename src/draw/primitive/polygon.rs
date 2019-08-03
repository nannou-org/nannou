use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{self, orientation, position};
use crate::draw::properties::{
    ColorScalar, Draw, Drawn, IntoDrawn, LinSrgba, SetColor, SetOrientation, SetPosition,
};
use crate::draw::{self, mesh, Drawing};
use crate::geom;
use crate::math::BaseFloat;
use std::iter;

/// A polygon prior to being initialised.
#[derive(Clone, Debug, Default)]
pub struct Pointless;

/// Properties related to drawing a **Polygon**.
#[derive(Clone, Debug)]
pub struct Polygon<C = Fill, S = geom::scalar::Default> {
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    color: C,
    ranges: draw::IntermediaryVertexDataRanges,
}

/// Color all vertices of the polygon with a single color.
#[derive(Clone, Debug)]
pub struct Fill(Option<mesh::vertex::Color>);

/// Color each vertex individually.
#[derive(Clone, Debug)]
pub struct PerVertex;

impl Pointless {
    /// Draw a filled, convex polygon whose edges are defined by the given list of vertices.
    pub(crate) fn points<P, S>(
        self,
        vertex_data: &mut draw::IntermediaryVertexData<S>,
        points: P,
    ) -> Polygon<Fill, S>
    where
        P: IntoIterator,
        P::Item: Into<mesh::vertex::Point<S>>,
        S: BaseFloat,
    {
        let mut ranges = draw::IntermediaryVertexDataRanges::default();
        ranges.points.start = vertex_data.points.len();
        vertex_data
            .points
            .extend(points.into_iter().map(Into::into));
        ranges.points.end = vertex_data.points.len();
        let color = Fill(None);
        Polygon::new(color, ranges)
    }

    /// Draw a convex polygon whose edges and vertex colours are described by the given sequence of
    /// vertices.
    pub(crate) fn colored_points<P, S>(
        self,
        vertex_data: &mut draw::IntermediaryVertexData<S>,
        points: P,
    ) -> Polygon<PerVertex, S>
    where
        P: IntoIterator,
        P::Item: Into<crate::mesh::vertex::WithColor<mesh::vertex::Point<S>, mesh::vertex::Color>>,
        S: BaseFloat,
    {
        let mut ranges = draw::IntermediaryVertexDataRanges::default();
        ranges.points.start = vertex_data.points.len();
        ranges.colors.start = vertex_data.colors.len();
        for v in points.into_iter().map(Into::into) {
            vertex_data.points.push(v.vertex);
            vertex_data.colors.push(v.color);
        }
        ranges.points.end = vertex_data.points.len();
        ranges.colors.end = vertex_data.colors.len();
        let color = PerVertex;
        Polygon::new(color, ranges)
    }
}

impl<C, S> Polygon<C, S>
where
    S: BaseFloat,
{
    // Initialise a new `Polygon` with no points, ready for drawing.
    fn new(color: C, ranges: draw::IntermediaryVertexDataRanges) -> Self {
        let orientation = Default::default();
        let position = Default::default();
        Polygon {
            orientation,
            position,
            color,
            ranges,
        }
    }
}

impl<'a, S> Drawing<'a, Pointless, S>
where
    S: BaseFloat,
{
    /// Describe the polygon's edges with the given list of consecutive vertices that join them.
    pub fn points<P>(self, points: P) -> Drawing<'a, Polygon<Fill, S>, S>
    where
        P: IntoIterator,
        P::Item: Into<mesh::vertex::Point<S>>,
        S: BaseFloat,
    {
        self.map_ty_with_vertices(|ty, mesh| ty.points(&mut mesh.vertex_data, points))
    }

    /// Describe the polygon's edges with the given list of consecutive vertices that join them.
    ///
    /// Each vertex may be colored uniquely.
    pub fn colored_points<P>(self, points: P) -> Drawing<'a, Polygon<PerVertex, S>, S>
    where
        P: IntoIterator,
        P::Item: Into<mesh::vertex::ColoredPoint<S>>,
        S: BaseFloat,
    {
        self.map_ty_with_vertices(|ty, mesh| ty.colored_points(&mut mesh.vertex_data, points))
    }
}

impl<S> IntoDrawn<S> for Pointless
where
    S: BaseFloat,
{
    type Vertices = iter::Empty<draw::mesh::Vertex<S>>;
    type Indices = iter::Empty<usize>;
    fn into_drawn(self, _draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let properties = Default::default();
        let vertices = iter::empty();
        let indices = iter::empty();
        (properties, vertices, indices)
    }
}

impl<S> IntoDrawn<S> for Polygon<Fill, S>
where
    S: BaseFloat,
{
    type Vertices = draw::properties::VerticesFromRanges;
    type Indices = geom::polygon::TriangleIndices;
    fn into_drawn(self, draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Polygon {
            orientation,
            position,
            color: Fill(color),
            ranges,
        } = self;

        // If color is not specified within the ranges, determine the fill colour to use.
        let fill_color = match ranges.colors.len() {
            0 => {
                let color = color
                    .or_else(|| {
                        draw.theme(|theme| {
                            theme
                                .color
                                .primitive
                                .get(&draw::theme::Primitive::Polygon)
                                .map(|&c| c.into_linear())
                        })
                    })
                    .unwrap_or(draw.theme(|t| t.color.default.into_linear()));
                Some(color)
            }
            _ => None,
        };

        let dimensions = spatial::dimension::Properties::default();
        let spatial = spatial::Properties {
            dimensions,
            orientation,
            position,
        };
        let indices = geom::polygon::triangle_indices(ranges.points.len());
        let vertices = draw::properties::VerticesFromRanges { ranges, fill_color };
        (spatial, vertices, indices)
    }
}

impl<S> IntoDrawn<S> for Polygon<PerVertex, S>
where
    S: BaseFloat,
{
    type Vertices = draw::properties::VerticesFromRanges;
    type Indices = geom::polygon::TriangleIndices;
    fn into_drawn(self, _draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Polygon {
            orientation,
            position,
            ranges,
            ..
        } = self;
        let fill_color = None;
        let dimensions = spatial::dimension::Properties::default();
        let spatial = spatial::Properties {
            dimensions,
            orientation,
            position,
        };
        let indices = geom::polygon::triangle_indices(ranges.points.len());
        let vertices = draw::properties::VerticesFromRanges { ranges, fill_color };
        (spatial, vertices, indices)
    }
}

impl<C, S> SetOrientation<S> for Polygon<C, S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl<C, S> SetPosition<S> for Polygon<C, S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.position)
    }
}

impl<S> SetColor<ColorScalar> for Polygon<Fill, S> {
    fn rgba_mut(&mut self) -> &mut Option<LinSrgba> {
        SetColor::rgba_mut(&mut self.color.0)
    }
}

impl<S> From<Pointless> for Primitive<S> {
    fn from(prim: Pointless) -> Self {
        Primitive::PolygonPointless(prim)
    }
}

impl<S> From<Polygon<Fill, S>> for Primitive<S> {
    fn from(prim: Polygon<Fill, S>) -> Self {
        Primitive::PolygonFill(prim)
    }
}

impl<S> From<Polygon<PerVertex, S>> for Primitive<S> {
    fn from(prim: Polygon<PerVertex, S>) -> Self {
        Primitive::PolygonColorPerVertex(prim)
    }
}

impl<S> Into<Option<Pointless>> for Primitive<S> {
    fn into(self) -> Option<Pointless> {
        match self {
            Primitive::PolygonPointless(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<Polygon<Fill, S>>> for Primitive<S> {
    fn into(self) -> Option<Polygon<Fill, S>> {
        match self {
            Primitive::PolygonFill(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<Polygon<PerVertex, S>>> for Primitive<S> {
    fn into(self) -> Option<Polygon<PerVertex, S>> {
        match self {
            Primitive::PolygonColorPerVertex(prim) => Some(prim),
            _ => None,
        }
    }
}

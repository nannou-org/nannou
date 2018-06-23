use draw::{self, Drawing};
use draw::mesh::vertex::IntoVertex;
use draw::properties::{Draw, Drawn, IntoDrawn, Primitive, SetOrientation, SetPosition};
use draw::properties::spatial::{self, orientation, position};
use geom;
use math::BaseFloat;
use mesh::vertex::{WithColor, WithTexCoords};
use std::{iter, ops};

/// A polyline prior to being initialised.
#[derive(Clone, Debug, Default)]
pub struct Vertexless;

/// Properties related to drawing a **Polyline**.
#[derive(Clone, Debug)]
pub struct Polyline<S = geom::DefaultScalar> {
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    vertex_data_ranges: draw::IntermediaryVertexDataRanges,
}

impl Vertexless {
    /// Draw a polyline whose path is defined by the given list of vertices.
    pub(crate) fn vertices<S, I>(
        self,
        mesh: &mut draw::IntermediaryMesh<S>,
        vertices: I,
    ) -> Polyline<S>
    where
        S: BaseFloat,
        I: IntoIterator,
        I::Item: IntoVertex<S>,
    {
        let mut vertex_data_ranges = draw::IntermediaryVertexDataRanges::default();
        vertex_data_ranges.points.start = mesh.vertex_data.points.len();
        vertex_data_ranges.colors.start = mesh.vertex_data.colors.len();
        vertex_data_ranges.tex_coords.start = mesh.vertex_data.tex_coords.len();

        for vertex in vertices {
            let WithTexCoords {
                tex_coords,
                vertex: WithColor {
                    color,
                    vertex: point,
                },
            } = vertex.into_vertex();
            mesh.vertex_data.points.push(point);
            mesh.vertex_data.colors.push(color);
            mesh.vertex_data.tex_coords.push(tex_coords);
        }

        vertex_data_ranges.points.end = mesh.vertex_data.points.len();
        vertex_data_ranges.colors.end = mesh.vertex_data.colors.len();
        vertex_data_ranges.tex_coords.end = mesh.vertex_data.tex_coords.len();
        Polyline::new(vertex_data_ranges)
    }
}

impl<S> Polyline<S>
where
    S: BaseFloat,
{
    // Initialise a new `Polyline` with its ranges into the intermediary mesh, ready for drawing.
    fn new(vertex_data_ranges: draw::IntermediaryVertexDataRanges) -> Self {
        let orientation = Default::default();
        let position = Default::default();
        Polyline {
            orientation,
            position,
            vertex_data_ranges,
        }
    }
}

impl<'a, S> Drawing<'a, Vertexless, S>
where
    S: BaseFloat,
{
    /// Describe the mesh with the given sequence of triangles.
    pub fn vertices<I>(self, vertices: I) -> Drawing<'a, Polyline<S>, S>
    where
        I: IntoIterator,
        I::Item: IntoVertex<S>,
    {
        self.map_ty_with_vertices(|ty, mesh| ty.vertices(mesh, vertices))
    }
}

impl<S> IntoDrawn<S> for Vertexless
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

impl<S> IntoDrawn<S> for Polyline<S>
where
    S: BaseFloat,
{
    // TODO: A `Vertices` type that converts the `VerticesFromRange`
    type Vertices = Vertices;
    type Indices = geom::polyline::Indices;
    fn into_drawn(self, _draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Polyline {
            orientation,
            position,
            vertex_data_ranges,
        } = self;
        let dimensions = spatial::dimension::Properties::default();
        let spatial = spatial::Properties { dimensions, orientation, position };
        let vertices_from_range = draw::properties::VerticesFromRanges {
            ranges: vertex_data_ranges,
            fill_color: None,
        };
        let vertices = unimplemented!();
        let indices = unimplemented!();
        (spatial, vertices, indices)
    }
}

impl<S> SetOrientation<S> for Polyline<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl<S> SetPosition<S> for Polyline<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.position)
    }
}

impl<S> From<Vertexless> for Primitive<S> {
    fn from(prim: Vertexless) -> Self {
        Primitive::PolylineVertexless(prim)
    }
}

impl<S> From<Polyline<S>> for Primitive<S> {
    fn from(prim: Polyline<S>) -> Self {
        Primitive::Polyline(prim)
    }
}

impl<S> Into<Option<Vertexless>> for Primitive<S> {
    fn into(self) -> Option<Vertexless> {
        match self {
            Primitive::PolylineVertexless(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<Polyline<S>>> for Primitive<S> {
    fn into(self) -> Option<Polyline<S>> {
        match self {
            Primitive::Polyline(prim) => Some(prim),
            _ => None,
        }
    }
}

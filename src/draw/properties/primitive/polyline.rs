use crate::draw::mesh::vertex::{IntoPoint, IntoVertex};
use crate::draw::properties::spatial::{self, orientation, position};
use crate::draw::properties::{Draw, Drawn, IntoDrawn, Primitive, SetOrientation, SetPosition};
use crate::draw::{self, Drawing};
use crate::geom::line::join::miter;
use crate::geom::{self, pt2, Point2};
use crate::math::BaseFloat;
use crate::mesh::vertex::{WithColor, WithTexCoords};
use std::iter;

/// A polyline prior to being initialised.
#[derive(Clone, Debug, Default)]
pub struct Vertexless;

/// Properties related to drawing a **Polyline**.
#[derive(Clone, Debug)]
pub struct Polyline<S = geom::scalar::Default> {
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    vertex_data_ranges: draw::IntermediaryVertexDataRanges,
}

impl Vertexless {
    /// Draw a polyline whose path is defined by the given list of vertices.
    pub(crate) fn vertices<S, I>(
        self,
        mesh: &mut draw::IntermediaryMesh<S>,
        half_thickness: S,
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

        fn v_to_pt2<S>(v: draw::mesh::Vertex<S>) -> Point2<S>
        where
            S: BaseFloat,
        {
            pt2(v.x, v.y)
        }

        // For each vertex in the given sequence, generate the miter point pairs. Colour and
        // texture each pair of points by using the colour and texture coords of the original
        // vertex.
        let mut v_iter = vertices.into_iter().map(IntoVertex::into_vertex);
        let mut a = None;
        let mut v = v_iter.next();
        let mut b = v.clone().map(v_to_pt2);
        loop {
            let next_v = v_iter.next();
            let next = next_v.clone().map(v_to_pt2);
            let [l, r] = match miter::next_pair(half_thickness, &mut a, &mut b, next) {
                None => break,
                Some(pair) => pair,
            };

            // `v` should always be `Some` if `Some` next miter pair was yielded.
            let WithTexCoords {
                tex_coords,
                vertex: WithColor { color, .. },
            } = v.clone().expect("no vertex for the next miter pair");

            // A function for pushing the left and right miter points.
            let mut push_point = |point| {
                mesh.vertex_data.points.push(point);
                mesh.vertex_data.colors.push(color);
                mesh.vertex_data.tex_coords.push(tex_coords);
            };
            push_point(l.into_point());
            push_point(r.into_point());

            // Update the vertex used for texturing and colouring the next miter pair.
            if next_v.is_some() {
                v = next_v;
            }
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
    /// Describe the polyline with some "half_thickness" of the line and the given sequence of
    /// vertices.
    pub fn vertices<I>(self, half_thickness: S, vertices: I) -> Drawing<'a, Polyline<S>, S>
    where
        I: IntoIterator,
        I::Item: IntoVertex<S>,
    {
        self.map_ty_with_vertices(|ty, mesh| ty.vertices(mesh, half_thickness, vertices))
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
    type Vertices = draw::properties::VerticesFromRanges;
    type Indices = geom::tri::FlattenIndices<miter::TriangleIndices>;
    fn into_drawn(self, _draw: Draw<S>) -> Drawn<S, Self::Vertices, Self::Indices> {
        let Polyline {
            orientation,
            position,
            vertex_data_ranges,
        } = self;
        let dimensions = spatial::dimension::Properties::default();
        let spatial = spatial::Properties {
            dimensions,
            orientation,
            position,
        };
        let n_points = (vertex_data_ranges.points.end - vertex_data_ranges.points.start) / 2;
        let vertices = draw::properties::VerticesFromRanges {
            ranges: vertex_data_ranges,
            fill_color: None,
        };
        let index_tris = miter::TriangleIndices::new(n_points);
        let indices = geom::tri::flatten_index_tris(index_tris);
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

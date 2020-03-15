use crate::draw::mesh::vertex::IntoVertex;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{SetOrientation, SetPosition};
use crate::draw::{self, Drawing};
use crate::geom;
use crate::math::BaseFloat;
use crate::mesh::vertex::{WithColor, WithTexCoords};
use std::ops;

/// The mesh type prior to being initialised with vertices or indices.
#[derive(Clone, Debug, Default)]
pub struct Vertexless;

/// Properties related to drawing an arbitrary mesh of colours, geometry and texture.
#[derive(Clone, Debug)]
pub struct Mesh<S = geom::scalar::Default> {
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    vertex_data_ranges: draw::IntermediaryVertexDataRanges,
    index_range: ops::Range<usize>,
    min_intermediary_index: usize,
}

// A simple iterator for flattening a fixed-size array of indices.
struct FlattenIndices<I> {
    iter: I,
    index: usize,
    min_intermediary_index: usize,
    current: [usize; 3],
}

impl Vertexless {
    /// Describe the mesh with a sequence of triangles.
    ///
    /// Each triangle may be composed of any vertex type that may be converted directly into the
    /// `draw::mesh::vertex` type.
    pub fn tris<S, I, V>(self, mesh: &mut draw::IntermediaryMesh<S>, tris: I) -> Mesh<S>
    where
        S: BaseFloat,
        I: IntoIterator<Item = geom::Tri<V>>,
        V: geom::Vertex + IntoVertex<S>,
    {
        let min_intermediary_index = mesh.vertex_data.points.len();
        let mut vertex_data_ranges = draw::IntermediaryVertexDataRanges::default();
        let mut index_range = 0..0;
        vertex_data_ranges.points.start = mesh.vertex_data.points.len();
        vertex_data_ranges.colors.start = mesh.vertex_data.colors.len();
        vertex_data_ranges.tex_coords.start = mesh.vertex_data.tex_coords.len();
        index_range.start = mesh.indices.len();

        let vertices = tris
            .into_iter()
            .flat_map(geom::Tri::vertices)
            .map(IntoVertex::into_vertex);
        for (i, vertex) in vertices.enumerate() {
            let WithTexCoords {
                tex_coords,
                vertex:
                    WithColor {
                        color,
                        vertex: point,
                    },
            } = vertex;
            mesh.vertex_data.points.push(point);
            mesh.vertex_data.colors.push(color);
            mesh.vertex_data.tex_coords.push(tex_coords);
            mesh.indices.push(min_intermediary_index + i);
        }

        vertex_data_ranges.points.end = mesh.vertex_data.points.len();
        vertex_data_ranges.colors.end = mesh.vertex_data.colors.len();
        vertex_data_ranges.tex_coords.end = mesh.vertex_data.tex_coords.len();
        index_range.end = mesh.indices.len();
        Mesh::new(vertex_data_ranges, index_range, min_intermediary_index)
    }

    /// Describe the mesh with the given indexed vertices.
    ///
    /// Each trio of `indices` describes a single triangle of `vertices`.
    ///
    /// Each vertex may be any type that may be converted directly into the `draw::mesh::vertex`
    /// type.
    pub fn indexed<S, V, I>(
        self,
        mesh: &mut draw::IntermediaryMesh<S>,
        vertices: V,
        indices: I,
    ) -> Mesh<S>
    where
        S: BaseFloat,
        V: IntoIterator,
        V::Item: IntoVertex<S>,
        I: IntoIterator<Item = [usize; 3]>,
    {
        let min_intermediary_index = mesh.vertex_data.points.len();
        let mut vertex_data_ranges = draw::IntermediaryVertexDataRanges::default();
        vertex_data_ranges.points.start = mesh.vertex_data.points.len();
        vertex_data_ranges.colors.start = mesh.vertex_data.colors.len();
        vertex_data_ranges.tex_coords.start = mesh.vertex_data.tex_coords.len();
        for vertex in vertices {
            let WithTexCoords {
                tex_coords,
                vertex:
                    WithColor {
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
        let mut index_range = mesh.indices.len()..mesh.indices.len();
        let iter = FlattenIndices {
            iter: indices.into_iter(),
            current: [0; 3],
            min_intermediary_index,
            index: 3,
        };
        mesh.indices.extend(iter);
        index_range.end = mesh.indices.len();
        Mesh::new(vertex_data_ranges, index_range, min_intermediary_index)
    }
}

impl<S> Mesh<S>
where
    S: BaseFloat,
{
    // Initialise a new `Mesh` with its ranges into the intermediary mesh, ready for drawing.
    fn new(
        vertex_data_ranges: draw::IntermediaryVertexDataRanges,
        index_range: ops::Range<usize>,
        min_intermediary_index: usize,
    ) -> Self {
        let orientation = Default::default();
        let position = Default::default();
        Mesh {
            orientation,
            position,
            vertex_data_ranges,
            index_range,
            min_intermediary_index,
        }
    }
}

impl<'a, S> Drawing<'a, Vertexless, S>
where
    S: BaseFloat,
{
    /// Describe the mesh with the given sequence of triangles.
    pub fn tris<I, V>(self, tris: I) -> Drawing<'a, Mesh<S>, S>
    where
        I: IntoIterator<Item = geom::Tri<V>>,
        V: geom::Vertex + IntoVertex<S>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.tris(ctxt.mesh, tris))
    }

    /// Describe the mesh with the given sequence of indexed vertices.
    pub fn indexed<V, I>(self, vertices: V, indices: I) -> Drawing<'a, Mesh<S>, S>
    where
        V: IntoIterator,
        V::Item: IntoVertex<S>,
        I: IntoIterator<Item = [usize; 3]>,
    {
        self.map_ty_with_context(|ty, ctxt| ty.indexed(ctxt.mesh, vertices, indices))
    }
}

impl draw::renderer::RenderPrimitive for Mesh<f32> {
    fn render_primitive(
        self,
        ctxt: draw::renderer::RenderContext,
        mesh: &mut draw::Mesh,
    ) -> draw::renderer::VertexMode {
        let Mesh {
            orientation,
            position,
            vertex_data_ranges,
            index_range,
            min_intermediary_index,
        } = self;

        // Determine the transform to apply to vertices.
        let global_transform = ctxt.transform;
        let local_transform = position.transform() * orientation.transform();
        let transform = global_transform * local_transform;

        // TODO: Could probably do this without `*FromRange`?
        let vertices_start_index = mesh.raw_vertex_count();
        let mut vertices = draw::properties::VerticesFromRanges::new(vertex_data_ranges, None);
        let mut indices =
            draw::properties::IndicesFromRange::new(index_range, min_intermediary_index);
        let vertices = std::iter::from_fn(|| {
            vertices.next(ctxt.intermediary_mesh).map(|mut v| {
                let p = *v.point();
                let p = cgmath::Point3::new(p.x, p.y, p.z);
                let p = cgmath::Transform::transform_point(&transform, p);
                v.vertex.vertex = geom::vec3(p.x, p.y, p.z);
                v
            })
        });
        let indices = std::iter::from_fn(|| {
            indices
                .next(&ctxt.intermediary_mesh.indices)
                .map(|i| (vertices_start_index + i - min_intermediary_index) as u32)
        });
        mesh.extend(vertices, indices);

        // TODO: Allow more options here.
        draw::renderer::VertexMode::Color
    }
}

impl<I> Iterator for FlattenIndices<I>
where
    I: Iterator<Item = [usize; 3]>,
{
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index < self.current.len() {
                let ix = self.current[self.index];
                self.index += 1;
                return Some(self.min_intermediary_index + ix);
            }
            match self.iter.next() {
                None => return None,
                Some(trio) => {
                    self.current = trio;
                    self.index = 0;
                }
            }
        }
    }
}

impl<S> SetOrientation<S> for Mesh<S> {
    fn properties(&mut self) -> &mut orientation::Properties<S> {
        SetOrientation::properties(&mut self.orientation)
    }
}

impl<S> SetPosition<S> for Mesh<S> {
    fn properties(&mut self) -> &mut position::Properties<S> {
        SetPosition::properties(&mut self.position)
    }
}

impl<S> From<Vertexless> for Primitive<S> {
    fn from(prim: Vertexless) -> Self {
        Primitive::MeshVertexless(prim)
    }
}

impl<S> From<Mesh<S>> for Primitive<S> {
    fn from(prim: Mesh<S>) -> Self {
        Primitive::Mesh(prim)
    }
}

impl<S> Into<Option<Vertexless>> for Primitive<S> {
    fn into(self) -> Option<Vertexless> {
        match self {
            Primitive::MeshVertexless(prim) => Some(prim),
            _ => None,
        }
    }
}

impl<S> Into<Option<Mesh<S>>> for Primitive<S> {
    fn into(self) -> Option<Mesh<S>> {
        match self {
            Primitive::Mesh(prim) => Some(prim),
            _ => None,
        }
    }
}

use crate::draw::mesh::vertex::IntoVertex;
use crate::draw::primitive::Primitive;
use crate::draw::properties::spatial::{orientation, position};
use crate::draw::properties::{SetOrientation, SetPosition};
use crate::draw::{self, Drawing};
use crate::geom;
use crate::math::BaseFloat;
use std::ops;

/// The mesh type prior to being initialised with vertices or indices.
#[derive(Clone, Debug, Default)]
pub struct Vertexless;

/// Properties related to drawing an arbitrary mesh of colours, geometry and texture.
#[derive(Clone, Debug)]
pub struct Mesh<S = geom::scalar::Default> {
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    vertex_range: ops::Range<usize>,
    index_range: ops::Range<usize>,
}

// A simple iterator for flattening a fixed-size array of indices.
struct FlattenIndices<I> {
    iter: I,
    index: usize,
    vertex_start_index: usize,
    current: [usize; 3],
}

impl Vertexless {
    // /// Describe the mesh with a sequence of colored triangles.
    // pub fn tris_colored<S, I, V>(self, mesh: &mut draw::IntermediaryMesh<S>, tris: I) -> Mesh<S>
    // where
    //     S: BaseFloat,
    //     I: IntoIterator<Item = geom::Tri<V>>,
    //     V: Into<draw::mesh::vertex::ColoredPoint<S>>,
    // {
    //     unimplemented!()
    // }

    // ///
    // pub fn tris_textured<S, I, V>(
    //     self,
    //     mesh: &mut draw::IntermediaryMesh<S>,

    /// Describe the mesh with a sequence of triangles.
    ///
    /// Each triangle may be composed of any vertex type that may be converted directly into the
    /// `draw::mesh::vertex` type.
    pub fn tris<S, I, V>(self, inner_mesh: &mut draw::Mesh<S>, tris: I) -> Mesh<S>
    where
        S: BaseFloat,
        I: IntoIterator<Item = geom::Tri<V>>,
        V: Clone + IntoVertex<S>,
    {
        let v_start = inner_mesh.points().len();
        let i_start = inner_mesh.indices().len();
        let vertices = tris
            .into_iter()
            .flat_map(geom::Tri::vertices)
            .map(IntoVertex::into_vertex);
        for (i, vertex) in vertices.enumerate() {
            inner_mesh.push_vertex(vertex);
            inner_mesh.push_index((v_start + i) as u32);
        }
        let v_end = inner_mesh.points().len();
        let i_end = inner_mesh.indices().len();
        Mesh::new(v_start..v_end, i_start..i_end)
    }

    /// Describe the mesh with the given indexed vertices.
    ///
    /// Each trio of `indices` describes a single triangle of `vertices`.
    ///
    /// Each vertex may be any type that may be converted directly into the `draw::mesh::vertex`
    /// type.
    pub fn indexed<S, V, I>(
        self,
        inner_mesh: &mut draw::Mesh<S>,
        vertices: V,
        indices: I,
    ) -> Mesh<S>
    where
        S: BaseFloat,
        V: IntoIterator,
        V::Item: IntoVertex<S>,
        I: IntoIterator<Item = [usize; 3]>,
    {
        let v_start = inner_mesh.points().len();
        let i_start = inner_mesh.indices().len();

        // Insert the vertices.
        inner_mesh.extend_vertices(vertices.into_iter().map(IntoVertex::into_vertex));

        // Insert the indices.
        let iter = FlattenIndices {
            iter: indices.into_iter(),
            current: [0; 3],
            vertex_start_index: v_start,
            index: 3,
        };
        inner_mesh.extend_indices(iter.map(|ix| ix as u32));

        let v_end = inner_mesh.points().len();
        let i_end = inner_mesh.indices().len();
        Mesh::new(v_start..v_end, i_start..i_end)
    }
}

impl<S> Mesh<S>
where
    S: BaseFloat,
{
    // Initialise a new `Mesh` with its ranges into the intermediary mesh, ready for drawing.
    fn new(vertex_range: ops::Range<usize>, index_range: ops::Range<usize>) -> Self {
        let orientation = Default::default();
        let position = Default::default();
        Mesh {
            orientation,
            position,
            vertex_range,
            index_range,
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
            vertex_range,
            index_range,
        } = self;

        // Determine the transform to apply to vertices.
        let global_transform = ctxt.transform;
        let local_transform = position.transform() * orientation.transform();
        let transform = global_transform * local_transform;

        // We need to update the indices to point to where vertices will be in the new mesh.
        let old_mesh_vertex_start = vertex_range.start as u32;
        let new_mesh_vertex_start = mesh.raw_vertex_count() as u32;
        let indices = index_range
            .map(|i| ctxt.intermediary_mesh.indices()[i])
            .map(|i| new_mesh_vertex_start + i - old_mesh_vertex_start);

        // Retrieve the vertices and transform them.
        let vertices = vertex_range
            .map(|i| {
                let p = ctxt.intermediary_mesh.points()[i];
                let p = cgmath::Point3::new(p.x, p.y, p.z);
                let p = cgmath::Transform::transform_point(&transform, p);
                let point = p.into();
                let color = ctxt.intermediary_mesh.colors()[i];
                let tex_coords = ctxt.intermediary_mesh.tex_coords()[i];
                draw::mesh::vertex::new(point, color, tex_coords)
            });

        // Extend the mesh!
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
                return Some(self.vertex_start_index + ix);
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

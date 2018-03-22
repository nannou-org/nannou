//! Items related to the custom mesh type used by the `Draw` API.

use geom;
use math::{BaseFloat, BaseNum};
use mesh::{self, MeshPoints, WithColors, WithIndices, WithTexCoords};
use std::ops::{Deref, DerefMut};

pub mod vertex;

pub use self::vertex::Vertex;

pub type Points<S> = Vec<vertex::Point<S>>;
pub type Indices = Vec<usize>;
pub type Colors = Vec<vertex::Color>;
pub type TexCoords<S> = Vec<vertex::TexCoords<S>>;

/// The inner mesh type used by the **draw::Mesh**.
pub type MeshType<S> =
    WithTexCoords<WithColors<WithIndices<MeshPoints<Points<S>>, Indices>, Colors>, TexCoords<S>, S>;

/// The custom mesh type used internally by the **Draw** API.
#[derive(Clone, Debug)]
pub struct Mesh<S = geom::DefaultScalar> {
    mesh: MeshType<S>,
}

impl<S> Mesh<S>
where
    S: BaseNum,
{
    /// The number of raw vertices contained within the mesh.
    pub fn raw_vertex_count(&self) -> usize {
        mesh::raw_vertex_count(self)
    }

    /// The number of vertices that would be yielded by a **Vertices** iterator for the given mesh.
    pub fn vertex_count(&self) -> usize {
        mesh::vertex_count(self)
    }

    /// The number of triangles that would be yielded by a **Triangles** iterator for the given mesh.
    pub fn triangle_count(&self) -> usize {
        mesh::triangle_count(self)
    }

    /// The **Mesh**'s vertex position channel.
    pub fn points(&self) -> &[vertex::Point<S>] {
        mesh::Points::points(self)
    }

    /// The **Mesh**'s vertex indices channel.
    pub fn indices(&self) -> &[usize] {
        mesh::Indices::indices(self)
    }

    /// The **Mesh**'s vertex colors channel.
    pub fn colors(&self) -> &[vertex::Color] {
        mesh::Colors::colors(self)
    }

    /// The **Mesh**'s vertex texture coordinates channel.
    pub fn tex_coords(&self) -> &[vertex::TexCoords<S>]
    where
        S: BaseFloat,
    {
        mesh::TexCoords::tex_coords(self)
    }

    /// Push the given vertex onto the inner channels.
    pub fn push_vertex(&mut self, v: Vertex<S>) {
        mesh::push_vertex(self, v);
    }

    /// Push the given index onto the inner **Indices** channel.
    pub fn push_index(&mut self, i: usize) {
        mesh::push_index(self, i);
    }

    /// Extend the mesh channels with the given vertices.
    pub fn extend_vertices<I>(&mut self, vs: I)
    where
        I: IntoIterator<Item = Vertex<S>>,
    {
        mesh::extend_vertices(self, vs);
    }

    /// Extend the **Mesh** indices channel with the given indices.
    pub fn extend_indices<I>(&mut self, is: I)
    where
        I: IntoIterator<Item = usize>,
    {
        mesh::extend_indices(self, is);
    }

    /// Extend the **Mesh** with the given vertices and indices.
    pub fn extend<V, I>(&mut self, vs: V, is: I)
    where
        V: IntoIterator<Item = Vertex<S>>,
        I: IntoIterator<Item = usize>,
    {
        self.extend_vertices(vs);
        self.extend_indices(is);
    }

    /// Clear all vertices from the mesh.
    pub fn clear_vertices(&mut self) {
        mesh::clear_vertices(self);
    }

    /// Clear all indices from the mesh.
    pub fn clear_indices(&mut self) {
        mesh::clear_indices(self);
    }

    /// Clear all vertices and indices from the mesh.
    pub fn clear(&mut self) {
        mesh::clear(self);
    }

    /// Produce an iterator yielding all raw (non-index-order) vertices.
    pub fn raw_vertices(&self) -> mesh::RawVertices<&Self> {
        mesh::raw_vertices(self)
    }

    /// Produce an iterator yielding all vertices in the order specified via the vertex indices.
    pub fn vertices(&self) -> mesh::Vertices<&Self> {
        mesh::vertices(self)
    }

    /// Produce an iterator yielding all triangles.
    pub fn triangles(&self) -> mesh::Triangles<&Self> {
        mesh::triangles(self)
    }

    /// Consume self and produce an iterator yielding all raw (non-index_order) vertices.
    pub fn into_raw_vertices(self) -> mesh::RawVertices<Self> {
        mesh::raw_vertices(self)
    }

    /// Consume self and produce an iterator yielding all vertices in index-order.
    pub fn into_vertices(self) -> mesh::Vertices<Self> {
        mesh::vertices(self)
    }

    /// Consume self and produce an iterator yielding all triangles.
    pub fn into_triangles(self) -> mesh::Triangles<Self> {
        mesh::triangles(self)
    }
}

impl<S> Default for Mesh<S> {
    fn default() -> Self {
        let mesh = Default::default();
        Mesh { mesh }
    }
}

impl<S> Deref for Mesh<S> {
    type Target = MeshType<S>;
    fn deref(&self) -> &Self::Target {
        &self.mesh
    }
}

impl<S> DerefMut for Mesh<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mesh
    }
}

impl<S> mesh::GetVertex for Mesh<S>
where
    MeshType<S>: mesh::GetVertex<Vertex = Vertex<S>>,
{
    type Vertex = Vertex<S>;
    fn get_vertex(&self, index: usize) -> Option<Self::Vertex> {
        self.mesh.get_vertex(index)
    }
}

impl<S> mesh::Points for Mesh<S>
where
    S: BaseNum,
{
    type Scalar = S;
    type Point = vertex::Point<S>;
    type Points = Points<S>;
    fn points(&self) -> &Self::Points {
        self.mesh.points()
    }
}

impl<S> mesh::Indices for Mesh<S> {
    type Indices = Indices;
    fn indices(&self) -> &Self::Indices {
        self.mesh.indices()
    }
}

impl<S> mesh::Colors for Mesh<S> {
    type Color = vertex::Color;
    type Colors = Colors;
    fn colors(&self) -> &Self::Colors {
        self.mesh.colors()
    }
}

impl<S> mesh::TexCoords for Mesh<S>
where
    S: BaseFloat,
{
    type TexCoordScalar = S;
    type TexCoords = TexCoords<S>;
    fn tex_coords(&self) -> &Self::TexCoords {
        self.mesh.tex_coords()
    }
}

impl<S> mesh::PushVertex<Vertex<S>> for Mesh<S> {
    fn push_vertex(&mut self, v: Vertex<S>) {
        self.mesh.push_vertex(v);
    }
}

impl<S> mesh::PushIndex for Mesh<S> {
    fn push_index(&mut self, index: usize) {
        self.mesh.push_index(index);
    }

    fn extend_indices<I>(&mut self, indices: I)
    where
        I: IntoIterator<Item = usize>,
    {
        self.mesh.extend_indices(indices);
    }
}

impl<S> mesh::ClearIndices for Mesh<S> {
    fn clear_indices(&mut self) {
        self.mesh.clear_indices();
    }
}

impl<S> mesh::ClearVertices for Mesh<S> {
    fn clear_vertices(&mut self) {
        self.mesh.clear_vertices();
    }
}

#[test]
fn test_method_access() {
    let mesh: Mesh = Default::default();
    assert_eq!(None, mesh::GetVertex::get_vertex(&mesh, 0));
    mesh::Points::points(&mesh);
    mesh::Indices::indices(&mesh);
    mesh::Colors::colors(&mesh);
    mesh::TexCoords::tex_coords(&mesh);
}

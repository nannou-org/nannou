//! Items related to the custom mesh type used by the `Draw` API.

use bevy::prelude::*;
use nannou_mesh as mesh;
use nannou_mesh::{self, MeshPoints, WithColors, WithIndices, WithTexCoords};
use std::ops::{Deref, DerefMut};
use bevy::render::extract_component::ExtractComponent;
use crate::mesh::vertex::Point;
use crate::RenderCommand;

pub mod builder;
pub mod vertex;

pub use self::builder::MeshBuilder;
pub use self::vertex::Vertex;

pub type Points = Vec<vertex::Point>;
pub type Indices = Vec<u32>;
pub type Colors = Vec<vertex::Color>;
pub type TexCoords = Vec<vertex::TexCoords>;

/// The inner mesh type used by the **draw::Mesh**.
pub type MeshType =
    WithTexCoords<WithColors<WithIndices<MeshPoints<Points>, Indices>, Colors>, TexCoords>;

/// The custom mesh type used internally by the **Draw** API.

#[derive(Component, Debug, Clone, ExtractComponent)]
pub struct ViewMesh {
    mesh: MeshType,
}

impl ViewMesh {
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
    pub fn points(&self) -> &[vertex::Point] {
        mesh::Points::points(self)
    }

    /// The **Mesh**'s vertex indices channel.
    pub fn indices(&self) -> &[u32] {
        mesh::Indices::indices(self)
    }

    /// The **Mesh**'s vertex colors channel.
    pub fn colors(&self) -> &[vertex::Color] {
        mesh::Colors::colors(self)
    }

    /// The **Mesh**'s vertex texture coordinates channel.
    pub fn tex_coords(&self) -> &[vertex::TexCoords] {
        mesh::TexCoords::tex_coords(self)
    }

    /// Push the given vertex onto the inner channels.
    pub fn push_vertex(&mut self, v: Vertex) {
        mesh::push_vertex(self, v);
    }

    /// Push the given index onto the inner **Indices** channel.
    pub fn push_index(&mut self, i: u32) {
        mesh::push_index(self, i);
    }

    /// Extend the mesh channels with the given vertices.
    pub fn extend_vertices<I>(&mut self, vs: I)
    where
        I: IntoIterator<Item = Vertex>,
    {
        mesh::extend_vertices(self, vs);
    }

    /// Extend the **Mesh** indices channel with the given indices.
    pub fn extend_indices<I>(&mut self, is: I)
    where
        I: IntoIterator<Item = u32>,
    {
        mesh::extend_indices(self, is);
    }

    /// Extend the **Mesh** with the given vertices and indices.
    pub fn extend<V, I>(&mut self, vs: V, is: I)
    where
        V: IntoIterator<Item = Vertex>,
        I: IntoIterator<Item = u32>,
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

    /// Consume self and produce an iterator yielding all raw (non-index_order) vertices.
    pub fn into_raw_vertices(self) -> mesh::RawVertices<Self> {
        mesh::raw_vertices(self)
    }

    /// Extend the mesh from the given slices.
    ///
    /// This is faster than `extend` which uses iteration internally.
    ///
    /// **Panic!**s if the length of the given points, colors and tex_coords slices do not match.
    pub fn extend_from_slices(
        &mut self,
        points: &[vertex::Point],
        indices: &[u32],
        colors: &[vertex::Color],
        tex_coords: &[vertex::TexCoords],
    ) {
        assert_eq!(points.len(), colors.len());
        assert_eq!(points.len(), tex_coords.len());
        let slices = (tex_coords, (colors, (indices, points)));
        mesh::ExtendFromSlice::extend_from_slice(&mut self.mesh, slices);
    }

    /// Extend the mesh with the given slices of vertices.
    pub fn extend_vertices_from_slices(
        &mut self,
        points: &[vertex::Point],
        colors: &[vertex::Color],
        tex_coords: &[vertex::TexCoords],
    ) {
        self.extend_from_slices(points, &[], colors, tex_coords);
    }

    /// Extend the mesh with the given slices of vertices.
    pub fn extend_indices_from_slice(&mut self, indices: &[u32]) {
        self.extend_from_slices(&[], indices, &[], &[]);
    }

    /// Produce an iterator yielding all vertices in the order specified via the vertex indices.
    pub fn vertices(&self) -> mesh::Vertices<&Self> {
        mesh::vertices(self)
    }

    /// Produce an iterator yielding all triangles.
    pub fn triangles(&self) -> mesh::Triangles<&Self> {
        mesh::triangles(self)
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

impl Default for ViewMesh {
    fn default() -> Self {
        let mesh = Default::default();
        ViewMesh { mesh }
    }
}

impl Deref for ViewMesh {
    type Target = MeshType;
    fn deref(&self) -> &Self::Target {
        &self.mesh
    }
}

impl DerefMut for ViewMesh {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mesh
    }
}

impl mesh::GetVertex<u32> for ViewMesh {
    type Vertex = Vertex;
    fn get_vertex(&self, index: u32) -> Option<Self::Vertex> {
        mesh::WithTexCoords::get_vertex(&self.mesh, index)
    }
}

impl mesh::Points for ViewMesh {
    type Point = vertex::Point;
    type Points = Points;
    fn points(&self) -> &Self::Points {
        self.mesh.points()
    }
}

impl mesh::Indices for ViewMesh {
    type Index = u32;
    type Indices = Indices;
    fn indices(&self) -> &Self::Indices {
        self.mesh.indices()
    }
}

impl mesh::Colors for ViewMesh {
    type Color = vertex::Color;
    type Colors = Colors;
    fn colors(&self) -> &Self::Colors {
        self.mesh.colors()
    }
}

impl mesh::TexCoords for ViewMesh {
    type TexCoord = Vec2;
    type TexCoords = TexCoords;
    fn tex_coords(&self) -> &Self::TexCoords {
        self.mesh.tex_coords()
    }
}

impl mesh::PushVertex<Vertex> for ViewMesh {
    fn push_vertex(&mut self, v: Vertex) {
        self.mesh.push_vertex(v);
    }
}

impl mesh::PushIndex for ViewMesh {
    type Index = u32;

    fn push_index(&mut self, index: Self::Index) {
        self.mesh.push_index(index);
    }

    fn extend_indices<I>(&mut self, indices: I)
    where
        I: IntoIterator<Item = Self::Index>,
    {
        self.mesh.extend_indices(indices);
    }
}

impl mesh::ClearIndices for ViewMesh {
    fn clear_indices(&mut self) {
        self.mesh.clear_indices();
    }
}

impl mesh::ClearVertices for ViewMesh {
    fn clear_vertices(&mut self) {
        self.mesh.clear_vertices();
    }
}

#[test]
fn test_method_access() {
    let mesh: ViewMesh = Default::default();
    assert_eq!(None, mesh::GetVertex::get_vertex(&mesh, 0));
    mesh::Points::points(&mesh);
    mesh::Indices::indices(&mesh);
    mesh::Colors::colors(&mesh);
    mesh::TexCoords::tex_coords(&mesh);
}

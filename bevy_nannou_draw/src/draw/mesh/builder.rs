//! Implementations of the lyon geometry builder traits for the `Mesh`.
//!
//! The aim is to allow for a tessellator to efficiently extend a mesh without using an
//! intermediary buffer.
//!
//! Lyon tessellators assume `f32` data, so we do the same in the following implementations.

use bevy::prelude::*;
use lyon::tessellation::{FillVertex, GeometryBuilderError, StrokeVertex, VertexId};
// use lyon::tessellation::{FillVertex, GeometryBuilderError, StrokeVertex, VertexId};
use lyon::tessellation::geometry_builder::{
    self, FillGeometryBuilder, GeometryBuilder, StrokeGeometryBuilder,
};

use crate::draw::mesh::MeshExt;

pub struct MeshBuilder<'a, A> {
    /// The mesh that is to be extended.
    mesh: &'a mut Mesh,
    /// The number of vertices in the mesh when begin was called.
    begin_vertex_count: u32,
    /// The number of indices in the mesh when begin was called.
    begin_index_count: u32,
    /// Transform matrix that also integrates position and orientation here.
    transform: Mat4,
    /// The way in which vertex attributes should be sourced.
    attributes: A,
}

pub struct SingleColor(Color);
pub struct ColorPerPoint;
pub struct TexCoordsPerPoint;

impl<'a, A> MeshBuilder<'a, A> {
    /// Begin extending the mesh.
    fn new(mesh: &'a mut Mesh, transform: Mat4, attributes: A) -> Self {
        MeshBuilder {
            mesh,
            begin_vertex_count: 0,
            begin_index_count: 0,
            transform,
            attributes,
        }
    }
}

impl<'a> MeshBuilder<'a, SingleColor> {
    /// Begin extending a mesh rendered with a single colour.
    pub fn single_color(mesh: &'a mut Mesh, transform: Mat4, color: Color) -> Self {
        Self::new(mesh, transform, SingleColor(color))
    }
}

impl<'a> MeshBuilder<'a, ColorPerPoint> {
    /// Begin extending a mesh where the path interpolates a unique color per point.
    pub fn color_per_point(mesh: &'a mut Mesh, transform: Mat4) -> Self {
        Self::new(mesh, transform, ColorPerPoint)
    }
}

impl<'a> MeshBuilder<'a, TexCoordsPerPoint> {
    /// Begin extending a mesh where the path interpolates a unique texture coordinates per point.
    pub fn tex_coords_per_point(mesh: &'a mut Mesh, transform: Mat4) -> Self {
        Self::new(mesh, transform, TexCoordsPerPoint)
    }
}

impl<'a, A> GeometryBuilder for MeshBuilder<'a, A> {
    fn begin_geometry(&mut self) {
        self.begin_vertex_count = self.mesh.count_vertices() as u32;
        self.begin_index_count = self.mesh.count_indices() as u32;
    }

    fn end_geometry(&mut self) -> geometry_builder::Count {
        geometry_builder::Count {
            vertices: self.mesh.count_vertices() as u32 - self.begin_vertex_count,
            indices: self.mesh.count_indices() as u32 - self.begin_index_count,
        }
    }

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        // Wind the indices in the opposite order to ensure the normals are facing outwards.
        self.mesh.push_index(c.to_usize() as u32);
        self.mesh.push_index(b.to_usize() as u32);
        self.mesh.push_index(a.to_usize() as u32);
    }

    fn abort_geometry(&mut self) {
        unimplemented!();
    }
}

impl<'a> FillGeometryBuilder for MeshBuilder<'a, SingleColor> {
    fn add_fill_vertex(&mut self, vertex: FillVertex) -> Result<VertexId, GeometryBuilderError> {
        // Retrieve the index.
        let id = VertexId::from_usize(self.mesh.count_vertices());

        let position = vertex.position();

        // Construct and insert the point
        let p = Vec2::new(position.x, position.y).extend(0.0);
        let point = self.transform.transform_point3(p);
        let SingleColor(color) = self.attributes;
        let tex_coords = Vec2::ZERO;

        self.mesh.points_mut().push(point.to_array());
        self.mesh.colors_mut().push(color.linear().to_f32_array());
        self.mesh.tex_coords_mut().push(tex_coords.to_array());
        self.mesh.normals_mut().push([0.0, 0.0, 1.0]);

        // Return the index.
        Ok(id)
    }
}

impl<'a> StrokeGeometryBuilder for MeshBuilder<'a, SingleColor> {
    fn add_stroke_vertex(
        &mut self,
        vertex: StrokeVertex,
    ) -> Result<VertexId, GeometryBuilderError> {
        // Retrieve the index.
        let id = VertexId::from_usize(self.mesh.count_vertices());

        let position = vertex.position();

        // Construct and insert the point
        let p = Vec2::new(position.x, position.y).extend(0.0);
        let point = self.transform.transform_point3(p);
        let SingleColor(color) = self.attributes;
        let tex_coords = Vec2::ZERO;

        self.mesh.points_mut().push(point.to_array());
        self.mesh.colors_mut().push(color.linear().to_f32_array());
        self.mesh.tex_coords_mut().push(tex_coords.to_array());
        self.mesh.normals_mut().push([0.0, 0.0, 1.0]);

        // Return the index.
        Ok(id)
    }
}

impl<'a> FillGeometryBuilder for MeshBuilder<'a, ColorPerPoint> {
    fn add_fill_vertex(
        &mut self,
        mut vertex: FillVertex,
    ) -> Result<VertexId, GeometryBuilderError> {
        // Retrieve the index.
        let id = VertexId::from_usize(self.mesh.count_vertices());

        let position = vertex.position();

        // Construct and insert the point
        let p = Vec2::new(position.x, position.y).extend(0.0);
        let point = self.transform.transform_point3(p);
        let col = vertex.interpolated_attributes();
        let color = Vec4::new(col[0], col[1], col[2], col[3]);
        let tex_coords = Vec2::ZERO;

        self.mesh.points_mut().push(point.to_array());
        self.mesh.colors_mut().push(color.to_array());
        self.mesh.tex_coords_mut().push(tex_coords.to_array());
        self.mesh.normals_mut().push([0.0, 0.0, 1.0]);

        // Return the index.
        Ok(id)
    }
}

impl<'a> StrokeGeometryBuilder for MeshBuilder<'a, ColorPerPoint> {
    fn add_stroke_vertex(
        &mut self,
        mut vertex: StrokeVertex,
    ) -> Result<VertexId, GeometryBuilderError> {
        // Retrieve the index.
        let id = VertexId::from_usize(self.mesh.count_vertices());

        let position = vertex.position();

        // Construct and insert the point
        let p = Vec2::new(position.x, position.y).extend(0.0);
        let point = self.transform.transform_point3(p);
        let col = vertex.interpolated_attributes();
        let color = Vec4::new(col[0], col[1], col[2], col[3]);
        let tex_coords = Vec2::ZERO;

        self.mesh.points_mut().push(point.to_array());
        self.mesh.colors_mut().push(color.to_array());
        self.mesh.tex_coords_mut().push(tex_coords.to_array());
        self.mesh.normals_mut().push([0.0, 0.0, 1.0]);

        // Return the index.
        Ok(id)
    }
}

impl<'a> FillGeometryBuilder for MeshBuilder<'a, TexCoordsPerPoint> {
    fn add_fill_vertex(
        &mut self,
        mut vertex: FillVertex,
    ) -> Result<VertexId, GeometryBuilderError> {
        // Retrieve the index.
        let id = VertexId::from_usize(self.mesh.count_vertices());

        let position = vertex.position();

        // Construct and insert the point
        let p = Vec2::new(position.x, position.y).extend(0.0);
        let point = self.transform.transform_point3(p);
        let tc = vertex.interpolated_attributes();
        let tex_coords = Vec2::new(tc[0], tc[1]);
        let color = Color::default();

        self.mesh.points_mut().push(point.to_array());
        self.mesh.colors_mut().push(color.linear().to_f32_array());
        self.mesh.tex_coords_mut().push(tex_coords.to_array());
        self.mesh.normals_mut().push([0.0, 0.0, 1.0]);

        // Return the index.
        Ok(id)
    }
}

impl<'a> StrokeGeometryBuilder for MeshBuilder<'a, TexCoordsPerPoint> {
    fn add_stroke_vertex(
        &mut self,
        mut vertex: StrokeVertex,
    ) -> Result<VertexId, GeometryBuilderError> {
        // Retrieve the index.
        let id = VertexId::from_usize(self.mesh.count_vertices());

        let position = vertex.position();

        // Construct and insert the point
        let p = Vec2::new(position.x, position.y).extend(0.0);
        let point = self.transform.transform_point3(p);
        let tc = vertex.interpolated_attributes();
        let tex_coords = Vec2::new(tc[0], tc[1]);
        let color = Color::default();

        self.mesh.points_mut().push(point.to_array());
        self.mesh.colors_mut().push(color.linear().to_f32_array());
        self.mesh.tex_coords_mut().push(tex_coords.to_array());
        self.mesh.normals_mut().push([0.0, 0.0, 1.0]);

        // Return the index.
        Ok(id)
    }
}

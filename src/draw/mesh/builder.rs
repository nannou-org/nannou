//! Implementations of the lyon geometry builder traits for the `Mesh`.
//!
//! The aim is to allow for a tessellator to efficiently extend a mesh without using an
//! intermediary buffer.
//!
//! Lyon tessellators assume `f32` data, so we do the same in the following implementations.

use crate::draw;
use crate::geom;
use cgmath::Matrix4;
use lyon::tessellation::geometry_builder::{
    self, FillGeometryBuilder, GeometryBuilder, StrokeGeometryBuilder,
};
use lyon::tessellation::{FillAttributes, GeometryBuilderError, StrokeAttributes, VertexId};

pub struct MeshBuilder<'a, A> {
    /// The mesh that is to be extended.
    mesh: &'a mut draw::Mesh,
    /// The number of vertices in the mesh when begin was called.
    begin_vertex_count: u32,
    /// The number of indices in the mesh when begin was called.
    begin_index_count: u32,
    /// Transform matrix that also integrates position and orientation here.
    transform: Matrix4<f32>,
    /// The way in which vertex attributes should be sourced.
    attributes: A,
}

pub struct SingleColor(draw::mesh::vertex::Color);
pub struct ColorPerPoint;

impl<'a, A> MeshBuilder<'a, A> {
    /// Begin extending the mesh.
    fn new(mesh: &'a mut draw::Mesh, transform: Matrix4<f32>, attributes: A) -> Self {
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
    pub fn single_color(
        mesh: &'a mut draw::Mesh,
        transform: Matrix4<f32>,
        color: draw::mesh::vertex::Color,
    ) -> Self {
        Self::new(mesh, transform, SingleColor(color))
    }
}

impl<'a> MeshBuilder<'a, ColorPerPoint> {
    /// Begin extending a mesh where the path interpolates a unique color per point.
    pub fn color_per_point(mesh: &'a mut draw::Mesh, transform: Matrix4<f32>) -> Self {
        Self::new(mesh, transform, ColorPerPoint)
    }
}

impl<'a, A> GeometryBuilder for MeshBuilder<'a, A> {
    fn begin_geometry(&mut self) {
        self.begin_vertex_count = self.mesh.points().len() as u32;
        self.begin_index_count = self.mesh.indices().len() as u32;
    }

    fn end_geometry(&mut self) -> geometry_builder::Count {
        geometry_builder::Count {
            vertices: self.mesh.points().len() as u32 - self.begin_vertex_count,
            indices: self.mesh.indices().len() as u32 - self.begin_index_count,
        }
    }

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        self.mesh.push_index(a.to_usize() as u32);
        self.mesh.push_index(b.to_usize() as u32);
        self.mesh.push_index(c.to_usize() as u32);
    }

    fn abort_geometry(&mut self) {
        unimplemented!();
    }
}

impl<'a> FillGeometryBuilder for MeshBuilder<'a, SingleColor> {
    fn add_fill_vertex(
        &mut self,
        position: lyon::math::Point,
        _attrs: FillAttributes,
    ) -> Result<VertexId, GeometryBuilderError> {
        // Retrieve the index.
        let id = VertexId::from_usize(self.mesh.points().len());

        // Construct and insert the point
        let p = geom::Point3::from(geom::Point2::from(position));
        let p = cgmath::Transform::transform_point(&self.transform, p.into());
        let point = geom::vec3(p.x, p.y, p.z);
        let SingleColor(color) = self.attributes;
        self.mesh
            .push_vertex(draw::mesh::vertex::IntoVertex::into_vertex((point, color)));

        // Return the index.
        Ok(id)
    }
}

impl<'a> StrokeGeometryBuilder for MeshBuilder<'a, SingleColor> {
    fn add_stroke_vertex(
        &mut self,
        position: lyon::math::Point,
        _attrs: StrokeAttributes,
    ) -> Result<VertexId, GeometryBuilderError> {
        // Retrieve the index.
        let id = VertexId::from_usize(self.mesh.points().len());

        // Construct and insert the point
        let p = geom::Point3::from(geom::Point2::from(position));
        let p = cgmath::Transform::transform_point(&self.transform, p.into());
        let point = geom::vec3(p.x, p.y, p.z);
        let SingleColor(color) = self.attributes;
        self.mesh
            .push_vertex(draw::mesh::vertex::IntoVertex::into_vertex((point, color)));

        // Return the index.
        Ok(id)
    }
}

impl<'a> FillGeometryBuilder for MeshBuilder<'a, ColorPerPoint> {
    fn add_fill_vertex(
        &mut self,
        position: lyon::math::Point,
        mut attrs: FillAttributes,
    ) -> Result<VertexId, GeometryBuilderError> {
        // Retrieve the index.
        let id = VertexId::from_usize(self.mesh.points().len());

        // Construct and insert the point
        let p = geom::Point3::from(geom::Point2::from(position));
        let p = cgmath::Transform::transform_point(&self.transform, p.into());
        let point = geom::vec3(p.x, p.y, p.z);
        let col = &attrs.interpolated_attributes();
        let color: draw::mesh::vertex::Color = (col[0], col[1], col[2], col[3]).into();
        self.mesh
            .push_vertex(draw::mesh::vertex::IntoVertex::into_vertex((point, color)));

        // Return the index.
        Ok(id)
    }
}

impl<'a> StrokeGeometryBuilder for MeshBuilder<'a, ColorPerPoint> {
    fn add_stroke_vertex(
        &mut self,
        position: lyon::math::Point,
        mut attrs: StrokeAttributes,
    ) -> Result<VertexId, GeometryBuilderError> {
        // Retrieve the index.
        let id = VertexId::from_usize(self.mesh.points().len());

        // Construct and insert the point
        let p = geom::Point3::from(geom::Point2::from(position));
        let p = cgmath::Transform::transform_point(&self.transform, p.into());
        let point = geom::vec3(p.x, p.y, p.z);
        let col = &attrs.interpolated_attributes();
        let color: draw::mesh::vertex::Color = (col[0], col[1], col[2], col[3]).into();
        self.mesh
            .push_vertex(draw::mesh::vertex::IntoVertex::into_vertex((point, color)));

        // Return the index.
        Ok(id)
    }
}

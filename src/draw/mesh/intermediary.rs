use crate::draw::mesh;
use crate::geom::{self, Point2, Point3};
use crate::mesh::vertex::{WithColor, WithTexCoords};
use lyon::tessellation::geometry_builder::{self, GeometryBuilder, GeometryBuilderError, VertexId};
use std::ops;

/// Types supported by the **IntermediaryMesh** **GeometryBuilder** implementation.
pub trait IntermediaryVertex<S> {
    fn add_to_data(self, mesh: &mut IntermediaryVertexData<S>);
}

/// A set of intermediary buffers for collecting geometry point data for geometry types that may
/// produce a dynamic number of vertices that may or not also contain colour or texture data.
#[derive(Clone, Debug)]
pub struct IntermediaryVertexData<S = geom::scalar::Default> {
    pub(crate) points: Vec<mesh::vertex::Point<S>>,
    pub(crate) colors: Vec<mesh::vertex::Color>,
    pub(crate) tex_coords: Vec<mesh::vertex::TexCoords<S>>,
}

/// An intermediary mesh to which drawings-in-progress may store vertex data and indices until they
/// are submitted to the **Draw**'s inner mesh.
#[derive(Clone, Debug)]
pub struct IntermediaryMesh<S = geom::scalar::Default> {
    pub(crate) vertex_data: IntermediaryVertexData<S>,
    pub(crate) indices: Vec<usize>,
}

/// A set of ranges into the **IntermediaryVertexData**.
///
/// This allows polygons, polylines, etc to track which slices of data are associated with their
/// own instance.
#[derive(Clone, Debug)]
pub struct IntermediaryVertexDataRanges {
    pub points: ops::Range<usize>,
    pub colors: ops::Range<usize>,
    pub tex_coords: ops::Range<usize>,
}

/// A `lyon::GeometryBuilder` around the `IntermediaryMesh` type.
#[derive(Debug)]
pub struct IntermediaryMeshBuilder<'a, S = geom::scalar::Default> {
    pub(crate) mesh: &'a mut IntermediaryMesh<S>,
    pub(crate) vertex_data_ranges: IntermediaryVertexDataRanges,
    pub(crate) index_range: ops::Range<usize>,
}

impl<S> IntermediaryMesh<S> {
    /// Produce a lyon-compatible `GeometryBuilder` for extending the `IntermediaryMesh`.
    pub fn builder(&mut self) -> IntermediaryMeshBuilder<S> {
        let vertex_data_ranges = Default::default();
        let index_range = 0..0;
        let mut builder = IntermediaryMeshBuilder {
            mesh: self,
            vertex_data_ranges,
            index_range,
        };
        builder.update_ranges_start();
        builder
    }
}

impl<'a, S> IntermediaryMeshBuilder<'a, S> {
    fn update_ranges_start(&mut self) {
        self.vertex_data_ranges.points.start = self.mesh.vertex_data.points.len();
        self.vertex_data_ranges.colors.start = self.mesh.vertex_data.colors.len();
        self.vertex_data_ranges.tex_coords.start = self.mesh.vertex_data.tex_coords.len();
        self.index_range.start = self.mesh.indices.len();
    }

    fn update_ranges_end(&mut self) {
        self.vertex_data_ranges.points.end = self.mesh.vertex_data.points.len();
        self.vertex_data_ranges.colors.end = self.mesh.vertex_data.colors.len();
        self.vertex_data_ranges.tex_coords.end = self.mesh.vertex_data.tex_coords.len();
        self.index_range.end = self.mesh.indices.len();
    }

    pub(crate) fn begin_geom(&mut self) {
        self.update_ranges_start();
        self.vertex_data_ranges.points.end = self.vertex_data_ranges.points.start;
    }

    pub(crate) fn end_geom(&mut self) -> geometry_builder::Count {
        self.update_ranges_end();
        let vertices = self.vertex_data_ranges.points.len() as u32;
        let indices = self.index_range.len() as u32;
        geometry_builder::Count { vertices, indices }
    }

    pub(crate) fn add_tri(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        self.mesh.indices.push(a.to_usize());
        self.mesh.indices.push(b.to_usize());
        self.mesh.indices.push(c.to_usize());
    }

    pub(crate) fn abort_geom(&mut self) {
        self.update_ranges_end();
    }

    pub fn vertex_data_ranges(&self) -> IntermediaryVertexDataRanges {
        self.vertex_data_ranges.clone()
    }

    pub fn index_range(&self) -> ops::Range<usize> {
        self.index_range.clone()
    }
}

impl<'a, V, S> GeometryBuilder<V> for IntermediaryMeshBuilder<'a, S>
where
    V: IntermediaryVertex<S>,
{
    fn begin_geometry(&mut self) {
        self.begin_geom();
    }

    fn end_geometry(&mut self) -> geometry_builder::Count {
        self.end_geom()
    }

    fn add_vertex(&mut self, v: V) -> Result<VertexId, GeometryBuilderError> {
        let id = self.vertex_data_ranges.points.end as u32;
        if id >= std::u32::MAX {
            return Err(GeometryBuilderError::TooManyVertices);
        }
        v.add_to_data(&mut self.mesh.vertex_data);
        debug_assert!(
            self.mesh.vertex_data.points.len() > id as usize,
            "intermediary vertices should always add at least one position attribute",
        );
        self.vertex_data_ranges.points.end += 1;
        Ok(VertexId(id))
    }

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        self.add_tri(a, b, c);
    }

    fn abort_geometry(&mut self) {
        self.abort_geom();
    }
}

impl<S> IntermediaryVertex<S> for Point2<S>
where
    S: crate::math::Zero,
{
    fn add_to_data(self, data: &mut IntermediaryVertexData<S>) {
        data.points.push(self.into());
    }
}

impl<S> IntermediaryVertex<S> for Point3<S> {
    fn add_to_data(self, data: &mut IntermediaryVertexData<S>) {
        data.points.push(self);
    }
}

impl<V, S> IntermediaryVertex<S> for WithColor<V, mesh::vertex::Color>
where
    V: IntermediaryVertex<S>,
{
    fn add_to_data(self, data: &mut IntermediaryVertexData<S>) {
        data.colors.push(self.color);
        self.vertex.add_to_data(data);
    }
}

impl<V, S> IntermediaryVertex<S> for WithTexCoords<V, Point2<S>>
where
    V: IntermediaryVertex<S>,
{
    fn add_to_data(self, data: &mut IntermediaryVertexData<S>) {
        data.tex_coords.push(self.tex_coords);
        self.vertex.add_to_data(data);
    }
}

impl<S> Default for IntermediaryVertexData<S> {
    fn default() -> Self {
        IntermediaryVertexData {
            points: Default::default(),
            colors: Default::default(),
            tex_coords: Default::default(),
        }
    }
}

impl<S> Default for IntermediaryMesh<S> {
    fn default() -> Self {
        IntermediaryMesh {
            vertex_data: Default::default(),
            indices: Default::default(),
        }
    }
}

impl Default for IntermediaryVertexDataRanges {
    fn default() -> Self {
        IntermediaryVertexDataRanges {
            points: 0..0,
            colors: 0..0,
            tex_coords: 0..0,
        }
    }
}

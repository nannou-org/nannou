use crate::draw::mesh;
use crate::geom;
use std::ops;

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

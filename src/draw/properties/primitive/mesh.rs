use draw::{self, mesh, Drawing};
use draw::properties::{ColorScalar, Draw, Drawn, IntoDrawn, Primitive, Rgba, SetColor, SetOrientation, SetPosition};
use draw::properties::spatial::{self, orientation, position};
use geom;
use math::BaseFloat;
use std::iter;

/// The mesh type prior to being initialised with vertices or indices.
#[derive(Clone, Debug, Default)]
pub struct Vertexless;

/// Properties related to drawing an arbitrary mesh of colours, geometry and texture.
#[derive(Clone, Debug)]
pub struct Mesh<S = geom::DefaultScalar> {
    position: position::Properties<S>,
    orientation: orientation::Properties<S>,
    vertex_data_ranges: draw::GeomVertexDataRanges,
    index_ranges: ops::Range<usize>,
}

impl Vertexless {
    /// Describe the mesh with a sequence of triangles.
    ///
    /// Each triangle may be composed of any vertex type that may be converted directly into the
    /// `draw;;mesh::vertex` type.
}

// draw.mesh().tris(tris)
// draw.mesh().indexed(vertices, indices)

// draw.mesh().textured_tris(tris) // calls `tris` under the hood.
// draw.mesh().colored_tris(tris) // calls `tris` under the hood.
// draw.mesh().colored_textured_tris(tris)

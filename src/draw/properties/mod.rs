//! Parameters which a **Drawing** instance may use to describe certain properties of a drawing.
//!
//! Each time a new method is chained onto a **Drawing** instance, it uses the given values to set
//! one or more properties for the drawing.
//!
//! Each **Drawing** instance is associated with a specific **Node** in the geometry graph and has
//! a unique **node::Index** to simplify this.

pub mod color;
pub mod fill;
pub mod spatial;
pub mod stroke;

use crate::draw;
use crate::math::BaseFloat;
use std::ops;

pub use self::color::SetColor;
pub use self::fill::SetFill;
pub use self::spatial::dimension::SetDimensions;
pub use self::spatial::orientation::SetOrientation;
pub use self::spatial::position::SetPosition;
pub use self::stroke::SetStroke;

/// The scalar type used for the color channel values.
pub type ColorScalar = crate::color::DefaultScalar;

/// The RGBA type used by the `Common` params.
pub type Srgba = color::DefaultSrgba;

/// The RGBA type used by the `Common` params.
pub type LinSrgba = color::DefaultLinSrgba;

// Methods for updating **Draw**'s geometry graph and mesh upon completion of **Drawing**.

/// Uses a set of ranges to index into the intermediary mesh and produce vertices.
#[derive(Debug)]
pub struct VerticesFromRanges {
    pub ranges: draw::IntermediaryVertexDataRanges,
    pub fill_color: Option<draw::mesh::vertex::Color>,
}

/// Uses a range to index into the intermediary mesh indices.
#[derive(Debug)]
pub struct IndicesFromRange {
    pub range: ops::Range<usize>,
    pub min_index: usize,
}

impl VerticesFromRanges {
    pub fn new(ranges: draw::IntermediaryVertexDataRanges, fill_color: Option<LinSrgba>) -> Self {
        VerticesFromRanges { ranges, fill_color }
    }
}

impl IndicesFromRange {
    pub fn new(range: ops::Range<usize>, min_index: usize) -> Self {
        IndicesFromRange { range, min_index }
    }
}

impl VerticesFromRanges {
    pub fn next<S>(&mut self, mesh: &draw::IntermediaryMesh<S>) -> Option<draw::mesh::Vertex<S>>
    where
        S: BaseFloat,
    {
        let VerticesFromRanges {
            ref mut ranges,
            fill_color,
        } = *self;

        let point = Iterator::next(&mut ranges.points);
        let color = Iterator::next(&mut ranges.colors);
        let tex_coords = Iterator::next(&mut ranges.tex_coords);

        let point = match point {
            None => return None,
            Some(point_ix) => *mesh
                .vertex_data
                .points
                .get(point_ix)
                .expect("no point for point index in IntermediaryMesh"),
        };

        let color = color
            .map(|color_ix| {
                *mesh
                    .vertex_data
                    .colors
                    .get(color_ix)
                    .expect("no color for color index in IntermediaryMesh")
            })
            .or(fill_color)
            .expect("no color for vertex");

        let tex_coords = tex_coords
            .map(|tex_coords_ix| {
                *mesh
                    .vertex_data
                    .tex_coords
                    .get(tex_coords_ix)
                    .expect("no tex_coords for tex_coords index in IntermediaryMesh")
            })
            .unwrap_or_else(draw::mesh::vertex::default_tex_coords);

        Some(draw::mesh::vertex::new(point, color, tex_coords))
    }
}

impl IndicesFromRange {
    pub fn next(&mut self, intermediary_indices: &[usize]) -> Option<usize> {
        Iterator::next(&mut self.range).map(|ix| {
            *intermediary_indices
                .get(ix)
                .expect("index into `intermediary_indices` is out of range")
        })
    }

    pub fn min_index(&self) -> usize {
        self.min_index
    }
}

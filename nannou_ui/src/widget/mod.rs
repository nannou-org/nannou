use crate::graph::{self, Graph};
use nannou::geom::Vector2;
use std::any::Any;

pub use container::Container;
pub use mesh::Mesh;
pub use root::Root;

pub mod container;
pub mod mesh;
pub mod root;

pub trait Widget: 'static + Any {
    /// Compute the layout of this widget and its children.
    ///
    /// Returns the size
    fn layout(&mut self, ctx: LayoutCtx) -> Vector2;
}

struct RectConstraints;

pub struct LayoutCtx<'a> {
    graph: &'a mut Graph,
    constraints: RectConstraints,
}

pub type Id = graph::NodeIndex;

// TODO:
//
// Layout
//
// - Pad
// - Flex (Container)
// - Flow (Container)
// - Split (Container of two)
// - Either (Container conditionally showing child A or B).
// - Zoom
//
// Primitive
//
// - Crop (crops children to its bounds)
// - Blend (uses pipeline with given blend mode to render children)
// - Mesh
// - Path
// - Text

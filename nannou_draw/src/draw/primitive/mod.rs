use bevy::prelude::{Color, Component};
use lyon::tessellation::{FillOptions, StrokeOptions};

use nannou_core::geom::{Vec2, Vec3};

use crate::draw::primitive::polygon::{PolygonOptions, SetPolygon};
use crate::draw::properties::spatial::{dimension, orientation, position};
use crate::draw::properties::tex_coords::SetTexCoords;
use crate::draw::properties::{
    SetColor, SetDimensions, SetFill, SetOrientation, SetPosition, SetStroke,
};

pub use self::arrow::Arrow;
pub use self::ellipse::Ellipse;
pub use self::line::Line;
pub use self::mesh::PrimitiveMesh;
pub use self::path::{Path, PathFill, PathInit, PathStroke};
pub use self::polygon::{Polygon, PolygonInit};
pub use self::quad::Quad;
pub use self::rect::Rect;
pub use self::text::Text;
pub use self::tri::Tri;

pub mod arrow;
pub mod ellipse;
pub mod line;
pub mod mesh;
pub mod path;
pub mod polygon;
pub mod quad;
pub mod rect;
pub mod text;
pub mod tri;

type Vertex = (Vec3, Color, Vec2);

/// A wrapper around all primitive sets of properties so that they may be stored within the
/// **Draw**'s `drawing` field while they are being drawn.
///
/// This also allows us to flush all pending drawings to the mesh if `Draw::to_frame` is called
/// before their respective **Drawing** types are dropped.
#[derive(Component, Clone, Debug)]
pub enum Primitive {
    Arrow(Arrow),
    Ellipse(Ellipse),
    Line(Line),
    MeshVertexless(mesh::Vertexless),
    Mesh(PrimitiveMesh),
    PathInit(PathInit),
    PathFill(PathFill),
    PathStroke(PathStroke),
    Path(Path),
    PolygonInit(PolygonInit),
    Polygon(Polygon),
    Quad(Quad),
    Rect(Rect),
    Text(Text),
    Tri(Tri),
}

/// The cheapest possible variant, used as a placeholder when temporarily taking ownership of a
/// stored primitive (e.g. for type-state transitions).
impl Default for Primitive {
    fn default() -> Self {
        Primitive::PathInit(PathInit)
    }
}

// Total accessors over all variants for each property channel.
//
// Each returns `Some` exactly for the variants whose inner type implements the corresponding
// `Set*` trait, `None` otherwise. The marker-gated `Drawing` API only exposes a property method
// when the marker type implements the trait, so `None` is unreachable via the public builders -
// it exists to keep these functions total.
impl Primitive {
    pub(crate) fn color_mut(&mut self) -> Option<&mut Option<Color>> {
        match self {
            Primitive::Arrow(p) => Some(SetColor::color_mut(p)),
            Primitive::Ellipse(p) => Some(SetColor::color_mut(p)),
            Primitive::Line(p) => Some(SetColor::color_mut(p)),
            Primitive::Mesh(p) => Some(SetColor::color_mut(p)),
            Primitive::PathFill(p) => Some(SetColor::color_mut(p)),
            Primitive::PathStroke(p) => Some(SetColor::color_mut(p)),
            Primitive::Path(p) => Some(SetColor::color_mut(p)),
            Primitive::PolygonInit(p) => Some(SetColor::color_mut(p)),
            Primitive::Polygon(p) => Some(SetColor::color_mut(p)),
            Primitive::Quad(p) => Some(SetColor::color_mut(p)),
            Primitive::Rect(p) => Some(SetColor::color_mut(p)),
            Primitive::Text(p) => Some(SetColor::color_mut(p)),
            Primitive::Tri(p) => Some(SetColor::color_mut(p)),
            Primitive::MeshVertexless(_) | Primitive::PathInit(_) => None,
        }
    }

    pub(crate) fn position_mut(&mut self) -> Option<&mut position::Properties> {
        match self {
            Primitive::Arrow(p) => Some(SetPosition::properties(p)),
            Primitive::Ellipse(p) => Some(SetPosition::properties(p)),
            Primitive::Line(p) => Some(SetPosition::properties(p)),
            Primitive::Mesh(p) => Some(SetPosition::properties(p)),
            Primitive::PathFill(p) => Some(SetPosition::properties(p)),
            Primitive::PathStroke(p) => Some(SetPosition::properties(p)),
            Primitive::Path(p) => Some(SetPosition::properties(p)),
            Primitive::PolygonInit(p) => Some(SetPosition::properties(p)),
            Primitive::Polygon(p) => Some(SetPosition::properties(p)),
            Primitive::Quad(p) => Some(SetPosition::properties(p)),
            Primitive::Rect(p) => Some(SetPosition::properties(p)),
            Primitive::Text(p) => Some(SetPosition::properties(p)),
            Primitive::Tri(p) => Some(SetPosition::properties(p)),
            Primitive::MeshVertexless(_) | Primitive::PathInit(_) => None,
        }
    }

    pub(crate) fn orientation_mut(&mut self) -> Option<&mut orientation::Properties> {
        match self {
            Primitive::Arrow(p) => Some(SetOrientation::properties(p)),
            Primitive::Ellipse(p) => Some(SetOrientation::properties(p)),
            Primitive::Line(p) => Some(SetOrientation::properties(p)),
            Primitive::Mesh(p) => Some(SetOrientation::properties(p)),
            Primitive::PathFill(p) => Some(SetOrientation::properties(p)),
            Primitive::PathStroke(p) => Some(SetOrientation::properties(p)),
            Primitive::Path(p) => Some(SetOrientation::properties(p)),
            Primitive::PolygonInit(p) => Some(SetOrientation::properties(p)),
            Primitive::Polygon(p) => Some(SetOrientation::properties(p)),
            Primitive::Quad(p) => Some(SetOrientation::properties(p)),
            Primitive::Rect(p) => Some(SetOrientation::properties(p)),
            Primitive::Text(p) => Some(SetOrientation::properties(p)),
            Primitive::Tri(p) => Some(SetOrientation::properties(p)),
            Primitive::MeshVertexless(_) | Primitive::PathInit(_) => None,
        }
    }

    pub(crate) fn dimensions_mut(&mut self) -> Option<&mut dimension::Properties> {
        match self {
            Primitive::Ellipse(p) => Some(SetDimensions::properties(p)),
            Primitive::Quad(p) => Some(SetDimensions::properties(p)),
            Primitive::Rect(p) => Some(SetDimensions::properties(p)),
            Primitive::Text(p) => Some(SetDimensions::properties(p)),
            Primitive::Tri(p) => Some(SetDimensions::properties(p)),
            Primitive::Arrow(_)
            | Primitive::Line(_)
            | Primitive::Mesh(_)
            | Primitive::MeshVertexless(_)
            | Primitive::PathInit(_)
            | Primitive::PathFill(_)
            | Primitive::PathStroke(_)
            | Primitive::Path(_)
            | Primitive::PolygonInit(_)
            | Primitive::Polygon(_) => None,
        }
    }

    pub(crate) fn stroke_options_mut(&mut self) -> Option<&mut StrokeOptions> {
        match self {
            Primitive::Arrow(p) => Some(SetStroke::stroke_options_mut(p)),
            Primitive::Ellipse(p) => Some(SetStroke::stroke_options_mut(p)),
            Primitive::Line(p) => Some(SetStroke::stroke_options_mut(p)),
            Primitive::PathStroke(p) => Some(SetStroke::stroke_options_mut(p)),
            Primitive::PolygonInit(p) => Some(SetStroke::stroke_options_mut(p)),
            Primitive::Quad(p) => Some(SetStroke::stroke_options_mut(p)),
            Primitive::Rect(p) => Some(SetStroke::stroke_options_mut(p)),
            Primitive::Tri(p) => Some(SetStroke::stroke_options_mut(p)),
            Primitive::Mesh(_)
            | Primitive::MeshVertexless(_)
            | Primitive::PathInit(_)
            | Primitive::PathFill(_)
            | Primitive::Path(_)
            | Primitive::Polygon(_)
            | Primitive::Text(_) => None,
        }
    }

    pub(crate) fn fill_options_mut(&mut self) -> Option<&mut FillOptions> {
        match self {
            Primitive::PathFill(p) => Some(SetFill::fill_options_mut(p)),
            _ => None,
        }
    }

    pub(crate) fn polygon_options_mut(&mut self) -> Option<&mut PolygonOptions> {
        match self {
            Primitive::Ellipse(p) => Some(SetPolygon::polygon_options_mut(p)),
            Primitive::PolygonInit(p) => Some(SetPolygon::polygon_options_mut(p)),
            Primitive::Quad(p) => Some(SetPolygon::polygon_options_mut(p)),
            Primitive::Rect(p) => Some(SetPolygon::polygon_options_mut(p)),
            Primitive::Tri(p) => Some(SetPolygon::polygon_options_mut(p)),
            Primitive::Arrow(_)
            | Primitive::Line(_)
            | Primitive::Mesh(_)
            | Primitive::MeshVertexless(_)
            | Primitive::PathInit(_)
            | Primitive::PathFill(_)
            | Primitive::PathStroke(_)
            | Primitive::Path(_)
            | Primitive::Polygon(_)
            | Primitive::Text(_) => None,
        }
    }

    pub(crate) fn tex_coords_mut(&mut self) -> Option<&mut Option<nannou_core::geom::Rect>> {
        match self {
            Primitive::Rect(p) => Some(SetTexCoords::tex_coords_mut(p)),
            _ => None,
        }
    }
}

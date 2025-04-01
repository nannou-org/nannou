use bevy::prelude::{Color, Component};

use nannou_core::geom::{Vec2, Vec3};

pub use self::{
    arrow::Arrow,
    ellipse::Ellipse,
    line::Line,
    mesh::PrimitiveMesh,
    path::{Path, PathFill, PathInit, PathStroke},
    polygon::{Polygon, PolygonInit},
    quad::Quad,
    rect::Rect,
    text::Text,
    tri::Tri,
};

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

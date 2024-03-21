pub mod arrow;
pub mod ellipse;
pub mod line;
pub mod mesh;
pub mod path;
pub mod polygon;
pub mod quad;
pub mod rect;
pub mod text;
pub mod texture;
pub mod tri;

use bevy::prelude::Color;
use nannou_core::geom::{Vec2, Vec3};
pub use self::arrow::Arrow;
pub use self::ellipse::Ellipse;
pub use self::line::Line;
pub use self::mesh::PrimitiveMesh;
pub use self::path::{Path, PathFill, PathInit, PathStroke};
pub use self::polygon::{Polygon, PolygonInit};
pub use self::quad::Quad;
pub use self::rect::Rect;
pub use self::text::Text;
pub use self::texture::Texture;
pub use self::tri::Tri;


type Vertex = (Vec3, Color, Vec2);


/// A wrapper around all primitive sets of properties so that they may be stored within the
/// **Draw**'s `drawing` field while they are being drawn.
///
/// This also allows us to flush all pending drawings to the mesh if `Draw::to_frame` is called
/// before their respective **Drawing** types are dropped.
#[derive(Clone, Debug)]
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
    Texture(Texture),
    Tri(Tri),
}

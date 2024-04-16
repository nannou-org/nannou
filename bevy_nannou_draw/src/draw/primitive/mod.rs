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

use bevy::pbr::Material;
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
use bevy::prelude::{Color, StandardMaterial};
use nannou_core::geom::{Vec2, Vec3};

type Vertex = (Vec3, Color, Vec2);

/// A wrapper around all primitive sets of properties so that they may be stored within the
/// **Draw**'s `drawing` field while they are being drawn.
///
/// This also allows us to flush all pending drawings to the mesh if `Draw::to_frame` is called
/// before their respective **Drawing** types are dropped.
#[derive(Clone, Debug)]
pub enum Primitive<M: Material = StandardMaterial> {
    Arrow(Arrow<M>),
    Ellipse(Ellipse<M>),
    Line(Line<M>),
    MeshVertexless(mesh::Vertexless<M>),
    Mesh(PrimitiveMesh<M>),
    PathInit(PathInit<M>),
    PathFill(PathFill<M>),
    PathStroke(PathStroke<M>),
    Path(Path<M>),
    PolygonInit(PolygonInit<M>),
    Polygon(Polygon<M>),
    Quad(Quad<M>),
    Rect(Rect<M>),
    Text(Text<M>),
    Texture(Texture<M>),
    Tri(Tri<M>),
}

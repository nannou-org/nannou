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

use crate::geom;

pub use self::arrow::Arrow;
pub use self::ellipse::Ellipse;
pub use self::line::Line;
pub use self::mesh::Mesh;
pub use self::path::{Path, PathFill, PathInit, PathStroke};
pub use self::polygon::{Polygon, PolygonInit};
pub use self::quad::Quad;
pub use self::rect::Rect;
pub use self::text::Text;
pub use self::texture::Texture;
pub use self::tri::Tri;

/// A wrapper around all primitive sets of properties so that they may be stored within the
/// **Draw**'s `drawing` field while they are being drawn.
///
/// This also allows us to flush all pending drawings to the mesh if `Draw::to_frame` is called
/// before their respective **Drawing** types are dropped.
#[derive(Clone, Debug)]
pub enum Primitive<S = geom::scalar::Default> {
    Arrow(Arrow<S>),
    Ellipse(Ellipse<S>),
    Line(Line<S>),
    MeshVertexless(mesh::Vertexless),
    Mesh(Mesh<S>),
    PathInit(PathInit<S>),
    PathFill(PathFill<S>),
    PathStroke(PathStroke<S>),
    Path(Path<S>),
    PolygonInit(PolygonInit<S>),
    Polygon(Polygon<S>),
    Quad(Quad<S>),
    Rect(Rect<S>),
    Text(Text<S>),
    Texture(Texture<S>),
    Tri(Tri<S>),
}

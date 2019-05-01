use crate::geom;

pub mod ellipse;
pub mod line;
pub mod mesh;
pub mod polygon;
pub mod polyline;
pub mod quad;
pub mod rect;
pub mod tri;

pub use self::ellipse::Ellipse;
pub use self::line::Line;
pub use self::mesh::Mesh;
pub use self::polygon::Polygon;
pub use self::polyline::Polyline;
pub use self::quad::Quad;
pub use self::rect::Rect;
pub use self::tri::Tri;

/// A wrapper around all primitive sets of properties so that they may be stored within the
/// **Draw**'s `drawing` field while they are being drawn.
///
/// This also allows us to flush all pending drawings to the mesh if `Draw::to_frame` is called
/// before their respective **Drawing** types are dropped.
#[derive(Clone, Debug)]
pub enum Primitive<S = geom::scalar::Default> {
    Ellipse(Ellipse<S>),
    Line(Line<S>),
    MeshVertexless(mesh::Vertexless),
    Mesh(Mesh<S>),
    PolygonPointless(polygon::Pointless),
    PolygonFill(Polygon<polygon::Fill, S>),
    PolygonColorPerVertex(Polygon<polygon::PerVertex, S>),
    PolylineVertexless(polyline::Vertexless),
    Polyline(Polyline<S>),
    Quad(Quad<S>),
    Rect(Rect<S>),
    Tri(Tri<S>),
}

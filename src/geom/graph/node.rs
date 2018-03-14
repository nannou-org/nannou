//! Items related to the nodes of a geometry graph.

use daggy::{self, Walker};
use daggy::petgraph::visit::{self, Visitable};
use geom;
use geom::{DefaultScalar, Graph};
use geom::graph::Edge;
use math::{self, BaseFloat, Basis3, Euler, Point2, Point3, Rad, Rotation, Vector3, Zero};
use std::collections::HashMap;
use std::{iter, ops, slice};

/// Unique index for a **Node** within a **Graph**.
pub type Index = daggy::NodeIndex<usize>;

/// Each of the primitive graphics types that may be instantiated within the graph.
///
/// Primitives that are described by a dynamic number of vertices share a single vertex buffer
/// that is owned by the graph. These variants store a range into this vertex buffer indicating
/// which slice of vertices describe it.
#[derive(Clone, Debug)]
pub enum Primitive<S = DefaultScalar> {
    Cuboid(geom::Cuboid<S>),
    Ellipse(geom::Ellipse<S>),
    Line(geom::Line<S>),
    Polygon {
        range: ops::Range<usize>,
    },
    Polyline {
        join: geom::polyline::join::Dynamic,
        cap: geom::polyline::cap::Dynamic,
        range: ops::Range<usize>,
        thickness: S,
    },
    Quad(geom::Quad<Point3<S>>),
    Rect(geom::Rect<S>),
    Tri(geom::Tri<Point3<S>>),
}

/// An iterator yielding all vertices for a primitive.
#[derive(Clone)]
pub enum PrimitiveVertices<'a, S: 'a = DefaultScalar> {
    Cuboid(geom::cuboid::Corners<'a, S>),
    Ellipse(geom::ellipse::Circumference<S>),
    Line(geom::line::Vertices<S>),
    Polygon(iter::Cloned<slice::Iter<'a, Point3<S>>>),
    // TODO: Add a `polyline::Part::vertices` method and use `polyline::Vertices` instead.
    //Polyline(geom::polyline::DynamicTriangles<slice::Iter<'a, Point3<S>>, S>),
    Quad(geom::quad::Vertices<Point3<S>>),
    Rect(geom::rect::Corners<S>),
    Tri(geom::tri::Vertices<Point3<S>>),
}

/// An iterator yielding all triangles for a primitive.
#[derive(Clone)]
pub enum PrimitiveTriangles<'a, S: 'a = DefaultScalar>
where
    S: BaseFloat,
{
    Cuboid(geom::cuboid::Triangles<'a, S>),
    Ellipse(geom::ellipse::Triangles<S>),
    Line(geom::line::Triangles<S>),
    Polygon(Option<geom::polygon::Triangles<iter::Cloned<slice::Iter<'a, Point3<S>>>>>),
    Polyline(geom::polyline::DynamicTriangles<iter::Cloned<slice::Iter<'a, Point3<S>>>>),
    Quad(geom::quad::Triangles<Point3<S>>),
    Rect(geom::rect::Triangles<S>),
    Tri(Option<geom::Tri<Point3<S>>>),
}

/// The **Node** type used within the **Graph**.
#[derive(Clone, Debug)]
pub enum Node<S = DefaultScalar>
where
    S: BaseFloat,
{
    /// A point has no vertices other than that yielded at the node's position.
    ///
    /// Useful for acting as an invisible "reference" node for controlling other children nodes.
    ///
    /// Also used to represent the graph's "origin" node.
    Point,
    /// A node representing some primitive geometric type (e.g. `Tri`, `Cuboid`, `Quad`, etc).
    Primitive(Primitive<S>),
    /// A nested Graph.
    Graph {
        graph: super::Graph<S>,
        dfs: Dfs<S>,
    },
}

/// An iterator yielding all vertices from a node.
pub enum Vertices<'a, S: 'a = DefaultScalar>
where
    S: BaseFloat,
{
    Point(Option<Point3<S>>),
    Primitive(PrimitiveVertices<'a, S>),
    Graph(geom::graph::Vertices<'a, S>),
}

/// An iterator yielding all triangles from a node.
pub enum Triangles<'a, S: 'a = DefaultScalar>
where
    S: BaseFloat,
{
    Point,
    Primitive(PrimitiveTriangles<'a, S>),
    Graph(geom::graph::Triangles<'a, S>),
}

/// An iterator yielding all vertices for a node transformed by some given transform.
pub struct TransformedVertices<'a, S: 'a = DefaultScalar>
where
    S: BaseFloat,
{
    transform: Transform<S>,
    vertices: Vertices<'a, S>,
}

/// An iterator yielding all vertices for a node transformed by some given transform.
pub struct TransformedTriangles<'a, S: 'a = DefaultScalar>
where
    S: BaseFloat,
{
    transform: Transform<S>,
    triangles: Triangles<'a, S>,
}

/// A node's resulting rotation, displacement and scale relative to the graph's origin.
///
/// A transform is calculated and applied to a node's vertices in the following order:
///
/// 1. **scale**: `1.0 * parent_scale * edge_scale`
/// 2. **rotation**: `0.0 + parent_position + edge_displacement`
/// 3. **displacement**: 0.0 + parent_orientation + edge_orientation`
#[derive(Clone, Debug, PartialEq)]
pub struct Transform<S = DefaultScalar>
where
    S: BaseFloat,
{
    /// A scaling amount along each axis.
    ///
    /// The scaling amount is multiplied onto each vertex of the node.
    pub scale: Vector3<S>,
    /// A rotation amount along each axis, describing a relative orientation.
    ///
    /// Rotates all vertices around the node origin when applied.
    pub rot: Euler<Rad<S>>,
    /// A displacement amount along each axis.
    ///
    /// This vector is added onto the position of each vertex of the node.
    pub disp: Vector3<S>,
}

/// A depth-first-search over nodes in the graph, yielding each node's unique index alongside its
/// absolute transform.
///
/// The traversal may start at any given node.
///
/// **Note:** The algorithm may not behave correctly if nodes are removed during iteration. It may
/// not necessarily visit added nodes or edges.
#[derive(Clone, Debug)]
pub struct Dfs<S = DefaultScalar>
where
    S: BaseFloat,
{
    dfs: visit::Dfs<Index, <Graph<S> as Visitable>::Map>,
    visited: TransformMap<S>,
}

impl<S> Dfs<S>
where
    S: BaseFloat,
{
    /// Create a new **Dfs** starting from the graph's origin.
    pub fn new(graph: &Graph<S>) -> Self {
        Self::start_from(graph, graph.origin())
    }

    /// Create a new **Dfs** starting from the node at the given node.
    pub fn start_from(graph: &Graph<S>, start: Index) -> Self {
        let dfs = visit::Dfs::new(graph, start);
        let visited = TransformMap::default();
        Dfs { dfs, visited }
    }

    /// Clears the visit state.
    pub fn reset(&mut self, graph: &Graph<S>) {
        self.dfs.reset(graph);
    }

    /// Keep the discovered map but clear the visit stack and restart the dfs from the given node.
    pub fn move_to(&mut self, start: Index) {
        self.dfs.move_to(start);
    }

    /// Return the tranform for the next node in the DFS.
    ///
    /// Returns `None` if the traversal is finished.
    pub fn next_transform(&mut self, graph: &Graph<S>) -> Option<(Index, Transform<S>)> {
        let n = match self.dfs.next(graph) {
            None => return None,
            Some(n) => n,
        };
        let mut transform = Transform::default();
        for (e, parent) in graph.parents(n).iter(graph) {
            let parent_transform = &self.visited[&parent];
            let edge = &graph[e];
            transform.apply_edge(parent_transform, edge);
        }
        self.visited.map.insert(n, transform.clone());
        Some((n, transform))
    }

    /// Return the vertices for the next node in the DFS.
    ///
    /// Uses `Dfs::next_transform` and `Node::vertices` internally.
    ///
    /// Returns `None` if the traversal is finished.
    pub fn next_vertices<'a>(
        &mut self,
        graph: &'a Graph<S>,
    ) -> Option<(Index, TransformedVertices<'a, S>)>
    {
        self.next_transform(graph)
            .and_then(|(n, transform)| {
                graph.node(n)
                    .map(|node| {
                        let vertices = node.vertices(&graph.vertices);
                        (n, TransformedVertices { vertices, transform })
                    })
            })
    }

    /// Return the triangles for the next node in the DFS.
    ///
    /// Uses `Dfs::next_transform` and `Node::triangles` internally.
    ///
    /// Returns `None` if the traversal is finished.
    pub fn next_triangles<'a>(
        &mut self,
        graph: &'a Graph<S>,
    ) -> Option<(Index, TransformedTriangles<'a, S>)>
    {
        self.next_transform(graph)
            .and_then(|(n, transform)| {
                graph.node(n)
                    .map(|node| {
                        let triangles = node.triangles(&graph.vertices);
                        (n, TransformedTriangles { triangles, transform })
                    })
            })
    }
}

impl<'a, S> Walker<&'a Graph<S>> for Dfs<S>
where
    S: BaseFloat,
{
    type Item = (Index, Transform<S>);
    fn walk_next(&mut self, graph: &'a Graph<S>) -> Option<Self::Item> {
        self.next_transform(graph)
    }
}

/// Mappings from node indices to their respective transform within the graph.
///
/// This is calculated via the `Graph::update_transform_map` method.
#[derive(Clone, Debug, PartialEq)]
pub struct TransformMap<S = DefaultScalar>
where
    S: BaseFloat,
{
    map: HashMap<Index, Transform<S>>
}

impl<S> Default for TransformMap<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        TransformMap {
            map: Default::default(),
        }
    }
}

impl<S> ops::Deref for TransformMap<S>
where
    S: BaseFloat,
{
    type Target = HashMap<Index, Transform<S>>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<S> Into<HashMap<Index, Transform<S>>> for TransformMap<S>
where
    S: BaseFloat,
{
    fn into(self) -> HashMap<Index, Transform<S>> {
        self.map
    }
}

impl<S> Transform<S>
where
    S: BaseFloat,
{
    /// Apply the given parent `Edge` to this transform.
    pub fn apply_edge(&mut self, parent: &Self, edge: &Edge<S>) {
        use geom::graph::edge::{Axis, Relative};
        match (edge.kind.relative, edge.kind.axis) {
            (Relative::Position, Axis::X) => self.disp.x += parent.disp.x + edge.weight,
            (Relative::Position, Axis::Y) => self.disp.y += parent.disp.y + edge.weight,
            (Relative::Position, Axis::Z) => self.disp.z += parent.disp.z + edge.weight,
            (Relative::Orientation, Axis::X) => self.rot.x += parent.rot.x + Rad(edge.weight),
            (Relative::Orientation, Axis::Y) => self.rot.y += parent.rot.y + Rad(edge.weight),
            (Relative::Orientation, Axis::Z) => self.rot.z += parent.rot.z + Rad(edge.weight),
            (Relative::Scale, Axis::X) => self.scale.x *= parent.scale.x * edge.weight,
            (Relative::Scale, Axis::Y) => self.scale.y *= parent.scale.y * edge.weight,
            (Relative::Scale, Axis::Z) => self.scale.z *= parent.scale.z * edge.weight,
        }
    }
}

impl<S> Default for Transform<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        let zero = S::zero();
        let one = S::one();
        let scale = Vector3 { x: one, y: one, z: one };
        let rot = Euler { x: Rad(zero), y: Rad(zero), z: Rad(zero) };
        let disp = Vector3 { x: zero, y: zero, z: zero };
        Transform { scale, rot, disp }
    }
}

impl<S> math::Transform<Point3<S>> for Transform<S>
where
    S: BaseFloat,
{
    fn one() -> Self {
        let one = S::one();
        let (x, y, z) = (one, one, one);
        Transform {
            scale: Vector3 { x, y, z },
            rot: Euler { x: Rad(x), y: Rad(y), z: Rad(z) },
            disp: Vector3 { x, y, z },
        }
    }

    fn look_at(eye: Point3<S>, center: Point3<S>, up: Vector3<S>) -> Self {
        unimplemented!();
    }

    fn transform_vector(&self, vec: Vector3<S>) -> Vector3<S> {
        unimplemented!();
    }

    fn inverse_transform_vector(&self, vec: Vector3<S>) -> Option<Vector3<S>> {
        unimplemented!();
    }

    fn transform_point(&self, point: Point3<S>) -> Point3<S> {
        unimplemented!();
    }

    fn concat(&self, other: &Self) -> Self {
        unimplemented!();
    }

    fn inverse_transform(&self) -> Option<Self> {
        unimplemented!();
    }
}

fn transform_point<S>(transform: &Transform<S>, mut point: Point3<S>) -> Point3<S>
where
    S: BaseFloat,
{
    // Scale the point relative to the node origin.
    point.x *= transform.scale.x;
    point.y *= transform.scale.y;
    point.z *= transform.scale.z;
    // Rotate the point around the node origin.
    point = Basis3::from(transform.rot).rotate_point(point);
    // Displace the point from the node origin.
    point += transform.disp;
    point
}

impl<'a, S> Iterator for TransformedVertices<'a, S>
where
    S: BaseFloat,
{
    type Item = Point3<S>;
    fn next(&mut self) -> Option<Self::Item> {
        self.vertices
            .next()
            .map(|point| transform_point(&self.transform, point))
    }
}

impl<'a, S> Iterator for TransformedTriangles<'a, S>
where
    S: BaseFloat,
{
    type Item = geom::Tri<Point3<S>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.triangles
            .next()
            .map(|tri| tri.map(|point| transform_point(&self.transform, point)))
    }
}

impl<S> Primitive<S>
where
    S: BaseFloat,
{
    /// The `Cuboid` that bounds the primitive.
    ///
    /// Returns `None` if the primitive is a polygon or polyline with no points.
    pub fn bounding_cuboid(&self, vertices: &[Point3<S>]) -> Option<geom::Cuboid<S>> {
        match *self {
            Primitive::Cuboid(ref cuboid) => {
                geom::bounding_cuboid(cuboid.corners_iter())
            },
            Primitive::Ellipse(ref ellipse) => {
                let (x, y) = (ellipse.rect.x, ellipse.rect.y);
                let z = geom::Range::new(Zero::zero(), Zero::zero());
                Some(geom::Cuboid { x, y, z })
            },
            Primitive::Line(ref line) => {
                let q = line.quad_corners();
                let vertices = q.iter()
                    .map(|p| {
                        let x = p.x;
                        let y = p.y;
                        let z = Zero::zero();
                        Point3 { x, y, z }
                    });
                geom::bounding_cuboid(vertices)
            },
            Primitive::Polygon { ref range } => {
                let vertices = vertices[range.clone()].iter().cloned();
                geom::bounding_cuboid(vertices)
            },
            Primitive::Polyline { join, cap, ref range, thickness } => {
                let vertices = vertices[range.clone()]
                    .iter()
                    .cloned()
                    .map(|p| Point2 { x: p.x, y: p.y });
                geom::Polyline::new(cap, join, vertices, thickness)
                    .bounding_rect()
                    .map(|rect| {
                        let (x, y) = (rect.x, rect.y);
                        let z = geom::Range::new(Zero::zero(), Zero::zero());
                        geom::Cuboid { x, y, z }
                    })
            },
            Primitive::Quad(ref quad) => {
                let vertices = quad.iter().cloned();
                geom::bounding_cuboid(vertices)
            },
            Primitive::Rect(ref rect) => {
                let (x, y) = (rect.x, rect.y);
                let z = geom::Range::new(Zero::zero(), Zero::zero());
                Some(geom::Cuboid { x, y, z })
            },
            Primitive::Tri(ref tri) => {
                let r = tri.bounding_rect();
                let (x, y) = (r.x, r.y);
                let z = geom::Range::new(Zero::zero(), Zero::zero());
                Some(geom::Cuboid { x, y, z })
            },
        }
    }

    /// Produce an iterator yielding all vertices within the primitive.
    pub fn vertices<'a>(&'a self, vertices: &'a [Point3<S>]) -> PrimitiveVertices<'a, S> {
        match *self {
            Primitive::Cuboid(ref cuboid) => {
                PrimitiveVertices::Cuboid(cuboid.corners_iter())
            },
            Primitive::Ellipse(ref ellipse) => {
                PrimitiveVertices::Ellipse(ellipse.circumference())
            },
            Primitive::Line(ref line) => {
                PrimitiveVertices::Line(line.quad_corners_iter())
            },
            Primitive::Polygon { ref range } => {
                let slice = &vertices[range.clone()];
                PrimitiveVertices::Polygon(slice.iter().cloned())
            },
            Primitive::Polyline { .. } => {
                unimplemented!();
            },
            Primitive::Quad(ref quad) => {
                PrimitiveVertices::Quad(quad.vertices())
            },
            Primitive::Rect(ref rect) => {
                PrimitiveVertices::Rect(rect.corners_iter())
            },
            Primitive::Tri(ref tri) => {
                PrimitiveVertices::Tri(tri.vertices())
            },
        }
    }

    /// Produce an iterator yielding all triangles within the primitive.
    pub fn triangles<'a>(&'a self, vertices: &'a [Point3<S>]) -> PrimitiveTriangles<'a, S> {
        match *self {
            Primitive::Cuboid(ref cuboid) => {
                PrimitiveTriangles::Cuboid(cuboid.triangles_iter())
            },
            Primitive::Ellipse(ref ellipse) => {
                PrimitiveTriangles::Ellipse(ellipse.triangles())
            },
            Primitive::Line(ref line) => {
                PrimitiveTriangles::Line(line.triangles_iter())
            },
            Primitive::Polygon { ref range } => {
                let slice = &vertices[range.clone()];
                let polygon = geom::Polygon::new(slice.iter().cloned());
                PrimitiveTriangles::Polygon(polygon.triangles())
            },
            Primitive::Polyline { .. } => {
                unimplemented!();
            },
            Primitive::Quad(ref quad) => {
                PrimitiveTriangles::Quad(quad.triangles_iter())
            },
            Primitive::Rect(ref rect) => {
                PrimitiveTriangles::Rect(rect.triangles_iter())
            },
            Primitive::Tri(ref tri) => {
                PrimitiveTriangles::Tri(Some(*tri))
            },
        }
    }
}

// A small function used for mapping 2D points to 3D ones within the PrimitiveVertices and
// PrimitiveTriangles iterators.
fn pt2_to_pt3<S>(p: Point2<S>) -> Point3<S>
where
    S: BaseFloat,
{
    let x = p.x;
    let y = p.y;
    let z = Zero::zero();
    Point3 { x, y, z }
}

impl<'a, S> Iterator for PrimitiveVertices<'a, S>
where
    S: BaseFloat,
{
    type Item = Point3<S>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            PrimitiveVertices::Cuboid(ref mut corners) => {
                corners.next()
            },
            PrimitiveVertices::Ellipse(ref mut circumference) => {
                circumference.next().map(pt2_to_pt3)
            },
            PrimitiveVertices::Line(ref mut vertices) => {
                vertices.next().map(pt2_to_pt3)
            },
            PrimitiveVertices::Polygon(ref mut vertices) => {
                vertices.next()
            },
            // PrimitiveVertices::Polyline(ref mut vertices) => {
            //     vertices.next()
            // },
            PrimitiveVertices::Quad(ref mut vertices) => {
                vertices.next()
            },
            PrimitiveVertices::Rect(ref mut corners) => {
                corners.next().map(pt2_to_pt3)
            },
            PrimitiveVertices::Tri(ref mut vertices) => {
                vertices.next()
            },
        }
    }
}

impl<'a, S> Iterator for PrimitiveTriangles<'a, S>
where
    S: BaseFloat,
{
    type Item = geom::Tri<Point3<S>>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            PrimitiveTriangles::Cuboid(ref mut tris) => {
                tris.next()
            },
            PrimitiveTriangles::Ellipse(ref mut tris) => {
                tris.next().map(|t| t.map(pt2_to_pt3))
            },
            PrimitiveTriangles::Line(ref mut tris) => {
                tris.next().map(|t| t.map(pt2_to_pt3))
            },
            PrimitiveTriangles::Polygon(ref mut tris) => {
                tris.as_mut().and_then(|ts| ts.next())
            },
            PrimitiveTriangles::Polyline(ref mut _tris) => {
                unimplemented!();
                // TODO: Implement Iterator for DynamicTriangles
                //tris.next().map(|t| t.map(pt2_to_pt3))
            },
            PrimitiveTriangles::Quad(ref mut tris) => {
                tris.next()
            },
            PrimitiveTriangles::Rect(ref mut tris) => {
                tris.next().map(|t| t.map(pt2_to_pt3))
            },
            PrimitiveTriangles::Tri(ref mut tri) => {
                tri.take()
            },
        }
    }
}

impl<'a, S> Iterator for Vertices<'a, S>
where
    S: BaseFloat,
{
    type Item = Point3<S>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            // A node point is always at the node's origin.
            Vertices::Point(ref mut point) => {
                point.take()
            },
            Vertices::Primitive(ref mut vertices) => {
                vertices.next()
            },
            Vertices::Graph(ref mut vertices) => {
                vertices.next()
            },
        }
    }
}

impl<'a, S> Iterator for Triangles<'a, S>
where
    S: BaseFloat,
{
    type Item = geom::Tri<Point3<S>>;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            Triangles::Point => {
                None
            },
            Triangles::Primitive(ref mut triangles) => {
                triangles.next()
            },
            Triangles::Graph(ref mut triangles) => {
                triangles.next()
            },
        }
    }
}

impl<S> Node<S>
where
    S: BaseFloat,
{
    /// The `Cuboid` that bounds the primitive.
    ///
    /// Returns `None` if the primitive is a polygon, polyline or graph with no points.
    pub fn bounding_cuboid(&self, vertices: &[Point3<S>]) -> Option<geom::Cuboid<S>> {
        match *self {
            Node::Point => {
                let zero = Zero::zero();
                let x = geom::Range::new(zero, zero);
                let y = geom::Range::new(zero, zero);
                let z = geom::Range::new(zero, zero);
                Some(geom::Cuboid { x, y, z })
            },
            Node::Primitive(ref primitive) => primitive.bounding_cuboid(vertices),
            Node::Graph { ref graph, .. } => graph.bounding_cuboid(),
        }
    }

    /// Produce an iterator yielding all vertices within the node.
    pub fn vertices<'a>(&'a self, vertices: &'a [Point3<S>]) -> Vertices<'a, S> {
        match *self {
            Node::Point => {
                let zero = Zero::zero();
                let (x, y, z) = (zero, zero, zero);
                Vertices::Point(Some(Point3 { x, y, z }))
            },
            Node::Primitive(ref primitive) => {
                Vertices::Primitive(primitive.vertices(vertices))
            },
            Node::Graph { .. } => {
                unimplemented!();
            },
        }
    }

    /// Produce an iterator yielding all triangles within the node.
    pub fn triangles<'a>(&'a self, vertices: &'a [Point3<S>]) -> Triangles<'a, S> {
        match *self {
            Node::Point => {
                Triangles::Point
            },
            Node::Primitive(ref primitive) => {
                Triangles::Primitive(primitive.triangles(vertices))
            },
            Node::Graph { .. } => {
                unimplemented!();
            }
        }
    }
}

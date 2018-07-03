//! Items related to the nodes of a geometry graph.

use daggy::petgraph::visit::{self, Visitable};
use daggy::{self, Walker};
use geom;
use geom::graph::Edge;
use geom::{scalar, Graph, Point3, Vector3};
use math::{self, BaseFloat, Basis3, Euler, Rad, Rotation};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops;

/// Unique index for a **Node** within a **Graph**.
pub type Index = daggy::NodeIndex<usize>;

/// The **Node** type used within the **Graph**.
#[derive(Clone, Debug)]
pub enum Node<S = scalar::Default>
where
    S: BaseFloat,
{
    /// A point has no vertices other than that yielded at the node's position.
    ///
    /// Useful for acting as an invisible "reference" node for controlling other children nodes.
    ///
    /// Also used to represent the graph's "origin" node.
    Point,
    /// A nested Graph.
    Graph { graph: super::Graph<S>, dfs: Dfs<S> },
}

/// An iterator yielding all vertices for a node transformed by some given transform.
#[derive(Clone, Debug)]
pub struct TransformedVertices<I, S = scalar::Default>
where
    S: BaseFloat,
{
    transform: PreparedTransform<S>,
    vertices: I,
}

/// An iterator yielding all vertices for a node transformed by some given transform.
#[derive(Clone, Debug)]
pub struct TransformedTriangles<I, V, S = scalar::Default>
where
    S: BaseFloat,
{
    transform: PreparedTransform<S>,
    triangles: I,
    _vertex: PhantomData<V>,
}

/// A node's resulting rotation, displacement and scale relative to the graph's origin.
///
/// A transform is calculated and applied to a node's vertices in the following order:
///
/// 1. **scale**: `1.0 * parent_scale * edge_scale`
/// 2. **rotation**: `0.0 + parent_position + edge_displacement`
/// 3. **displacement**: 0.0 + parent_orientation + edge_orientation`
#[derive(Clone, Debug, PartialEq)]
pub struct Transform<S = scalar::Default>
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

/// A node's resulting rotation, displacement and scale relative to the graph's origin.
///
/// The same as **Transfrom** but the euler has been converted to a matrix for more efficient
/// application.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedTransform<S = scalar::Default> {
    /// A scaling amount along each axis.
    ///
    /// The scaling amount is multiplied onto each vertex of the node.
    pub scale: Vector3<S>,
    /// A rotation amount along each axis, describing a relative orientation.
    ///
    /// Rotates all vertices around the node origin when applied.
    pub rot: Basis3<S>,
    /// A displacement amount along each axis.
    ///
    /// This vector is added onto the position of each vertex of the node.
    pub disp: Vector3<S>,
}

/// Mappings from node indices to their respective transform within the graph.
///
/// This is calculated via the `Graph::update_transform_map` method.
#[derive(Clone, Debug, PartialEq)]
pub struct TransformMap<S = scalar::Default>
where
    S: BaseFloat,
{
    map: HashMap<Index, Transform<S>>,
}

/// A depth-first-search over nodes in the graph, yielding each node's unique index alongside its
/// absolute transform.
///
/// The traversal may start at any given node.
///
/// **Note:** The algorithm may not behave correctly if nodes are removed during iteration. It may
/// not necessarily visit added nodes or edges.
#[derive(Clone, Debug)]
pub struct Dfs<S = scalar::Default>
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
        self.dfs.move_to(graph.origin());
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
    /// Uses `Dfs::next_transform` internally.
    ///
    /// Returns `None` if the traversal is finished.
    pub fn next_vertices<F, I>(
        &mut self,
        graph: &Graph<S>,
        vertices_fn: F,
    ) -> Option<(Index, TransformedVertices<I::IntoIter, S>)>
    where
        F: FnOnce(&Index) -> I,
        I: IntoIterator,
        I::Item: ApplyTransform<S>,
    {
        self.next_transform(graph).map(|(n, transform)| {
            let vertices = vertices_fn(&n);
            let vertices = vertices.into_iter();
            (n, transform.vertices(vertices))
        })
    }

    /// Return the triangles for the next node in the DFS.
    ///
    /// Uses `Dfs::next_transform` and `Node::triangles` internally.
    ///
    /// Returns `None` if the traversal is finished.
    pub fn next_triangles<F, I, V>(
        &mut self,
        graph: &Graph<S>,
        triangles_fn: F,
    ) -> Option<(Index, TransformedTriangles<I::IntoIter, V, S>)>
    where
        F: FnOnce(&Index) -> I,
        I: IntoIterator<Item = geom::Tri<V>>,
        V: geom::Vertex + ApplyTransform<S>,
    {
        self.next_transform(graph).map(|(n, transform)| {
            let triangles = triangles_fn(&n);
            let triangles = triangles.into_iter();
            (n, transform.triangles(triangles))
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
            (Relative::Orientation, Axis::X) => self.rot.x += parent.rot.x - Rad(edge.weight),
            (Relative::Orientation, Axis::Y) => self.rot.y += parent.rot.y - Rad(edge.weight),
            (Relative::Orientation, Axis::Z) => self.rot.z += parent.rot.z - Rad(edge.weight),
            (Relative::Scale, Axis::X) => self.scale.x *= parent.scale.x * edge.weight,
            (Relative::Scale, Axis::Y) => self.scale.y *= parent.scale.y * edge.weight,
            (Relative::Scale, Axis::Z) => self.scale.z *= parent.scale.z * edge.weight,
        }
    }

    /// Prepare this transform for application.
    pub fn prepare(self) -> PreparedTransform<S> {
        let Transform { disp, rot, scale } = self;
        let rot = Basis3::from(rot);
        PreparedTransform { disp, rot, scale }
    }

    /// Transform the given vertices.
    pub fn vertices<I>(self, vertices: I) -> TransformedVertices<I::IntoIter, S>
    where
        I: IntoIterator,
        I::Item: ApplyTransform<S>,
    {
        let transform = self.prepare();
        let vertices = vertices.into_iter();
        TransformedVertices {
            transform,
            vertices,
        }
    }

    /// Transform the given vertices.
    pub fn triangles<I, V>(self, triangles: I) -> TransformedTriangles<I::IntoIter, V, S>
    where
        I: IntoIterator<Item = geom::Tri<V>>,
        V: geom::Vertex + ApplyTransform<S>,
    {
        let transform = self.prepare();
        let triangles = triangles.into_iter();
        let _vertex = PhantomData;
        TransformedTriangles {
            transform,
            triangles,
            _vertex,
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
        let scale = Vector3 {
            x: one,
            y: one,
            z: one,
        };
        let rot = Euler {
            x: Rad(zero),
            y: Rad(zero),
            z: Rad(zero),
        };
        let disp = Vector3 {
            x: zero,
            y: zero,
            z: zero,
        };
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
            rot: Euler {
                x: Rad(x),
                y: Rad(y),
                z: Rad(z),
            },
            disp: Vector3 { x, y, z },
        }
    }

    fn look_at(_eye: Point3<S>, _center: Point3<S>, _up: Vector3<S>) -> Self {
        unimplemented!();
    }

    fn transform_vector(&self, _vec: Vector3<S>) -> Vector3<S> {
        unimplemented!();
    }

    fn inverse_transform_vector(&self, _vec: Vector3<S>) -> Option<Vector3<S>> {
        unimplemented!();
    }

    fn transform_point(&self, _point: Point3<S>) -> Point3<S> {
        unimplemented!();
    }

    fn concat(&self, _other: &Self) -> Self {
        unimplemented!();
    }

    fn inverse_transform(&self) -> Option<Self> {
        unimplemented!();
    }
}

/// Apply the given transform to the given 3D point.
pub fn transform_point<S>(transform: &PreparedTransform<S>, mut point: Point3<S>) -> Point3<S>
where
    S: BaseFloat,
{
    // Scale the point relative to the node origin.
    point.x *= transform.scale.x;
    point.y *= transform.scale.y;
    point.z *= transform.scale.z;
    // Rotate the point around the node origin.
    point = transform.rot.rotate_point(point.into()).into();
    // Displace the point from the node origin.
    point += transform.disp;
    point
}

/// Vertex types which may apply a transform and produce a resulting transform.
pub trait ApplyTransform<S>
where
    S: BaseFloat,
{
    /// Apply the given transform and return the result.
    fn apply_transform(self, transform: &PreparedTransform<S>) -> Self;
}

impl<S> ApplyTransform<S> for Point3<S>
where
    S: BaseFloat,
{
    fn apply_transform(self, transform: &PreparedTransform<S>) -> Self {
        transform_point(transform, self)
    }
}

impl<I, S> Iterator for TransformedVertices<I, S>
where
    I: Iterator,
    I::Item: ApplyTransform<S>,
    S: BaseFloat,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.vertices
            .next()
            .map(|vertex| vertex.apply_transform(&self.transform))
    }
}

impl<I, V, S> Iterator for TransformedTriangles<I, V, S>
where
    I: Iterator<Item = geom::Tri<V>>,
    V: geom::Vertex + ApplyTransform<S>,
    S: BaseFloat,
{
    type Item = geom::Tri<V>;
    fn next(&mut self) -> Option<Self::Item> {
        self.triangles
            .next()
            .map(|tri| tri.map_vertices(|vertex| vertex.apply_transform(&self.transform)))
    }
}

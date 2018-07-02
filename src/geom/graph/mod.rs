use daggy::petgraph::visit::{GraphBase, IntoNeighbors, Visitable};
use daggy::{self, Walker};
use geom;
use math::{BaseFloat, Point3, Vector3};
use std::iter;
use std::ops;
use std::option;

pub use self::edge::Edge;
pub use self::node::Node;

pub mod edge;
pub mod node;

/// A composition of geometry described by an acyclic directed graph.
///
/// The `Node`s within a graph may describe some primitive geometry (e.g. `Line`, `Cuboid`, etc) or
/// may contain other `Graph`s. This allows graphs to be composed of other graphs, which may then
/// be composed with other graphs, etc.
///
/// The `Edge`s within a graph describe the relationships between the nodes. They allow for
/// describing the **position**, **orientation** and **scale** of nodes relative to others.
///
/// All `Node`s other than the graph's "origin" node must have at least one parent, but may never
/// have more than one parent of each `edge::Kind`.
#[derive(Clone, Debug)]
pub struct Graph<S = geom::scalar::Default>
where
    S: BaseFloat,
{
    dag: Dag<S>,
    origin: node::Index,
}

/// The `daggy` "directed acyclic graph" type used within the geometry graph.
pub type Dag<S = geom::scalar::Default> = daggy::Dag<Node<S>, Edge<S>, usize>;

/// A **Walker** over some node's parent nodes.
pub struct Parents<S = geom::scalar::Default>
where
    S: BaseFloat,
{
    parents: daggy::Parents<Node<S>, Edge<S>, usize>,
}

/// A **Walker** over some node's children nodes.
pub struct Children<S = geom::scalar::Default>
where
    S: BaseFloat,
{
    children: daggy::Children<Node<S>, Edge<S>, usize>,
}

/// The slice of all nodes stored within the graph.
pub type RawNodes<'a, S = geom::scalar::Default> = daggy::RawNodes<'a, Node<S>, usize>;

/// The slice of all edges stored within the graph.
pub type RawEdges<'a, S = geom::scalar::Default> = daggy::RawEdges<'a, Edge<S>, usize>;

/// An alias for our Graph's **WouldCycle** error type.
pub type WouldCycle<S = geom::scalar::Default> = daggy::WouldCycle<Edge<S>>;

/// An alias for our Graph's recursive walker.
pub type RecursiveWalk<F, S = geom::scalar::Default> = daggy::walker::Recursive<Graph<S>, F>;

// An alias for the iterator yielding three parents that may or may not exist.
//
// Used for the `Position`, `Orientation` and `Scale` parents.
type ThreeNodes = iter::Chain<
    iter::Chain<option::IntoIter<node::Index>, option::IntoIter<node::Index>>,
    option::IntoIter<node::Index>,
>;

/// An alias for the iterator yielding **X**, **Y** and **Z** **Position** parents.
pub type PositionParents = ThreeNodes;
/// An alias for the iterator yielding **X**, **Y** and **Z** **Orientation** parents.
pub type OrientationParents = ThreeNodes;
/// An alias for the iterator yielding **X**, **Y** and **Z** **Scale** parents.
pub type ScaleParents = ThreeNodes;
/// An alias for the iterator yielding **Position**, **Orientation** and **Scale** parents over the
/// **X** axis.
pub type XParents = ThreeNodes;
/// An alias for the iterator yielding **Position**, **Orientation** and **Scale** parents over the
/// **Y** axis.
pub type YParents = ThreeNodes;
/// An alias for the iterator yielding **Position**, **Orientation** and **Scale** parents over the
/// **Z** axis.
pub type ZParents = ThreeNodes;

/// A **Walker** type yielding all transformed vertices of all nodes within the graph.
pub struct WalkVertices<'a, F, I, S: 'a = geom::scalar::Default>
where
    F: Fn(&node::Index) -> I,
    I: IntoIterator,
    S: BaseFloat,
{
    dfs: &'a mut node::Dfs<S>,
    vertices_fn: F,
    node: Option<node::TransformedVertices<I::IntoIter, S>>,
}

/// A **Walker** type yielding all transformed triangles of all nodes within the graph.
pub struct WalkTriangles<'a, F, I, V, S: 'a = geom::scalar::Default>
where
    F: Fn(&node::Index) -> I,
    I: IntoIterator,
    S: BaseFloat,
{
    dfs: &'a mut node::Dfs<S>,
    triangles_fn: F,
    node: Option<node::TransformedTriangles<I::IntoIter, V, S>>,
}

/// An iterator yielding all vertices of all nodes within the graph.
///
/// Uses the `WalkVertices` internally.
pub struct Vertices<'a, 'b, F, I, S: 'a + 'b = geom::scalar::Default>
where
    F: Fn(&node::Index) -> I,
    I: IntoIterator,
    S: BaseFloat,
{
    graph: &'a Graph<S>,
    walker: WalkVertices<'b, F, I, S>,
}

/// An iterator yielding all triangles of all nodes within the graph.
///
/// Uses the `WalkTriangles` internally.
pub struct Triangles<'a, 'b, F, I, V, S: 'a + 'b = geom::scalar::Default>
where
    F: Fn(&node::Index) -> I,
    I: IntoIterator,
    S: BaseFloat,
{
    graph: &'a Graph<S>,
    walker: WalkTriangles<'b, F, I, V, S>,
}

impl<'a, F, I, S> WalkVertices<'a, F, I, S>
where
    F: Fn(&node::Index) -> I,
    I: IntoIterator,
    I::Item: node::ApplyTransform<S>,
    S: BaseFloat,
{
    /// Return the next vertex in the graph.
    pub fn next(&mut self, graph: &Graph<S>) -> Option<I::Item> {
        let WalkVertices {
            ref mut dfs,
            ref vertices_fn,
            ref mut node,
        } = *self;
        loop {
            if let Some(v) = node.as_mut().and_then(|n| n.next()) {
                return Some(v);
            }
            match dfs.next_vertices(graph, vertices_fn) {
                Some((_n, vs)) => *node = Some(vs),
                None => return None,
            }
        }
    }
}

impl<'a, F, I, V, S> WalkTriangles<'a, F, I, V, S>
where
    F: Fn(&node::Index) -> I,
    I: IntoIterator<Item = geom::Tri<V>>,
    V: geom::Vertex + node::ApplyTransform<S>,
    S: BaseFloat,
{
    /// Return the next vertex in the graph.
    pub fn next(&mut self, graph: &Graph<S>) -> Option<geom::Tri<V>> {
        let WalkTriangles {
            ref mut dfs,
            ref triangles_fn,
            ref mut node,
        } = *self;
        loop {
            if let Some(v) = node.as_mut().and_then(|n| n.next()) {
                return Some(v);
            }
            match dfs.next_triangles(graph, triangles_fn) {
                Some((_n, ts)) => *node = Some(ts),
                None => return None,
            }
        }
    }
}

impl<'a, 'b, F, I, S> Iterator for Vertices<'a, 'b, F, I, S>
where
    F: Fn(&node::Index) -> I,
    I: IntoIterator,
    I::Item: node::ApplyTransform<S>,
    S: BaseFloat,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.graph)
    }
}

impl<'a, 'b, F, I, V, S> Iterator for Triangles<'a, 'b, F, I, V, S>
where
    F: Fn(&node::Index) -> I,
    I: IntoIterator<Item = geom::Tri<V>>,
    V: geom::Vertex + node::ApplyTransform<S>,
    S: BaseFloat,
{
    type Item = geom::Tri<V>;
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.graph)
    }
}

impl<S> Graph<S>
where
    S: BaseFloat,
{
    /// Construct a new empty `Graph` with a single "origin" node.
    ///
    /// The "origin" is the "parent-most" of all nodes. Its transform is always equal to
    /// `Transform::default()`.
    ///
    /// Calling this is the same as calling `Graph::default()`.
    pub fn new() -> Self {
        Graph::default()
    }

    /// The **node::Index** of the origin node.
    pub fn origin(&self) -> node::Index {
        self.origin
    }

    /// Construct the graph with pre-allocated buffers for the given `nodes`, `edges` and
    /// `vertices` capacities.
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        let mut dag = Dag::with_capacity(nodes, edges);
        let origin = dag.add_node(Node::Point);
        Graph { dag, origin }
    }

    /// The total number of **Node**s in the **Graph**.
    pub fn node_count(&self) -> usize {
        self.dag.node_count()
    }

    /// The total number of **Edge**s in the **Graph**.
    pub fn edge_count(&self) -> usize {
        self.dag.edge_count()
    }

    /// Borrow the node at the given **node::Index** if there is one.
    pub fn node(&self, idx: node::Index) -> Option<&Node<S>> {
        self.dag.node_weight(idx)
    }

    /// Determine the full **Transform** for the **Node**.
    ///
    /// Returns **None** if the node does not exist.
    pub fn node_transform(&self, idx: node::Index) -> Option<node::Transform<S>> {
        // If there is no node for the index, return **None**.
        if self.node(idx).is_none() {
            return None;
        }

        // Calculate the transform.
        let mut transform = node::Transform::default();
        for (e, parent) in self.parents(idx).iter(self) {
            let parent_transform = self
                .node_transform(parent)
                .expect("no node for yielded parent");
            let edge = &self[e];
            transform.apply_edge(&parent_transform, edge);
        }

        Some(transform)
    }

    /// Transform the given vertices with the given node's **Transform**.
    ///
    /// This method uses the **node_transform** method internally.
    ///
    /// Returns **None** if the node does not exist.
    pub fn node_vertices<I>(
        &self,
        idx: node::Index,
        vertices: I,
    ) -> Option<node::TransformedVertices<I::IntoIter, S>>
    where
        I: IntoIterator,
        I::Item: node::ApplyTransform<S>,
    {
        self.node_transform(idx)
            .map(|transform| transform.vertices(vertices.into_iter()))
    }

    /// Transform the given triangles with the given node's **Transform**.
    ///
    /// This method uses the **node_transform** method internally.
    ///
    /// Returns **None** if the node does not exist.
    pub fn node_triangles<I, V>(
        &self,
        idx: node::Index,
        triangles: I,
    ) -> Option<node::TransformedTriangles<I::IntoIter, V, S>>
    where
        I: IntoIterator<Item = geom::Tri<V>>,
        V: geom::Vertex + node::ApplyTransform<S>,
    {
        self.node_transform(idx)
            .map(|transform| transform.triangles(triangles.into_iter()))
    }

    /// Borrow the edge at the given **edge::Index** if there is one.
    pub fn edge(&self, idx: edge::Index) -> Option<&Edge<S>> {
        self.dag.edge_weight(idx)
    }

    /// The slice containing all nodes.
    pub fn raw_nodes(&self) -> RawNodes<S> {
        self.dag.raw_nodes()
    }

    /// The slice containing all edges.
    pub fn raw_edges(&self) -> RawEdges<S> {
        self.dag.raw_edges()
    }

    /// Removes all **Node**s, **Edge**s and vertices from the **Graph** and resets the origin node.
    ///
    /// This does not de-allocate any buffers and should retain capacity. To drop all inner
    /// buffers, use `mem::replace` with a new empty **Graph**.
    pub fn clear(&mut self) {
        self.dag.clear();
        let origin = self.dag.add_node(Node::Point);
        self.origin = origin;
    }

    /// Return the parent and child nodes on either end of the **Edge** at the given index.
    pub fn edge_endpoints(&self, idx: edge::Index) -> Option<(node::Index, node::Index)> {
        self.dag.edge_endpoints(idx)
    }

    /// A **Walker** type that recursively walks the **Graph** using the given `recursive_fn`.
    ///
    /// **Panics** If the given start index does not exist within the **Graph**.
    pub fn recursive_walk<F>(&self, start: node::Index, recursive_fn: F) -> RecursiveWalk<F, S>
    where
        F: FnMut(&Self, node::Index) -> Option<(edge::Index, node::Index)>,
    {
        RecursiveWalk::new(start, recursive_fn)
    }

    /// Borrow the inner `daggy::Dag` (directed acyclic graph) upon which `Graph` is built.
    ///
    /// Can be useful/necessary for utilising some of the `daggy::Walker` types.
    pub fn dag(&self) -> &Dag<S> {
        &self.dag
    }

    /// Add the given **Node** to the graph.
    ///
    /// The created node will be a child of the graph's origin node.
    ///
    /// Returns the index of the new node.
    pub fn add_node(&mut self, node: Node<S>) -> node::Index {
        let origin = self.origin();
        let zero = S::zero();
        let (x, y, z) = (zero, zero, zero);
        let edges = edge::displace(Vector3 { x, y, z });
        let (_es, n) = self.add_child(origin, edges.iter().cloned(), node);
        n
    }

    // Remove and return the **Edge** at the given index.
    //
    // Return `None` if it didn't exist.
    fn remove_edge(&mut self, idx: edge::Index) -> Option<Edge<S>> {
        self.dag.remove_edge(idx)
    }

    /// Set the given **Edge** within the graph.
    ///
    /// The added edge will be in the direction `a` -> `b`
    ///
    /// There may only ever be one **Edge** of the given variant between `a` -> `b`.
    ///
    /// Checks if the edge would create a cycle in the **Graph**.
    ///
    /// If adding the edge **would not** cause the graph to cycle, the edge will be added and its
    /// `edge::Index` returned.
    ///
    /// If adding the edge **would** cause the graph to cycle, the edge will not be added and
    /// instead a `WouldCycle` error with the given weight will be returned.
    ///
    /// **Panics** if either `a` or `b` do not exist within the **Graph**.
    ///
    /// **Panics** if the **Graph** is at the maximum number of nodes for its index type.
    pub fn set_edge(
        &mut self,
        a: node::Index,
        b: node::Index,
        edge: Edge<S>,
    ) -> Result<edge::Index, WouldCycle<S>> {
        // Check to see if the node already has some matching incoming edge.
        // Keep it if it's the one we want. Otherwise, remove any incoming edge that matches the given
        // edge kind but isn't coming from the node that we desire.
        let mut parents = self.parents(b);
        let mut already_set = None;

        while let Some((in_edge_idx, in_node_idx)) = parents.walk_next(self) {
            if edge.kind == self[in_edge_idx].kind {
                if in_node_idx == a {
                    self.dag[in_edge_idx].weight = edge.weight;
                    already_set = Some(in_edge_idx);
                } else {
                    self.remove_edge(in_edge_idx);
                }
                // Note that we only need to check for *one* edge as there can only ever be one
                // parent edge of any kind for each node. We know this, as this method is the only
                // function used by a public method that adds edges.
                break;
            }
        }

        // If we don't already have an incoming edge from the requested parent, add one.
        match already_set {
            Some(edge_idx) => Ok(edge_idx),
            None => self.dag.add_edge(a, b, edge),
        }
    }

    /// Add the given node as a child to the given `parent` connected via the given edges.
    ///
    /// There may only ever be one **Edge** of the given variant between `a` -> `b`.
    ///
    /// **Panics** if:
    ///
    /// - `parent` does not exist within the **Graph**.
    /// - `edges` does not contain at least one **Edge<S>**.
    /// - the **Graph** is at the maximum number of nodes for its index type.
    pub fn add_child<E>(
        &mut self,
        parent: node::Index,
        edges: E,
        node: Node<S>,
    ) -> (edge::Indices, node::Index)
    where
        E: IntoIterator<Item = Edge<S>>,
    {
        let n = self.dag.add_node(node);
        let mut edges = edges.into_iter().peekable();
        match edges.next() {
            None => panic!("`edges` must contain at least one edge"),
            Some(first) => {
                let edges = Some(first).into_iter().chain(edges).map(|e| (parent, n, e));
                let edge_indices = self
                    .dag
                    .add_edges(edges)
                    .expect("cannot create a cycle when adding a new child");
                (edge_indices, n)
            }
        }
    }

    /// Add the given children to the given `parent` connected via the given edges for each child.
    ///
    /// Returns an iterator yielding the node and edge indices for each child that was added.
    ///
    /// There may only ever be one **Edge** of the given variant between `a` -> `b`.
    ///
    /// **Panics** if:
    ///
    /// - `parent` does not exist within the **Graph**.
    /// - any of the child `edges` iterators do not contain at least one **Edge<S>**.
    /// - the **Graph** is at the maximum number of nodes for its index type.
    pub fn add_children<C, E>(
        &mut self,
        parent: node::Index,
        children: C,
    ) -> Vec<(edge::Indices, node::Index)>
    where
        C: IntoIterator<Item = (E, Node<S>)>,
        E: IntoIterator<Item = Edge<S>>,
    {
        let mut children_indices = vec![];
        for (edges, node) in children {
            let child_indices = self.add_child(parent, edges, node);
            children_indices.push(child_indices);
        }
        children_indices
    }

    /// Produce a walker yielding all vertices from all nodes within the graph in order of
    /// discovery within a depth-first-search.
    pub fn walk_vertices<'a, F, I>(
        &self,
        dfs: &'a mut node::Dfs<S>,
        vertices_fn: F,
    ) -> WalkVertices<'a, F, I, S>
    where
        F: Fn(&node::Index) -> I,
        I: IntoIterator,
        I::Item: node::ApplyTransform<S>,
    {
        let node = None;
        WalkVertices {
            dfs,
            vertices_fn,
            node,
        }
    }

    /// Produce a walker yielding all vertices from all nodes within the graph in order of
    /// discovery within a depth-first-search.
    pub fn walk_triangles<'a, F, I, V>(
        &self,
        dfs: &'a mut node::Dfs<S>,
        triangles_fn: F,
    ) -> WalkTriangles<'a, F, I, V, S>
    where
        F: Fn(&node::Index) -> I,
        I: IntoIterator<Item = geom::Tri<V>>,
        V: geom::Vertex + node::ApplyTransform<S>,
    {
        let node = None;
        WalkTriangles {
            dfs,
            triangles_fn,
            node,
        }
    }

    /// Produce an iterator yielding all vertices from all nodes within the graph in order of
    /// discovery within a depth-first-search.
    pub fn vertices<'a, 'b, F, I>(
        &'a self,
        dfs: &'b mut node::Dfs<S>,
        vertices_fn: F,
    ) -> Vertices<'a, 'b, F, I, S>
    where
        F: Fn(&node::Index) -> I,
        I: IntoIterator,
        I::Item: node::ApplyTransform<S>,
    {
        let walker = self.walk_vertices(dfs, vertices_fn);
        let graph = self;
        Vertices { graph, walker }
    }

    /// Produce an iterator yielding all triangles from all nodes within the graph in order of
    /// discovery within a depth-first-search.
    pub fn triangles<'a, 'b, F, I, V>(
        &'a self,
        dfs: &'b mut node::Dfs<S>,
        triangles_fn: F,
    ) -> Triangles<'a, 'b, F, I, V, S>
    where
        F: Fn(&node::Index) -> I,
        I: IntoIterator<Item = geom::Tri<V>>,
        V: geom::Vertex + node::ApplyTransform<S>,
    {
        let walker = self.walk_triangles(dfs, triangles_fn);
        let graph = self;
        Triangles { graph, walker }
    }

    /// The `Cuboid` that bounds all nodes within the geometry graph.
    ///
    /// Returns `None` if the graph contains no vertices.
    ///
    /// 1. Iterates over all nodes.
    /// 2. Expands a cuboid to the max bounds of each node.
    /// 3. Returns the resulting cuboid.
    pub fn bounding_cuboid<F, I>(
        &self,
        dfs: &mut node::Dfs<S>,
        vertices_fn: F,
    ) -> Option<geom::Cuboid<S>>
    where
        F: Fn(&node::Index) -> I,
        I: IntoIterator<Item = Point3<S>>,
    {
        let vertices = self.vertices(dfs, vertices_fn);
        geom::bounding_cuboid(vertices)
    }

    //---------------
    // PARENT METHODS
    //---------------

    /// A **Walker** type that may be used to step through the parents of the given child node.
    pub fn parents(&self, child: node::Index) -> Parents<S> {
        let parents = self.dag.parents(child);
        Parents { parents }
    }

    /// If the **Node** at the given index has some parent along an **Edge** of the given variant,
    /// return an index to it.
    pub fn edge_parent(&self, idx: node::Index, edge: edge::Kind) -> Option<node::Index> {
        self.parents(idx)
            .iter(self)
            .find(|&(e, _)| self[e].kind == edge)
            .map(|(_, n)| n)
    }

    /// Return the index of the parent along the given node's **Position** **Edge**.
    pub fn position_parent(&self, idx: node::Index, axis: edge::Axis) -> Option<node::Index> {
        let kind = edge::Kind::new(axis, edge::Relative::Position);
        self.edge_parent(idx, kind)
    }

    /// Return the index of the parent along the given node's **Position** **Edge**.
    pub fn x_position_parent(&self, idx: node::Index) -> Option<node::Index> {
        self.position_parent(idx, edge::Axis::X)
    }

    /// Return the index of the parent along the given node's **Position** **Edge**.
    pub fn y_position_parent(&self, idx: node::Index) -> Option<node::Index> {
        self.position_parent(idx, edge::Axis::Y)
    }

    /// Return the index of the parent along the given node's **Position** **Edge**.
    pub fn z_position_parent(&self, idx: node::Index) -> Option<node::Index> {
        self.position_parent(idx, edge::Axis::Z)
    }

    /// Produce an **Iterator** yielding the **Position** parents to the given node.
    ///
    /// Parents are always yielded in order of axis, e.g. **X**, **Y** then **Z**.
    pub fn position_parents(&self, idx: node::Index) -> PositionParents {
        self.x_position_parent(idx)
            .into_iter()
            .chain(self.y_position_parent(idx))
            .chain(self.z_position_parent(idx))
    }

    /// Return the index of the parent along the given node's **Orientation** **Edge**.
    pub fn orientation_parent(&self, idx: node::Index, axis: edge::Axis) -> Option<node::Index> {
        let kind = edge::Kind::new(axis, edge::Relative::Orientation);
        self.edge_parent(idx, kind)
    }

    /// Return the index of the parent along the given node's **Orientation** **Edge**.
    pub fn x_orientation_parent(&self, idx: node::Index) -> Option<node::Index> {
        self.orientation_parent(idx, edge::Axis::X)
    }

    /// Return the index of the parent along the given node's **Orientation** **Edge**.
    pub fn y_orientation_parent(&self, idx: node::Index) -> Option<node::Index> {
        self.orientation_parent(idx, edge::Axis::Y)
    }

    /// Return the index of the parent along the given node's **Orientation** **Edge**.
    pub fn z_orientation_parent(&self, idx: node::Index) -> Option<node::Index> {
        self.orientation_parent(idx, edge::Axis::Z)
    }

    /// Produce an **Iterator** yielding the **Orientation** parents to the given node.
    ///
    /// Parents are always yielded in order of axis, e.g. **X**, **Y** then **Z**.
    pub fn orientation_parents(&self, idx: node::Index) -> OrientationParents {
        self.x_orientation_parent(idx)
            .into_iter()
            .chain(self.y_orientation_parent(idx))
            .chain(self.z_orientation_parent(idx))
    }

    /// Return the index of the parent along the given node's **Scale** **Edge**.
    pub fn scale_parent(&self, idx: node::Index, axis: edge::Axis) -> Option<node::Index> {
        let kind = edge::Kind::new(axis, edge::Relative::Scale);
        self.edge_parent(idx, kind)
    }

    /// Return the index of the parent along the given node's **Scale** **Edge**.
    pub fn x_scale_parent(&self, idx: node::Index) -> Option<node::Index> {
        self.scale_parent(idx, edge::Axis::X)
    }

    /// Return the index of the parent along the given node's **Scale** **Edge**.
    pub fn y_scale_parent(&self, idx: node::Index) -> Option<node::Index> {
        self.scale_parent(idx, edge::Axis::Y)
    }

    /// Return the index of the parent along the given node's **Scale** **Edge**.
    pub fn z_scale_parent(&self, idx: node::Index) -> Option<node::Index> {
        self.scale_parent(idx, edge::Axis::Z)
    }

    /// Produce an **Iterator** yielding the **Scale** parents to the given node.
    ///
    /// Parents are always yielded in order of axis, e.g. **X**, **Y** then **Z**.
    pub fn scale_parents(&self, idx: node::Index) -> ScaleParents {
        self.x_scale_parent(idx)
            .into_iter()
            .chain(self.y_scale_parent(idx))
            .chain(self.z_scale_parent(idx))
    }

    /// Produce an **Iterator** yielding the **X** parents to the given node.
    ///
    /// Parents are always yielded in order of **Position**, **Orientation** and **Scale**.
    pub fn x_parents(&self, idx: node::Index) -> XParents {
        self.x_position_parent(idx)
            .into_iter()
            .chain(self.x_orientation_parent(idx))
            .chain(self.x_scale_parent(idx))
    }

    /// Produce an **Iterator** yielding the **Y** parents to the given node.
    ///
    /// Parents are always yielded in order of **Position**, **Orientation** and **Scale**.
    pub fn y_parents(&self, idx: node::Index) -> YParents {
        self.y_position_parent(idx)
            .into_iter()
            .chain(self.y_orientation_parent(idx))
            .chain(self.y_scale_parent(idx))
    }

    /// Produce an **Iterator** yielding the **Z** parents to the given node.
    ///
    /// Parents are always yielded in order of **Position**, **Orientation** and **Scale**.
    pub fn z_parents(&self, idx: node::Index) -> ZParents {
        self.z_position_parent(idx)
            .into_iter()
            .chain(self.z_orientation_parent(idx))
            .chain(self.z_scale_parent(idx))
    }

    //-----------------
    // CHILDREN METHODS
    //-----------------

    /// A **Walker** type that may be used to step through the children of the given parent node.
    pub fn children(&self, parent: node::Index) -> Children<S> {
        let children = self.dag.children(parent);
        Children { children }
    }

    // TODO: Add `x_position_children`, `position_children`, etc methods.
}

impl<S> Default for Graph<S>
where
    S: BaseFloat,
{
    fn default() -> Self {
        let mut dag = Dag::new();
        let origin = dag.add_node(Node::Point);
        Graph { dag, origin }
    }
}

impl<S> GraphBase for Graph<S>
where
    S: BaseFloat,
{
    type NodeId = node::Index;
    type EdgeId = edge::Index;
}

impl<S> Visitable for Graph<S>
where
    S: BaseFloat,
{
    type Map = <Dag<S> as Visitable>::Map;
    fn visit_map(&self) -> Self::Map {
        self.dag.visit_map()
    }
    fn reset_map(&self, map: &mut Self::Map) {
        self.dag.reset_map(map)
    }
}

impl<'a, S> IntoNeighbors for &'a Graph<S>
where
    S: BaseFloat,
{
    type Neighbors = <&'a Dag<S> as IntoNeighbors>::Neighbors;
    fn neighbors(self, n: Self::NodeId) -> Self::Neighbors {
        self.dag.neighbors(n)
    }
}

impl<S> ops::Index<node::Index> for Graph<S>
where
    S: BaseFloat,
{
    type Output = Node<S>;
    fn index<'a>(&'a self, id: node::Index) -> &'a Self::Output {
        self.node(id).unwrap()
    }
}

impl<S> ops::Index<edge::Index> for Graph<S>
where
    S: BaseFloat,
{
    type Output = Edge<S>;
    fn index<'a>(&'a self, idx: edge::Index) -> &'a Self::Output {
        self.edge(idx).unwrap()
    }
}

impl<'a, S> Walker<&'a Graph<S>> for Children<S>
where
    S: BaseFloat,
{
    type Item = (edge::Index, node::Index);
    #[inline]
    fn walk_next(&mut self, graph: &'a Graph<S>) -> Option<Self::Item> {
        self.children.walk_next(&graph.dag)
    }
}

impl<'a, S> Walker<&'a Graph<S>> for Parents<S>
where
    S: BaseFloat,
{
    type Item = (edge::Index, node::Index);
    #[inline]
    fn walk_next(&mut self, graph: &'a Graph<S>) -> Option<Self::Item> {
        self.parents.walk_next(&graph.dag)
    }
}

//! A directed acyclic graph to manage both storing widgets and describing their relationships.
//!
//! The primary type of interest in this module is the [**Graph**](./struct.Graph) type.

use crate::widget::{self, Widget};
use daggy::petgraph::visit::GraphBase;
use std::fmt;
use std::ops::{Index, IndexMut};

pub use daggy;
pub use daggy::Walker;

// pub mod algo;

/// The unsigned integer type used to index into the widget graph.
pub type IndexTy = u32;

/// An alias for our Graph's Node Index.
pub type NodeIndex = daggy::NodeIndex<IndexTy>;

/// An alias for our Graph's Edge Index.
pub type EdgeIndex = daggy::EdgeIndex<IndexTy>;

/// An alias for a tuple containing an associated `Edge/widget::Id` pair.
pub type IndexPair = (EdgeIndex, widget::Id);

/// A **Walker** over some node's parent nodes.
pub type Parents = daggy::stable_dag::Parents<Node, Edge, IndexTy>;
/// A **Walker** over some node's child nodes.
pub type Children = daggy::stable_dag::Children<Node, Edge, IndexTy>;

/// An alias for some filtered children walker.
pub type FilteredChildren =
    daggy::walker::Filter<Graph, Children, fn(&Graph, EdgeIndex, widget::Id) -> bool>;

/// An alias for our Graph's recursive walker.
pub type RecursiveWalk<F> = daggy::walker::Recursive<Graph, F>;

/// An alias for our Graph's **WouldCycle** error type.
pub type WouldCycle = daggy::WouldCycle<Edge>;

/// A node within the UI graph.
pub struct Node {
    /// Dynamically stored widget state.
    pub widget: Option<Box<dyn Widget>>,
}

/// An edge between nodes within the UI Graph.
///
/// An edge from *a -> b* indicates that *a* is the parent of *b*. In other words, *a* owns *b*.
/// Parents are rendered before their children.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Edge;

/// An alias for the petgraph::Graph used within our Ui Graph.
type Dag = daggy::stable_dag::StableDag<Node, Edge>;

/// The store of all widgets within a user interface.
///
/// Nodes represent the individual widgets.
///
/// Edges represent the relationship between widgets. That is, the edge *a -> b* indicates that *a*
/// is a parent of *b* and in turn, *a* owns *b*.
#[derive(Debug, Default)]
pub struct Graph {
    dag: Dag,
}

impl Edge {
    /// The number of different variants of the **Edge** type.
    pub const VARIANTS: usize = 1;
}

impl Node {
    /// A reference to the inner **Widget**.
    pub fn widget(&self) -> Option<&dyn Widget> {
        self.widget.as_ref().map(|w| &**w)
    }

    /// A mutable reference to the inner **Widget**.
    pub fn widget_mut(&mut self) -> Option<&mut dyn Widget> {
        self.widget.as_mut().map(|w| &mut **w)
    }
}

impl Graph {
    /// A new empty **Graph**.
    pub fn new() -> Self {
        Graph { dag: Dag::new() }
    }

    /// A new **Graph** with the given node capacity.
    ///
    /// We know that there can be no more than three parents per node as the public API enforces a
    /// maximum of one Depth, Position and Graphic parent each. Thus, we can assume an edge
    /// capacity of exactly three times the given node capacity.
    pub fn with_node_capacity(n_nodes: usize) -> Self {
        let n_edges = n_nodes * Edge::VARIANTS;
        Graph {
            dag: Dag::with_capacity(n_nodes, n_edges),
        }
    }

    /// Removes all **Node**s and **Edge**s from the **Graph**.
    pub fn clear(&mut self) {
        self.dag.clear()
    }

    /// The total number of **Node**s in the **Graph**.
    pub fn node_count(&self) -> usize {
        self.dag.node_count()
    }

    /// The total number of **Edge**s in the **Graph**.
    pub fn edge_count(&self) -> usize {
        self.dag.edge_count()
    }

    /// The current capacity for the **Graph**'s internal node `Vec`.
    pub fn node_capacity(&self) -> usize {
        unimplemented!();
    }

    /// Add the given **Node** to the graph.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Returns the index of the new node.
    pub(crate) fn add_node(&mut self, node: Node) -> widget::Id {
        self.dag.add_node(node)
    }

    /// Remove the **Node** at the given `Id` along with all nodes that are children of this node.
    ///
    /// Produces an iterator yielding removed nodes. The order of node removal is a DFS from the
    /// node at the given ID.
    fn remove_branch<'a>(&'a mut self, _id: widget::Id) -> impl 'a + Iterator<Item = Node> {
        // TODO
        unimplemented!();
        std::iter::empty()
    }

    // /// Set the given **Edge** within the graph.
    // ///
    // /// The added edge will be in the direction `a` -> `b`
    // ///
    // /// There may only ever be one **Edge** of the given variant between `a` -> `b`. In turn, the
    // /// **Graph** could be described as "three rose trees super imposed on top of one another,
    // /// where there is one tree for each edge variant".
    // ///
    // /// Checks if the edge would create a cycle in the **Graph**.
    // ///
    // /// If adding the edge **would not** cause the graph to cycle, the edge will be added and its
    // /// `EdgeIndex` returned.
    // ///
    // /// If adding the edge **would** cause the graph to cycle, the edge will not be added and
    // /// instead a `WouldCycle` error with the given weight will be returned.
    // ///
    // /// **Panics** if either `a` or `b` do not exist within the **Graph**.
    // ///
    // /// **Panics** if the **Graph** is at the maximum number of nodes for its index type.
    // fn set_edge(
    //     &mut self,
    //     a: widget::Id,
    //     b: widget::Id,
    //     edge: Edge,
    // ) -> Result<EdgeIndex, WouldCycle> {
    //     // Check to see if the node already has some matching incoming edge.
    //     // Keep it if it's the one we want. Otherwise, remove any incoming edge that matches the given
    //     // edge kind but isn't coming from the node that we desire.
    //     let mut parents = self.parents(b);
    //     let mut already_set = None;

    //     while let Some((in_edge_idx, in_node_idx)) = parents.next(self) {
    //         if edge == self[in_edge_idx] {
    //             if in_node_idx == a {
    //                 already_set = Some(in_edge_idx);
    //             } else {
    //                 self.remove_edge(in_edge_idx);
    //             }
    //             // Note that we only need to check for *one* edge as there can only ever be one
    //             // parent edge of any kind for each node. We know this, as this method is the only
    //             // function used by a public method that adds edges.
    //             break;
    //         }
    //     }

    //     // If we don't already have an incoming edge from the requested parent, add one.
    //     match already_set {
    //         Some(edge_idx) => Ok(edge_idx),
    //         None => self.dag.add_edge(a, b, edge),
    //     }
    // }

    /// Remove and return the **Edge** at the given index.
    ///
    /// Return `None` if it didn't exist.
    fn remove_edge(&mut self, idx: EdgeIndex) -> Option<Edge> {
        self.dag.remove_edge(idx)
    }

    pub fn add_child(&mut self, parent: widget::Id, node: Node) -> (EdgeIndex, widget::Id) {
        self.dag.add_child(parent, Edge, node)
    }

    /// Borrow the node at the given **widget::Id** if there is one.
    pub fn node(&self, idx: widget::Id) -> Option<&Node> {
        self.dag.node_weight(idx)
    }

    /// Mutably borrow the node at the given **widget::Id** if there is one.
    pub fn node_mut(&mut self, idx: widget::Id) -> Option<&mut Node> {
        self.dag.node_weight_mut(idx)
    }

    /// Borrow the edge at the given **EdgeIndex** if there is one.
    pub fn edge(&self, idx: EdgeIndex) -> Option<&Edge> {
        self.dag.edge_weight(idx)
    }

    /// Mutably borrow the edge at the given **EdgeIndex** if there is one.
    pub fn edge_mut(&mut self, idx: EdgeIndex) -> Option<&mut Edge> {
        self.dag.edge_weight_mut(idx)
    }

    /// Return the parent and child nodes on either end of the **Edge** at the given index.
    pub fn edge_endpoints(&self, idx: EdgeIndex) -> Option<(widget::Id, widget::Id)> {
        self.dag.edge_endpoints(idx)
    }

    /// If there is a Widget for the given index, return a reference to it.
    pub fn widget(&self, idx: widget::Id) -> Option<&dyn Widget> {
        self.node(idx).and_then(|node| node.widget())
    }

    /// If there is a Widget for the given Id, return a mutable reference to it.
    pub fn widget_mut(&mut self, idx: widget::Id) -> Option<&mut dyn Widget> {
        self.node_mut(idx).and_then(|node| node.widget_mut())
    }

    /// A **Walker** type that may be used to step through the parents of the given child node.
    pub fn parents(&self, child: widget::Id) -> Parents {
        self.dag.parents(child)
    }

    /// A **Walker** type that recursively walks the **Graph** using the given `recursive_fn`.
    ///
    /// **Panics** If the given start index does not exist within the **Graph**.
    pub fn recursive_walk<F>(&self, start: widget::Id, recursive_fn: F) -> RecursiveWalk<F>
    where
        F: FnMut(&Self, widget::Id) -> Option<(EdgeIndex, widget::Id)>,
    {
        RecursiveWalk::new(start, recursive_fn)
    }

    /// A **Walker** type that may be used to step through the children of the given parent node.
    pub fn children(&self, parent: widget::Id) -> Children {
        self.dag.children(parent)
    }

    /// Does the given edge type exist between the nodes `parent` -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_edge_exist<F>(&self, parent: widget::Id, child: widget::Id, is_edge: F) -> bool
    where
        F: Fn(Edge) -> bool,
    {
        self.parents(child)
            .iter(self)
            .any(|(e, n)| n == parent && is_edge(self[e]))
    }

    /// Are the given `parent` and `child` nodes connected by a single chain of edges of the given
    /// kind?
    ///
    /// i.e. `parent` -> x -> y -> `child`.
    ///
    /// Returns `false` if either of the given node indices do not exist.
    pub fn does_recursive_edge_exist<F>(
        &self,
        parent: widget::Id,
        child: widget::Id,
        is_edge: F,
    ) -> bool
    where
        F: Fn(Edge) -> bool,
    {
        self.recursive_walk(child, |g, n| g.parents(n).iter(g).find(|&(e, _)| is_edge(g[e])))
            .iter(self)
            .any(|(_, n)| n == parent)
    }

    // /// Cache some `PreUpdateCache` widget data into the graph.
    // ///
    // /// This is called (via the `ui` module) from within the `widget::set_widget` function prior to
    // /// the `Widget::update` method being called.
    // ///
    // /// This is done so that if this Widget were to internally `set` some other `Widget`s within
    // /// its own `update` method, this `Widget`s positioning and dimension data already exists
    // /// within the `Graph` for reference.
    // pub fn pre_update_cache(
    //     &mut self,
    //     root: widget::Id,
    //     widget: widget::PreUpdateCache,
    //     instantiation_order_idx: usize,
    // ) {
    //     let widget::PreUpdateCache {
    //         type_id,
    //         id,
    //         maybe_parent_id,
    //         maybe_x_positioned_relatively_id,
    //         maybe_y_positioned_relatively_id,
    //         rect,
    //         depth,
    //         kid_area,
    //         maybe_dragged_from,
    //         maybe_floating,
    //         crop_kids,
    //         maybe_x_scroll_state,
    //         maybe_y_scroll_state,
    //         maybe_graphics_for,
    //         is_over,
    //     } = widget;

    //     assert!(
    //         self.node(id).is_some(),
    //         "No node found for the given widget::Id {:?}",
    //         id
    //     );

    //     // Construct a new `Container` to place in the `Graph`.
    //     let new_container = || Container {
    //         maybe_state: None,
    //         type_id: type_id,
    //         rect: rect,
    //         depth: depth,
    //         kid_area: kid_area,
    //         maybe_dragged_from: maybe_dragged_from,
    //         maybe_floating: maybe_floating,
    //         crop_kids: crop_kids,
    //         maybe_x_scroll_state: maybe_x_scroll_state,
    //         maybe_y_scroll_state: maybe_y_scroll_state,
    //         instantiation_order_idx: instantiation_order_idx,
    //         is_over: IsOverFn(is_over),
    //     };

    //     // Retrieves the widget's parent index.
    //     //
    //     // `panic!` if the widget does not exist within the graph. This should rarely be the case
    //     // as all existing `widget::Id`s should be generated from the graph itself.
    //     //
    //     // This should only be `None` if the widget is the `root` node (i.e. the `Window` widget).
    //     let maybe_parent_id = |graph: &mut Self| match maybe_parent_id {
    //         Some(parent_id) => match graph.node(parent_id).is_some() {
    //             true => Some(parent_id),
    //             false => panic!(
    //                 "No node found for the given parent widget::Id {:?}",
    //                 parent_id
    //             ),
    //         },
    //         // Check that this node is not the root node before using the root node as the parent.
    //         None => {
    //             if id == root {
    //                 None
    //             } else {
    //                 Some(root)
    //             }
    //         }
    //     };

    //     // Ensure that we have an `Edge::Depth` in the graph representing the parent.
    //     if let Some(parent_id) = maybe_parent_id(self) {
    //         self.set_edge(parent_id, id, Edge::Depth).unwrap();
    //     }

    //     match &mut self.dag[id] {
    //         // If the node is currently a `Placeholder`, construct a new container and use this
    //         // to set it as the `Widget` variant.
    //         node @ &mut Node::Placeholder => *node = Node::Widget(new_container()),

    //         // Otherwise, update the data in the container that already exists.
    //         &mut Node::Widget(ref mut container) => {
    //             // If the container already exists with the state of some other kind of
    //             // widget, we can assume there's been a mistake with the given Id.
    //             //
    //             // TODO: It might be overkill to panic here.
    //             assert!(
    //                 container.type_id == type_id,
    //                 "A widget of a different type already exists at the given id \
    //                 ({:?}). You tried to insert a widget with state of type {:?}, \
    //                 however the existing widget state is of type {:?}. Check your \
    //                 `WidgetId`s for errors.",
    //                 id,
    //                 &type_id,
    //                 container.type_id
    //             );

    //             container.type_id = type_id;
    //             container.rect = rect;
    //             container.depth = depth;
    //             container.kid_area = kid_area;
    //             container.maybe_dragged_from = maybe_dragged_from;
    //             container.maybe_floating = maybe_floating;
    //             container.crop_kids = crop_kids;
    //             container.maybe_x_scroll_state = maybe_x_scroll_state;
    //             container.maybe_y_scroll_state = maybe_y_scroll_state;
    //             container.instantiation_order_idx = instantiation_order_idx;
    //             container.is_over = IsOverFn(is_over);
    //         }
    //     }

    //     // Now that we've updated the widget's cached data, we need to check if we should add any
    //     // `Edge::Position`s.
    //     //
    //     // If the widget is *not* positioned relatively to any other widget, we should ensure that
    //     // there are no incoming `Position` edges.

    //     // X
    //     if let Some(relative_id) = maybe_x_positioned_relatively_id {
    //         self.set_edge(relative_id, id, Edge::Position(Axis::X))
    //             .unwrap();
    //     } else {
    //         self.remove_parent_edge(id, Edge::Position(Axis::X));
    //     }

    //     // Y
    //     if let Some(relative_id) = maybe_y_positioned_relatively_id {
    //         self.set_edge(relative_id, id, Edge::Position(Axis::Y))
    //             .unwrap();
    //     } else {
    //         self.remove_parent_edge(id, Edge::Position(Axis::Y));
    //     }

    //     // Check whether or not the widget is a graphics element for some other widget.
    //     if let Some(graphic_parent_id) = maybe_graphics_for {
    //         self.set_edge(graphic_parent_id, id, Edge::Graphic).unwrap();
    //     // If not, ensure that there is no parent **Graphic** edge from the widget.
    //     } else {
    //         self.remove_parent_edge(id, Edge::Graphic);
    //     }
    // }

    // /// Cache some `PostUpdateCache` widget data into the graph.
    // ///
    // /// This is called (via the `ui` module) from within the `widget::set_widget` function after
    // /// the `Widget::update` method is called and some new state is returned.
    // pub fn post_update_cache<W>(&mut self, widget: widget::PostUpdateCache<W>)
    // where
    //     W: Widget,
    //     W::State: 'static,
    //     W::Style: 'static,
    // {
    //     let widget::PostUpdateCache {
    //         id, state, style, ..
    //     } = widget;

    //     // We know that their must be a widget::Id for this id, as `Graph::pre_update_cache` will
    //     // always be called prior to this method being called.
    //     if let Some(ref mut container) = self.widget_mut(id) {
    //         // Construct the `UniqueWidgetState` ready to store as an `Any` within the container.
    //         let unique_state: UniqueWidgetState<W::State, W::Style> = UniqueWidgetState {
    //             state: state,
    //             style: style,
    //         };

    //         container.maybe_state = Some(Box::new(unique_state));
    //     }
    // }
}

// Node impls

impl<T> From<T> for Node
where
    T: Widget,
{
    fn from(w: T) -> Self {
        let boxed = Box::new(w) as Box<dyn Widget>;
        let widget = Some(boxed);
        Self { widget }
    }
}

impl Into<Option<Box<dyn Widget>>> for Node {
    fn into(self) -> Option<Box<dyn Widget>> {
        self.widget
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: Use method to retrieve widget name?
        write!(f, "{:?}", self.widget().map(|_| ()))
    }
}

// Walker impls

impl<'a> Walker<&'a Graph> for Children {
    type Item = (EdgeIndex, widget::Id);
    fn walk_next(&mut self, graph: &'a Graph) -> Option<Self::Item> {
        self.walk_next(&graph.dag)
    }
}

impl<'a> Walker<&'a Graph> for Parents {
    type Item = (EdgeIndex, widget::Id);
    fn walk_next(&mut self, graph: &'a Graph) -> Option<Self::Item> {
        self.walk_next(&graph.dag)
    }
}

// Graph impls

impl GraphBase for Graph {
    type EdgeId = EdgeIndex;
    type NodeId = NodeIndex;
}

impl Index<widget::Id> for Graph {
    type Output = Node;
    fn index<'a>(&'a self, id: widget::Id) -> &'a Node {
        self.node(id).unwrap()
    }
}

impl IndexMut<widget::Id> for Graph {
    fn index_mut<'a>(&'a mut self, id: widget::Id) -> &'a mut Node {
        self.node_mut(id).unwrap()
    }
}

impl Index<EdgeIndex> for Graph {
    type Output = Edge;
    fn index<'a>(&'a self, idx: EdgeIndex) -> &'a Edge {
        self.edge(idx).unwrap()
    }
}

impl IndexMut<EdgeIndex> for Graph {
    fn index_mut<'a>(&'a mut self, idx: EdgeIndex) -> &'a mut Edge {
        self.edge_mut(idx).unwrap()
    }
}

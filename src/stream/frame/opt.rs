//! A series of optimisation passes for laser frames.

use crate::Point;
use petgraph::{Directed, Undirected};
use petgraph::visit::EdgeRef;

/// Represents a line segment over which the laser scanner will travel.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Segment {
    pub start: Point,
    pub end: Point,
    pub kind: SegmentKind,
}

/// describes whether a line segment between two points is blank or not.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum SegmentKind {
    Blank,
    NonBlank,
}

/// A type used to represent graph describing the points in a frame and how they are joined.
///
/// Only non-blank edges are represented in this representation.
pub type PointGraph = petgraph::Graph<Point, (), Undirected, u32>;

/// A type used to represent a graph of points that contains at least one euler circuit.
pub type EulerGraph = petgraph::Graph<Point, SegmentKind, Undirected, u32>;

/// A type used to represent a directed graph describing a euler circuit.
pub type EulerCircuit = petgraph::Graph<Point, SegmentKind, Directed, u32>;

type EdgeIndex = petgraph::graph::EdgeIndex<u32>;
type NodeIndex = petgraph::graph::NodeIndex<u32>;

/// An iterator yielding all non-blank line segments.
#[derive(Clone)]
pub struct Segments<I> {
    points: I,
    last_point: Option<Point>,
}

impl<I> Iterator for Segments<I>
where
    I: Iterator<Item = Point>,
{
    type Item = Segment;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(end) = self.points.next() {
            let start = match self.last_point.replace(end) {
                None => continue,
                Some(last) => last,
            };

            // Skip duplicates.
            if start == end {
                continue;
            }

            let kind = if start.is_blank() && end.is_blank() {
                SegmentKind::Blank
            } else {
                SegmentKind::NonBlank
            };

            return Some(Segment { start, end, kind })
        }
        None
    }
}

/// Create an iterator yielding segments from an iterator yielding points.
pub fn points_to_segments<I>(points: I) -> Segments<I::IntoIter>
where
    I: IntoIterator<Item = Point>,
{
    let points = points.into_iter();
    let last_point = None;
    Segments { points, last_point }
}

/// Convert the given laser frame vector segments to a graph of points.
pub fn segments_to_point_graph<I>(segments: I) -> PointGraph
where
    I: IntoIterator<Item = Segment>,
{
    // A hashable version of a `Point`, used for removing point duplicates during graph generation.
    #[derive(Eq, Hash, PartialEq)]
    struct HashPoint {
        pos: [i32; 2],
        rgb: [u32; 3],
    }

    impl From<Point> for HashPoint {
        fn from(p: Point) -> Self {
            let [px, py] = p.position;
            let [pr, pg, pb] = p.color;
            let x = (px * std::i16::MAX as f32) as i32;
            let y = (py * std::i16::MAX as f32) as i32;
            let r = (pr * std::u16::MAX as f32) as u32;
            let g = (pg * std::u16::MAX as f32) as u32;
            let b = (pb * std::u16::MAX as f32) as u32;
            let pos = [x, y];
            let rgb = [r, g, b];
            HashPoint { pos, rgb }
        }
    }

    let mut g = PointGraph::default();
    let mut pt_to_id = hashbrown::HashMap::new();

    // Build the graph.
    for seg in segments {
        match seg.kind {
            SegmentKind::Blank => (),
            SegmentKind::NonBlank => {
                let ha = HashPoint::from(seg.start);
                let hb = HashPoint::from(seg.end);
                let na = *pt_to_id.entry(ha).or_insert_with(|| g.add_node(seg.start));
                let nb = *pt_to_id.entry(hb).or_insert_with(|| g.add_node(seg.end));
                g.add_edge(na, nb, ());
            }
        }
    }

    g
}

/// Convert a point graph to a euler graph.
///
/// This determines the minimum number of blank segments necessary to create a euler circuit
/// from the given point graph. A euler circuit is useful as it represents a graph that can be
/// drawn unicursally (one continuous path that covers all nodes while only traversing each edge
/// once).
pub fn point_graph_to_euler_graph(pg: &PointGraph) -> EulerGraph {
    // Find the connected components.
    let ccs = petgraph::algo::kosaraju_scc(pg);

    // The indices of the connected components whose nodes all have an even degree.
    let euler_components: hashbrown::HashSet<_> = ccs.iter()
        .enumerate()
        .filter(|(_, cc)| cc.iter().all(|&n| pg.edges(n).count() % 2 == 0))
        .map(|(i, _)| i)
        .collect();

    // Represents the nodes to be connected for a single component.
    struct ToConnect {
        // Connection to the previous component.
        prev: NodeIndex,
        // Consecutive connections within the component.
        inner: Vec<NodeIndex>,
        // Connection to the next component.
        next: NodeIndex,
    }

    // Collect the free nodes from each connected component that are to be connected by blanks.
    let mut to_connect = vec![];
    for (i, cc) in ccs.iter().enumerate() {
        if euler_components.contains(&i) {
            // Take the first point.
            let n = cc[0];
            to_connect.push(ToConnect { prev: n, inner: vec![], next: n });
        } else {
            let v: Vec<_> = cc.iter().filter(|&&n| pg.edges(n).count() % 2 != 0).collect();
            assert_eq!(
                v.len() % 2, 0,
                "expected even number of odd-degree nodes for non-Euler component",
            );
            let prev = *v[0];
            let inner = v[1..v.len()-1].iter().map(|&&n| n).collect();
            let next = *v[v.len()-1];
            to_connect.push(ToConnect { prev, inner, next });
        }
    }

    // Convert the `to_connect` Vec containing the nodes to be connected for each connected
    // component to a `Vec` containing the pairs of nodes which will be directly connected.
    let mut pairs = vec![];
    let mut iter = to_connect.iter().peekable();
    while let Some(this) = iter.next() {
        for ch in this.inner.chunks(2) {
            pairs.push((ch[0], ch[1]));
        }
        match iter.peek() {
            Some(next) => pairs.push((this.next, next.prev)),
            None => pairs.push((this.next, to_connect[0].prev)),
        }
    }

    // Turn the graph into a euler graph by adding the blanks.
    let mut eg = pg.map(|_n_ix, n| n.clone(), |_e_ix, _| SegmentKind::NonBlank);
    for (na, nb) in pairs {
        eg.add_edge(na, nb, SegmentKind::Blank);
    }

    eg
}

/// Given a Euler Graph describing the vector image to be drawn, return the optimal Euler Circuit
/// describing the path over which the laser should travel.
pub fn euler_graph_to_euler_circuit(eg: &EulerGraph) -> EulerCircuit {
    // The starting node. `v0`.
    let start = match eg.node_indices().next() {
        Some(n_ix) => n_ix,
        None => return Default::default(),
    };

    let mut visit_order: Vec<EdgeIndex> = vec![];

    // The visit order of the traversal. `T0`.
    let mut n = start;
    loop {
        let e_ref = eg.edges(n).next().expect("expected a strongly connected euler graph");
        visit_order.push(e_ref.id());
        if e_ref.target() == start {
            break;
        }
    }

    // Create the initial visit order by traversing from `v0` until `v0` is reached again.

    // If the set of edges in

    unimplemented!()
}

//! A series of optimisation passes for laser frames.

use crate::lerp::Lerp;
use crate::point::{Point, Position, RawPoint};
use hashbrown::{HashMap, HashSet};
use petgraph::visit::{EdgeRef, Walker};
use petgraph::{Directed, Undirected};

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
    Lit,
}

/// A type used to represent graph describing the points in a frame and how they are joined.
///
/// Only lit edges are represented in this representation.
pub type PointGraph = petgraph::Graph<Point, (), Undirected, u32>;

/// A type used to represent a graph of points that contains at least one euler circuit.
pub type EulerGraph = petgraph::Graph<Point, SegmentKind, Undirected, u32>;

/// A type used to represent a directed graph describing a euler circuit.
pub type EulerCircuit = petgraph::Graph<Point, SegmentKind, Directed, u32>;

type EdgeIndex = petgraph::graph::EdgeIndex<u32>;
type NodeIndex = petgraph::graph::NodeIndex<u32>;
type EulerCircuitEdgeRef<'a> = petgraph::graph::EdgeReference<'a, SegmentKind, u32>;

/// An iterator yielding all lit line segments.
#[derive(Clone)]
pub struct Segments<I> {
    points: I,
    last_point: Option<Point>,
}

/// A walker for traversing a Eulerian circuit.
///
/// Note that the resulting walk is an infinite walk, and care should be taken to break when
/// returning to the starting node if necessary.
#[derive(Clone)]
pub struct EulerCircuitWalk {
    current_node: NodeIndex,
}

/// Configuration options for eulerian circuit interpolation.
#[derive(Clone, Debug, PartialEq)]
pub struct InterpolationConfig {
    /// The minimum distance the interpolator can travel along an edge before a new point is
    /// required.
    pub distance_per_point: f32,
    /// The number of points to insert at the end of a blank to account for light modulator delay.
    pub blank_delay_points: u32,
    /// The amount of delay to add based on the angle of the corner in radians.
    pub radians_per_point: f32,
}

/// Parameters for the frame interpolator.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct InterpolationConfigBuilder {
    pub distance_per_point: Option<f32>,
    pub blank_delay_points: Option<u32>,
    pub radians_per_point: Option<f32>,
}

/// For the blank ab: `[a, a.blanked(), b.blanked(), (0..delay).map(|_| b.blanked())]`.
pub const BLANK_MIN_POINTS: u32 = 3;

impl EulerCircuitWalk {
    /// Construct a walk from the starting node.
    pub fn new(current_node: NodeIndex) -> Self {
        EulerCircuitWalk { current_node }
    }

    /// The next edge in the given `EulerCircuit`.
    ///
    /// **Panic!**s if the given `EulerCircuit` is not in fact a true `EulerCircuit`.
    pub fn next<'a>(&mut self, ec: &'a EulerCircuit) -> EulerCircuitEdgeRef<'a> {
        let e_ref = ec.edges_directed(self.current_node, petgraph::Outgoing)
            .next()
            .expect("expected a eulerian circuit but current nod has no edges");
        self.current_node = e_ref.target();
        e_ref
    }
}

impl InterpolationConfig {
    /// The default distance the interpolator can travel before a new point is required.
    pub const DEFAULT_DISTANCE_PER_POINT: f32 = 0.1;
    /// The default number of points inserted for the end of each blank segment.
    pub const DEFAULT_BLANK_DELAY_POINTS: u32 = 10;
    /// The default radians per point of delay to reduce corner inertia.
    pub const DEFAULT_RADIANS_PER_POINT: f32 = 0.6;

    /// Start building a new `InterpolationConfig`.
    pub fn start() -> InterpolationConfigBuilder {
        InterpolationConfigBuilder::default()
    }
}

impl InterpolationConfigBuilder {
    /// The minimum distance the interpolator can travel along an edge before a new point is
    /// required.
    ///
    /// By default, this value is `InterpolationConfig::DEFAULT_DISTANCE_PER_POINT`.
    pub fn distance_per_point(mut self, dpp: f32) -> Self {
        self.distance_per_point = Some(dpp);
        self
    }

    /// The number of points to insert at the end of a blank to account for light modulator delay.
    ///
    /// By default, this value is `InterpolationConfig::DEFAULT_BLANK_DELAY_POINTS`.
    pub fn blank_delay_points(mut self, points: u32) -> Self {
        self.blank_delay_points = Some(points);
        self
    }

    /// The amount of delay to add based on the angle of the corner in radians.
    ///
    /// By default, this value is `InterpolationConfig::DEFAULT_RADIANS_PER_POINT`.
    pub fn radians_per_point(mut self, radians: f32) -> Self {
        self.radians_per_point = Some(radians);
        self
    }

    /// Build the `InterpolationConfig`, falling back to defaults where necessary.
    pub fn build(self) -> InterpolationConfig {
        InterpolationConfig {
            distance_per_point: self.distance_per_point
                .unwrap_or(InterpolationConfig::DEFAULT_DISTANCE_PER_POINT),
            blank_delay_points: self.blank_delay_points
                .unwrap_or(InterpolationConfig::DEFAULT_BLANK_DELAY_POINTS),
            radians_per_point: self.radians_per_point
                .unwrap_or(InterpolationConfig::DEFAULT_RADIANS_PER_POINT),
        }
    }
}

impl<'a> Walker<&'a EulerCircuit> for EulerCircuitWalk {
    type Item = EulerCircuitEdgeRef<'a>;
    fn walk_next(&mut self, ec: &'a EulerCircuit) -> Option<Self::Item> {
        Some(self.next(ec))
    }
}

impl Default for InterpolationConfig {
    fn default() -> Self {
        Self::start().build()
    }
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
            if start.position == end.position {
                continue;
            }

            let kind = if start.is_blank() && end.is_blank() {
                SegmentKind::Blank
            } else {
                SegmentKind::Lit
            };

            return Some(Segment { start, end, kind });
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

    struct Node {
        ix: NodeIndex,
        weight: u32,
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
    let mut pt_to_id = HashMap::new();

    // Build the graph.
    for seg in segments {
        match seg.kind {
            SegmentKind::Blank => (),
            SegmentKind::Lit => {
                let ha = HashPoint::from(seg.start);
                let hb = HashPoint::from(seg.end);
                let na = {
                    let n = pt_to_id.entry(ha).or_insert_with(|| {
                        let ix = g.add_node(seg.start);
                        let weight = seg.start.weight;
                        Node { ix, weight }
                    });
                    n.weight = std::cmp::max(n.weight, seg.start.weight);
                    n.ix
                };
                let nb = {
                    let n = pt_to_id.entry(hb).or_insert_with(|| {
                        let ix = g.add_node(seg.end);
                        let weight = seg.end.weight;
                        Node { ix, weight }
                    });
                    n.weight = std::cmp::max(n.weight, seg.end.weight);
                    n.ix
                };
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
    let euler_components: hashbrown::HashSet<_> = ccs
        .iter()
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
            to_connect.push(ToConnect {
                prev: n,
                inner: vec![],
                next: n,
            });
        } else {
            let v: Vec<_> = cc
                .iter()
                .filter(|&&n| pg.edges(n).count() % 2 != 0)
                .collect();
            assert_eq!(
                v.len() % 2,
                0,
                "expected even number of odd-degree nodes for non-Euler component",
            );
            let prev = *v[0];
            let inner = v[1..v.len() - 1].iter().map(|&&n| n).collect();
            let next = *v[v.len() - 1];
            to_connect.push(ToConnect { prev, inner, next });
        }
    }

    // Convert the `to_connect` Vec containing the nodes to be connected for each connected
    // component to a `Vec` containing the pairs of nodes which will be directly connected.
    let mut pairs = vec![];
    let mut iter = to_connect.iter().enumerate().peekable();
    while let Some((i, this)) = iter.next() {
        for ch in this.inner.chunks(2) {
            pairs.push((ch[0], ch[1]));
        }
        match iter.peek() {
            Some((_, next)) => pairs.push((this.next, next.prev)),
            None if i > 0 => pairs.push((this.next, to_connect[0].prev)),
            None => match euler_components.contains(&0) {
                // If there is only one component and it is euler, we are done.
                true => (),
                // If there is only one non-euler, connect it to itself.
                false => pairs.push((this.next, this.prev)),
            },
        }
    }

    // Turn the graph into a euler graph by adding the blanks.
    let mut eg = pg.map(|_n_ix, n| n.clone(), |_e_ix, _| SegmentKind::Lit);
    for (na, nb) in pairs {
        eg.add_edge(na, nb, SegmentKind::Blank);
    }

    eg
}

/// Given a Euler Graph describing the vector image to be drawn, return the optimal Euler Circuit
/// describing the path over which the laser should travel.
///
/// This is Hierholzer's Algorithm with the amendment that during traversal of each vertex the edge
/// with the closest angle to a straight line is always chosen.
pub fn euler_graph_to_euler_circuit(eg: &EulerGraph) -> EulerCircuit {
    // If there is one or less nodes, there's no place for edges.
    if eg.node_count() == 0 || eg.node_count() == 1 {
        return eg
            .map(|_ni, nw| nw.clone(), |_ei, ew| ew.clone())
            .into_edge_type();
    }

    // Begin the traversals to build the circuit, starting at `v0`.
    let start_n = eg
        .node_indices()
        .next()
        .expect("expected at least two nodes, found none");
    let mut visited: HashSet<EdgeIndex> = HashSet::new();
    let mut visit_order: Vec<EdgeIndex> = vec![];
    loop {
        // Find a node in the visit order with untraversed edges, or pick one to begin if we're
        // just starting. We will do a traversal from this node. Keep track of where in the
        // existing `visit_order` we should merge this new traversal. If there are no nodes with
        // untraversed edges, we are done.
        let (merge_ix, n) = match visit_order.is_empty() {
            true => (0, start_n),
            false => {
                match visit_order
                    .iter()
                    .map(|&e| eg.raw_edges()[e.index()].source())
                    .enumerate()
                    .find(|&(_i, n)| eg.edges(n).any(|e| !visited.contains(&e.id())))
                {
                    Some(n) => n,
                    None => break,
                }
            }
        };

        let traversal = traverse_unvisited(n, eg, &mut visited);
        let new_visit_order = visit_order
            .iter()
            .take(merge_ix)
            .cloned()
            .chain(traversal)
            .chain(visit_order.iter().skip(merge_ix).cloned())
            .collect();
        visit_order = new_visit_order;
    }

    // Construct the Euler Circuit.
    let mut ec = EulerCircuit::with_capacity(eg.node_count(), eg.edge_count());

    // Add the nodes.
    for n in eg.raw_nodes() {
        ec.add_node(n.weight.clone());
    }

    // Now re-add all edges directed in order of visitation.
    for e in visit_order {
        let e_ref = &eg.raw_edges()[e.index()];
        ec.add_edge(e_ref.source(), e_ref.target(), e_ref.weight.clone());
    }

    ec
}

// A traversal through unvisited edges of the graph starting from `n`.
//
// Traversal ends when `n` is reached again.
//
// The returned `Vec` contains the index of each edge traversed.
fn traverse_unvisited(
    start: NodeIndex,
    eg: &EulerGraph,
    visited: &mut HashSet<EdgeIndex>,
) -> Vec<EdgeIndex> {
    let mut n = start;
    let mut traversal: Vec<EdgeIndex> = vec![];
    loop {
        // Find the straightest edge that hasn't yet been traversed.
        let e_ref = {
            let mut untraversed_edges = eg.edges(n).filter(|e_ref| !visited.contains(&e_ref.id()));

            let init_e_ref = untraversed_edges
                .next()
                .expect("expected a strongly connected euler graph");

            match traversal
                .last()
                .map(|e| eg.raw_edges()[e.index()].source())
                .map(|n| eg[n].position)
            {
                // If this is the first edge in the traversal, use the first ref.
                None => init_e_ref,

                // Retrieve the three positions used to determine the angle.
                Some(prev_source_p) => {
                    let source_p = eg[init_e_ref.source()].position;
                    let target_p = eg[init_e_ref.target()].position;
                    let init_dist = straight_angle_variance(prev_source_p, source_p, target_p);
                    let init = (init_e_ref, init_dist);
                    let (e_ref, _) = untraversed_edges.fold(init, |best, e_ref| {
                        let (_, best_dist) = best;
                        let target_p = eg[e_ref.target()].position;
                        let dist = straight_angle_variance(prev_source_p, source_p, target_p);
                        if dist < best_dist {
                            (e_ref, dist)
                        } else {
                            best
                        }
                    });
                    e_ref
                }
            }
        };

        // Add the edge into our visitation record.
        let e = e_ref.id();
        n = e_ref.target();
        visited.insert(e);
        traversal.push(e);

        // If this edge brings us back to the start, we have finished this traversal.
        if e_ref.target() == start {
            break;
        }
    }

    traversal
}

// Given an angle described by points a -> b -> c, return the variance from a straight angle in
// radians.
fn straight_angle_variance([ax, ay]: Position, [bx, by]: Position, [cx, cy]: Position) -> f32 {
    let [ux, uy] = [bx - ax, by - ay];
    let [vx, vy] = [cx - bx, cy - by];
    let ur = uy.atan2(ux);
    let vr = vy.atan2(vx);
    let diff_rad = vr - ur;

    // Convert the radians to the angular distance.
    fn angular_dist(rad: f32) -> f32 {
        let rad = rad.abs();
        if rad > std::f32::consts::PI {
            -rad + std::f32::consts::PI * 2.0
        } else {
            rad
        }
    }

    angular_dist(diff_rad)
}

fn distance_squared(a: Position, b: Position) -> f32 {
    let [ax, ay] = a;
    let [bx, by] = b;
    let [abx, aby] = [bx - ax, by - ay];
    abx * abx + aby * aby
}

/// The number of points used per blank segment given the `blank_delay_points` from a config.
pub fn blank_segment_point_count(a_weight: u32, blank_delay_points: u32) -> u32 {
    a_weight + BLANK_MIN_POINTS + blank_delay_points
}

/// Returns the points used to blank between two given lit points *a* and *b*.
pub fn blank_segment_points(
    a: Point,
    br: RawPoint,
    blank_delay_points: u32,
) -> impl Iterator<Item = RawPoint> {
    let ar = a.to_raw();
    Some(ar)
        .into_iter()
        .chain(a.to_raw_weighted())
        .chain(Some(ar.blanked()))
        .chain(Some(br.blanked()))
        .chain((0..blank_delay_points).map(move |_| br.blanked()))
}

/// The number of points added at a lit corner given its angle and angular delay rate.
pub fn corner_point_count(rad: f32, corner_delay_radians_per_point: f32) -> u32 {
    (rad / corner_delay_radians_per_point) as _
}

/// The minimum points for traversing a lit segment (not including end corner delays).
pub fn distance_min_point_count(dist: f32, min_distance_per_point: f32) -> u32 {
    // There must be at least one point at the beginning of the line.
    const MIN_COUNT: u32 = 1;
    MIN_COUNT + (dist * min_distance_per_point) as u32
}

/// The minimum number of points used for a lit segment of the given distance and end angle.
///
/// `a_weight` refers to the weight of the point at the beginning of the segment.
pub fn lit_segment_min_point_count(
    distance: f32,
    end_corner_radians: f32,
    distance_per_point: f32,
    radians_per_point: f32,
    a_weight: u32,
) -> u32 {
    a_weight
        + corner_point_count(end_corner_radians, radians_per_point)
        + distance_min_point_count(distance, distance_per_point)
}

/// Returns the points that make up a lit segment between *a* and *b* including delay for the end
/// corner.
///
/// `excess_points` are distributed across the distance point count. This is used to allow the
/// interpolation process to evenly distribute left-over points across a frame.
pub fn lit_segment_points(
    a: Point,
    br: RawPoint,
    corner_point_count: u32,
    distance_min_point_count: u32,
    excess_points: u32,
) -> impl Iterator<Item = RawPoint> {
    let dist_point_count = distance_min_point_count + excess_points;
    let weight_points = a.to_raw_weighted();
    let ar = a.to_raw();
    let dist_points = (0..dist_point_count).map(move |i| {
        let lerp_amt = i as f32 / dist_point_count as f32;
        ar.lerp(&br, lerp_amt)
    });
    let corner_points = (0..corner_point_count).map(move |_| br);
    weight_points.chain(dist_points).chain(corner_points)
}

/// Interpolate the given `EulerCircuit` with the given configuration in order to produce a path
/// ready to be submitted to the DAC.
///
/// The interpolation process will attempt to generate `target_points` number of points along the
/// circuit, but may generate *more* points in the user's `InterpolationConfig` indicates that more
/// are required for interpolating the specified circuit.
///
/// Performs the following steps:
///
/// 1. Determine the minimum number of required points:
///     - 1 for each edge plus the 1 for the end.
///     - The number of points required for each edge.
///         - For lit edges:
///             - The distance of each edge accounting for minimum points per distance.
///             - The angular distance to the following lit edge (none if blank).
///         - For blank edges:
///             - The specified blank delay.
/// 2. If the total is greater than `target_points`, we're done. If not, goto 3.
/// 3. Determine a weight per lit edge based on the distance of each edge.
/// 4. Distribute the remaining points between each lit edge distance based on their weights.
///
/// **Panic!**s if the given graph is not actually a `EulerCircuit`.
pub fn interpolate_euler_circuit(
    ec: &EulerCircuit,
    target_points: u32,
    conf: &InterpolationConfig,
) -> Vec<RawPoint> {
    // Capture a profile of each edge to assist with interpolation.
    struct EdgeProfile {
        a_weight: u32,
        kind: EdgeProfileKind,
    }

    enum EdgeProfileKind {
        Blank,
        Lit {
            distance: f32,
            end_corner: f32,
        }
    }

    impl EdgeProfile {
        // Create an `EdgeProfile` for the edge at the given index.
        fn from_index(e: EdgeIndex, ec: &EulerCircuit) -> Self {
            let e_ref = &ec.raw_edges()[e.index()];
            let a = ec[e_ref.source()];
            let a_weight = a.weight;
            let kind = match e_ref.weight {
                SegmentKind::Blank => EdgeProfileKind::Blank,
                SegmentKind::Lit => {
                    let a_pos = a.position;
                    let b_pos = ec[e_ref.target()].position;
                    let distance = distance_squared(a_pos, b_pos).sqrt();
                    let e_ref = EulerCircuitWalk::new(e_ref.target()).next(ec);
                    let c_pos = ec[e_ref.target()].position;
                    let end_corner = straight_angle_variance(a_pos, b_pos, c_pos);
                    EdgeProfileKind::Lit { distance, end_corner }
                }
            };
            EdgeProfile { a_weight, kind }
        }

        fn is_lit(&self) -> bool {
            match self.kind {
                EdgeProfileKind::Lit { .. } => true,
                EdgeProfileKind::Blank => false,
            }
        }

        // The lit distance covered by this edge.
        fn lit_distance(&self) -> f32 {
            match self.kind {
                EdgeProfileKind::Lit { distance, .. } => distance,
                _ => 0.0,
            }
        }

        // The minimum number of points required to draw the edge.
        fn min_points(&self, conf: &InterpolationConfig) -> u32 {
            match self.kind {
                EdgeProfileKind::Blank => {
                    blank_segment_point_count(self.a_weight, conf.blank_delay_points)
                }
                EdgeProfileKind::Lit { distance, end_corner } => {
                    lit_segment_min_point_count(
                        distance,
                        end_corner,
                        conf.distance_per_point,
                        conf.radians_per_point,
                        self.a_weight,
                    )
                }
            }
        }

        // The points for this edge.
        fn points(
            &self,
            e: EdgeIndex,
            ec: &EulerCircuit,
            conf: &InterpolationConfig,
            excess_points: u32,
        ) -> Vec<RawPoint> {
            let e_ref = &ec.raw_edges()[e.index()];
            let a = ec[e_ref.source()];
            let b = ec[e_ref.target()];
            match self.kind {
                EdgeProfileKind::Blank => {
                    blank_segment_points(a, b.to_raw(), conf.blank_delay_points)
                        .collect()
                }
                EdgeProfileKind::Lit { end_corner, distance } => {
                    let dist_point_count =
                        distance_min_point_count(distance, conf.distance_per_point);
                    let corner_point_count =
                        corner_point_count(end_corner, conf.radians_per_point);
                    let br = b.to_raw();
                    lit_segment_points(a, br, corner_point_count, dist_point_count, excess_points)
                        .collect()
                }
            }
        }
    }

    // If the graph is empty, so is our path.
    let first_n = match ec.node_indices().next() {
        None => return vec![],
        Some(n) => n,
    };

    // The index of each edge in order of the eulerian circuit walk for faster lookup.
    let edge_indices = EulerCircuitWalk::new(first_n)
        .iter(ec)
        .take(ec.edge_count())
        .map(|e| e.id())
        .collect::<Vec<_>>();

    // Create a profile of each edge containing useful information for interpolation.
    let edge_profiles = edge_indices.iter()
        .map(|&ix| EdgeProfile::from_index(ix, ec))
        .collect::<Vec<_>>();

    // The minimum number of points required to display the image.
    let min_points = edge_profiles.iter()
        .map(|ep| ep.min_points(conf))
        .fold(0, |acc, n| acc + n);

    // The target number of points not counting the last to be added at the end.
    let target_points_minus_last = target_points - 1;

    // The excess points distributed across all edges.
    let edge_excess_point_counts = if min_points < target_points_minus_last {
        // A multiplier for determining excess points. This should be distributed across distance.
        let excess_points = target_points_minus_last - min_points;
        // The lit distance covered by each edge.
        let edge_lit_dists = edge_profiles.iter().map(EdgeProfile::lit_distance);
        // The total lit distance covered by the traversal.
        let total_lit_dist = edge_lit_dists.clone().fold(0.0, |acc, d| acc + d);
        // Determine the weights for each edge based on distance.
        let edge_weights = edge_lit_dists.map(|lit_dist| lit_dist / total_lit_dist);

        // Multiply the weight by the excess points. Track fractional error and distribute.
        let mut v = Vec::with_capacity(ec.edge_count());
        let mut err = 0.0;
        let mut count = 0;
        for w in edge_weights {
            if w == 0.0 {
                v.push(0);
                continue;
            }
            let nf = w * excess_points as f32 + err;
            err = nf.fract();
            let n = nf as u32;
            count += n;
            v.push(n);
        }

        // Check for rounding error.
        if count == (excess_points - 1) {
            // Find first lit edge index.
            let (i, _) = edge_profiles.iter()
                .enumerate()
                .find(|&(_, ep)| ep.is_lit())
                .expect("expected at least one lit edge");
            v[i] += 1;
            count += 1;
        }

        // Sanity check that rounding errors have been handled.
        debug_assert_eq!(count, excess_points);

        v
    } else {
        vec![0; ec.edge_count()]
    };

    // Collect all points.
    let total_points = std::cmp::max(min_points, target_points);
    let mut points = Vec::with_capacity(total_points as usize);
    for elem in edge_indices.iter().zip(&edge_profiles).zip(&edge_excess_point_counts) {
        let ((&ix, ep), &excess) = elem;
        points.extend(ep.points(ix, ec, conf, excess));
    }

    // Push the last point.
    let last_point = ec[ec.raw_edges()[edge_indices.last().unwrap().index()].target()];
    points.push(last_point.to_raw());

    // Sanity check that we generated at least `target_points`.
    debug_assert!(points.len() >= target_points as usize);

    points
}

#[cfg(test)]
mod test {
    use crate::point::{Point, Position};
    use hashbrown::HashSet;
    use petgraph::visit::EdgeRef;
    use super::{euler_graph_to_euler_circuit, point_graph_to_euler_graph, points_to_segments,
                segments_to_point_graph};
    use super::{EulerCircuit, EulerCircuitWalk, EulerGraph, PointGraph, SegmentKind};

    fn graph_eq<N, E, Ty, Ix>(
        a: &petgraph::Graph<N, E, Ty, Ix>,
        b: &petgraph::Graph<N, E, Ty, Ix>,
    ) -> bool
    where
        N: PartialEq,
        E: PartialEq,
        Ty: petgraph::EdgeType,
        Ix: petgraph::graph::IndexType + PartialEq,
    {
        let a_ns = a.raw_nodes().iter().map(|n| &n.weight);
        let b_ns = b.raw_nodes().iter().map(|n| &n.weight);
        let a_es = a.raw_edges().iter().map(|e| (e.source(), e.target(), &e.weight));
        let b_es = b.raw_edges().iter().map(|e| (e.source(), e.target(), &e.weight));
        a_ns.eq(b_ns) && a_es.eq(b_es)
    }

    fn is_euler_graph<N, E, Ty, Ix>(g: &petgraph::Graph<N, E, Ty, Ix>) -> bool
    where
        Ty: petgraph::EdgeType,
        Ix: petgraph::graph::IndexType,
    {
        let even_degree = g.node_indices().all(|n| g.edges(n).count() % 2 == 0);
        let strongly_connected = petgraph::algo::kosaraju_scc(g).len() == 1;
        even_degree && strongly_connected
    }

    fn white_pt(position: Position) -> Point {
        Point {
            position,
            color: [1.0; 3],
            weight: 0,
        }
    }

    fn blank_pt(position: Position) -> Point {
        Point {
            position,
            color: [0.0; 3],
            weight: 0,
        }
    }

    fn square_pts() -> [Point; 5] {
        let a = white_pt([-1.0, -1.0]);
        let b = white_pt([-1.0, 1.0]);
        let c = white_pt([1.0, 1.0]);
        let d = white_pt([1.0, -1.0]);
        [a, b, c, d, a]
    }

    fn two_vertical_lines_pts() -> [Point; 8] {
        let a = [-1.0, -1.0];
        let b = [-1.0, 1.0];
        let c = [1.0, -1.0];
        let d = [1.0, 1.0];
        [
            white_pt(a),
            white_pt(b),
            blank_pt(b),
            blank_pt(c),
            white_pt(c),
            white_pt(d),
            blank_pt(d),
            blank_pt(a),
        ]
    }

    #[test]
    fn test_points_to_point_graph_no_blanks() {
        let pts = square_pts();
        let segs = points_to_segments(pts.iter().cloned());
        let pg = segments_to_point_graph(segs);

        let mut expected = PointGraph::default();
        let na = expected.add_node(pts[0]);
        let nb = expected.add_node(pts[1]);
        let nc = expected.add_node(pts[2]);
        let nd = expected.add_node(pts[3]);
        expected.add_edge(na, nb, ());
        expected.add_edge(nb, nc, ());
        expected.add_edge(nc, nd, ());
        expected.add_edge(nd, na, ());

        assert!(graph_eq(&pg, &expected));
    }

    #[test]
    fn test_points_to_point_graph_with_blanks() {
        let pts = two_vertical_lines_pts();
        let segs = points_to_segments(pts.iter().cloned());
        let pg = segments_to_point_graph(segs);

        let mut expected = PointGraph::default();
        let na = expected.add_node(pts[0]);
        let nb = expected.add_node(pts[1]);
        let nc = expected.add_node(pts[4]);
        let nd = expected.add_node(pts[5]);
        expected.add_edge(na, nb, ());
        expected.add_edge(nc, nd, ());

        assert!(graph_eq(&pg, &expected));
    }

    #[test]
    fn test_point_graph_to_euler_graph_no_blanks() {
        let pts = square_pts();
        let segs = points_to_segments(pts.iter().cloned());
        let pg = segments_to_point_graph(segs);
        let eg = point_graph_to_euler_graph(&pg);

        let mut expected = EulerGraph::default();
        let na = expected.add_node(pts[0]);
        let nb = expected.add_node(pts[1]);
        let nc = expected.add_node(pts[2]);
        let nd = expected.add_node(pts[3]);
        expected.add_edge(na, nb, SegmentKind::Lit);
        expected.add_edge(nb, nc, SegmentKind::Lit);
        expected.add_edge(nc, nd, SegmentKind::Lit);
        expected.add_edge(nd, na, SegmentKind::Lit);

        assert!(graph_eq(&eg, &expected));
    }

    #[test]
    fn test_point_graph_to_euler_graph_with_blanks() {
        let pts = two_vertical_lines_pts();
        let segs = points_to_segments(pts.iter().cloned());
        let pg = segments_to_point_graph(segs);
        let eg = point_graph_to_euler_graph(&pg);

        assert!(is_euler_graph(&eg));

        let pg_ns: Vec<_> = pg.raw_nodes().iter().map(|n| n.weight).collect();
        let eg_ns: Vec<_> = eg.raw_nodes().iter().map(|n| n.weight).collect();
        assert_eq!(pg_ns, eg_ns);

        assert_eq!(eg.raw_edges().iter().filter(|e| e.weight == SegmentKind::Blank).count(), 2);
        assert_eq!(eg.raw_edges().iter().filter(|e| e.weight == SegmentKind::Lit).count(), 2);
    }

    #[test]
    fn test_euler_graph_to_euler_circuit_no_blanks() {
        let pts = square_pts();
        let segs = points_to_segments(pts.iter().cloned());
        let pg = segments_to_point_graph(segs);
        let eg = point_graph_to_euler_graph(&pg);
        let ec = euler_graph_to_euler_circuit(&eg);

        let mut expected = EulerCircuit::new();
        let na = expected.add_node(pts[0]);
        let nb = expected.add_node(pts[1]);
        let nc = expected.add_node(pts[2]);
        let nd = expected.add_node(pts[3]);
        expected.add_edge(na, nb, SegmentKind::Lit);
        expected.add_edge(nb, nc, SegmentKind::Lit);
        expected.add_edge(nc, nd, SegmentKind::Lit);
        expected.add_edge(nd, na, SegmentKind::Lit);

        assert!(graph_eq(&ec, &expected));
    }

    #[test]
    fn test_euler_graph_to_euler_circuit_with_blanks() {
        let pts = two_vertical_lines_pts();
        let segs = points_to_segments(pts.iter().cloned());
        let pg = segments_to_point_graph(segs);
        let eg = point_graph_to_euler_graph(&pg);
        let ec = euler_graph_to_euler_circuit(&eg);

        let pg_ns: Vec<_> = pg.raw_nodes().iter().map(|n| n.weight).collect();
        let ec_ns: Vec<_> = ec.raw_nodes().iter().map(|n| n.weight).collect();
        assert_eq!(pg_ns, ec_ns);

        assert_eq!(ec.raw_edges().iter().filter(|e| e.weight == SegmentKind::Blank).count(), 2);
        assert_eq!(ec.raw_edges().iter().filter(|e| e.weight == SegmentKind::Lit).count(), 2);

        let mut visited = HashSet::new();
        let mut walk = EulerCircuitWalk::new(ec.node_indices().next().unwrap());
        while visited.len() < 4 {
            let e_ref = walk.next(&ec);
            assert!(visited.insert(e_ref.id()));
        }
    }
}
